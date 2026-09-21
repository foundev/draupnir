//! Credential access for Inceptron.
//!
//! Inceptron keys are static (no refresh, no expiry): the user pastes one
//! once via `/setup inceptron key <key>` and we reuse it until they rotate
//! or disconnect. Persistence is opt-in: users who export
//! `INCEPTRON_API_KEY` in their shell get the zero-config behaviour, and
//! on-disk state is only created by the setup command.
//!
//! Storage lives in the consolidated [`crate::secrets`] store
//! (`<config>/brokk/secrets.json`, 0600, atomic), like DeepSeek and
//! OpenRouter. There is no legacy per-provider file to fall back to.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::discovery::INCEPTRON_API_KEY_ENV;

/// Flat one-field record. Inceptron keys are static so there's nothing
/// more to persist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InceptronAuth {
    pub api_key: String,
}

/// Snapshot of where Inceptron credentials currently come from. Single
/// source of truth for the "env owns" contract, mirroring the sibling
/// providers: whenever `INCEPTRON_API_KEY` is non-empty the environment
/// owns the credential lifecycle and `/setup inceptron key` explains
/// rather than mutating state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialState {
    pub env_set: bool,
    pub file_present: bool,
}

impl CredentialState {
    pub fn snapshot() -> Self {
        let env_set = std::env::var(INCEPTRON_API_KEY_ENV)
            .ok()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let file_present = match read() {
            Ok(Some(auth)) => !auth.api_key.trim().is_empty(),
            _ => false,
        };
        Self {
            env_set,
            file_present,
        }
    }

    /// Where the active credential, if any, is being read from. Mirrors
    /// the precedence in `build_inceptron_backend`: env wins over file,
    /// file wins over nothing.
    pub fn active_source(&self) -> &'static str {
        if self.env_set {
            "env"
        } else if self.file_present {
            "file"
        } else {
            "none"
        }
    }

    /// True when the environment owns the credential lifecycle.
    pub fn env_owns(&self) -> bool {
        self.env_set
    }
}

/// Read the stored Inceptron credentials from the consolidated store. A
/// malformed store degrades to "no stored key" with a warning; the env
/// var keeps working regardless.
pub fn read() -> Result<Option<InceptronAuth>> {
    match crate::secrets::read() {
        Ok(Some(secrets)) => Ok(secrets.inceptron),
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::warn!("secrets store unreadable; treating Inceptron as not stored: {e:#}");
            Ok(None)
        }
    }
}

/// Persist the key into the consolidated secrets store.
pub fn write(auth: &InceptronAuth) -> Result<()> {
    crate::secrets::update(|secrets| secrets.inceptron = Some(auth.clone()))
}

/// Best-effort logout: clear the stored key. Missing state is not an
/// error -- `/setup inceptron disconnect` is idempotent and must not
/// create a store when nothing was ever saved. A malformed store is
/// quarantined by `update` so disconnect self-heals broken state.
pub fn logout() -> Result<()> {
    match crate::secrets::read() {
        Ok(Some(secrets)) if secrets.inceptron.is_some() => {
            crate::secrets::update(|secrets| secrets.inceptron = None)
        }
        Ok(_) => Ok(()),
        Err(_) => crate::secrets::update(|secrets| secrets.inceptron = None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openrouter_auth::test_support::{ENV_GUARD, EnvScope};

    #[test]
    fn credential_state_reports_sources() {
        let _lock = ENV_GUARD.blocking_lock();
        let tmp = tempfile::tempdir().unwrap();
        let _brokk = EnvScope::set("BROKK_CONFIG_HOME", tmp.path());

        {
            let _env = EnvScope::set(INCEPTRON_API_KEY_ENV, "ic-env");
            let state = CredentialState::snapshot();
            assert!(state.env_set && state.env_owns());
            assert_eq!(state.active_source(), "env");
        }

        let _env = EnvScope::remove(INCEPTRON_API_KEY_ENV);
        let state = CredentialState::snapshot();
        assert_eq!(state.active_source(), "none");

        write(&InceptronAuth {
            api_key: "ic-file".into(),
        })
        .unwrap();
        let state = CredentialState::snapshot();
        assert!(state.file_present && !state.env_owns());
        assert_eq!(state.active_source(), "file");
    }

    #[test]
    fn logout_is_idempotent_and_never_creates_a_store() {
        let _lock = ENV_GUARD.blocking_lock();
        let tmp = tempfile::tempdir().unwrap();
        let _brokk = EnvScope::set("BROKK_CONFIG_HOME", tmp.path());

        logout().unwrap();
        assert!(
            !crate::secrets::secrets_path().unwrap().exists(),
            "disconnect with nothing saved must not create secrets.json"
        );

        write(&InceptronAuth {
            api_key: "ic".into(),
        })
        .unwrap();
        logout().unwrap();
        assert!(read().unwrap().is_none());
        logout().unwrap();
    }
}
