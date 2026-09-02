//! Turn-to-turn continuation bookkeeping shared by the Responses API backends.
//!
//! `LlmBackend::stream_chat` hands every backend the *full* conversation on
//! every call and carries no conversation id, so a backend that wants the
//! server's prompt cache to keep matching as the conversation grows has to
//! recognize "this call continues the call I made last turn" on its own.
//!
//! Responses API backends need exactly that recognition, and they do it the
//! same way: hash the ordered message context, remember what that context
//! produced, and on the next turn look for the largest stored prefix of the
//! new message list. Backends can store different continuation values under
//! that key, so the cache is generic over the stored value. The prefix
//! matching, the divergence reset, and the eviction behavior are shared,
//! not reimplemented per backend.

use std::collections::{HashMap, VecDeque};

use crate::llm_client::{ChatContentPart, ChatMessage};

/// How many message-context -> value entries a chain cache keeps before
/// evicting the oldest. Bounds memory for long-lived processes with
/// many/large conversations without needing an `lru` crate dependency.
pub(crate) const RESPONSES_CHAIN_CACHE_CAP: usize = 512;

/// Bounded content-keyed cache of "what did this exact message context
/// produce last time". Plain `HashMap` + FIFO insertion-order eviction -- no
/// promote-on-read -- which is enough to bound memory; see
/// `RESPONSES_CHAIN_CACHE_CAP`.
#[derive(Debug)]
pub(crate) struct ResponsesChainCache<V> {
    entries: HashMap<u64, V>,
    order: VecDeque<u64>,
    cap: usize,
}

impl<V: Clone> ResponsesChainCache<V> {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            cap,
        }
    }

    pub(crate) fn get(&self, key: u64) -> Option<V> {
        self.entries.get(&key).cloned()
    }

    pub(crate) fn insert(&mut self, key: u64, value: V) {
        if !self.entries.contains_key(&key) {
            self.order.push_back(key);
            while self.order.len() > self.cap {
                if let Some(oldest) = self.order.pop_front() {
                    self.entries.remove(&oldest);
                }
            }
        }
        self.entries.insert(key, value);
    }
}

/// Hashes an ordered Responses API message context (role + normalized
/// content + tool-call/tool-result fields, in order) to a stable 64-bit key
/// for `ResponsesChainCache`. Order-sensitive and content-sensitive: any
/// difference in message order, text, or tool-call identity/arguments
/// produces a different hash.
pub(crate) fn hash_responses_context(messages: &[ChatMessage]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    messages.len().hash(&mut hasher);
    for msg in messages {
        hash_responses_message(msg, &mut hasher);
    }
    hasher.finish()
}

fn hash_responses_message(msg: &ChatMessage, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    msg.role.hash(hasher);
    for part in &msg.content {
        match part {
            ChatContentPart::Text { text } => {
                0u8.hash(hasher);
                text.hash(hasher);
            }
            ChatContentPart::Image { image_url } => {
                1u8.hash(hasher);
                image_url.hash(hasher);
            }
        }
    }
    match &msg.tool_calls {
        Some(calls) => {
            calls.len().hash(hasher);
            for call in calls {
                call.id.hash(hasher);
                call.r#type.hash(hasher);
                call.function.name.hash(hasher);
                call.function.arguments.hash(hasher);
            }
        }
        None => 0usize.hash(hasher),
    }
    msg.tool_call_id.hash(hasher);
    msg.name.hash(hasher);
}

/// Finds the largest cached prefix of `messages` that matches a previously
/// stored Responses API context, so the delta sent as the next turn's input
/// is as small as possible.
///
/// Walks assistant-message boundary indices from the *end* of `messages`
/// toward the start. For each assistant index `a`, looks up
/// `hash(messages[0..a])`: a hit means messages `0..a` were exactly the
/// input of a previously stored response, and `messages[a]` is that
/// response's assistant turn now echoed back into history. The first
/// (largest-`a`) hit wins. Returns `(a, value)`; a prompt-cache caller reuses
/// the stored session identity for the turn.
///
/// Returns `None` on a fresh conversation, an evicted/never-seen prefix, or
/// a first turn -- callers fall back to a fresh, unchained turn.
pub(crate) fn find_responses_continuation<V>(
    messages: &[ChatMessage],
    lookup: impl Fn(u64) -> Option<V>,
) -> Option<(usize, V)> {
    for a in (0..messages.len()).rev() {
        if messages[a].role != "assistant" {
            continue;
        }
        let hash = hash_responses_context(&messages[..a]);
        if let Some(value) = lookup(hash) {
            return Some((a, value));
        }
    }
    None
}
