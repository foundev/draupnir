<h1 align="center">Draupnir</h1>

<p align="center">
  <a href="https://draupnir.brokk.ai/">
    <img src="docs/public/draupnir-social-card.png" alt="Draupnir — one portable agent runtime for your ACP interface" width="720">
  </a>
</p>

<p align="center">
  <a href="https://github.com/BrokkAi/draupnir/actions/workflows/ci.yml"><img src="https://github.com/BrokkAi/draupnir/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/BrokkAi/draupnir/releases/latest"><img src="https://img.shields.io/github/v/release/BrokkAi/draupnir" alt="Latest release"></a>
  <a href="https://crates.io/crates/brokk-draupnir"><img src="https://img.shields.io/crates/v/brokk-draupnir" alt="crates.io"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/BrokkAi/draupnir" alt="LGPL-3.0-only"></a>
</p>

<p align="center">
  <a href="#install-and-evaluate">Quickstart</a> ·
  <a href="https://draupnir.brokk.ai/">Documentation</a> ·
  <a href="https://draupnir.brokk.ai/evaluate-draupnir/">Ten-minute evaluation</a> ·
  <a href="https://discord.gg/geYkWUeH">Discord</a>
</p>

Draupnir is a Rust [Agent Client Protocol](https://agentclientprotocol.com/) server: one reusable agent engine for editors, bots, TUIs, and internal tools.

It is intentionally **just the ACP server**. Draupnir owns model routing, the tool loop, permission enforcement, sessions, context compaction, sandboxing, and MCP integration. The ACP client owns the user experience.

```text
ACP client              stdio / JSON-RPC              Draupnir
----------              ----------------              -----
Zed        ----------------------------------------->  agent loop
JetBrains   --------------------------------------->  model routing
issue bot   --------------------------------------->  permission gate
custom TUI  --------------------------------------->  tools + sessions
```

## Why Draupnir

- **One engine, many frontends.** Reuse the same execution semantics from a supported editor or any ACP-over-stdio client.
- **Model routing built in.** Connect Codex/ChatGPT, Ollama, ds4, DeepSeek, Kimi, Grok, OpenAI-compatible providers, or OpenRouter.
- **Real agent tooling.** Filesystem and shell tools, managed Bifrost code intelligence, MCP servers, skills, plugins, and subagents.
- **Explicit safety boundaries.** Clients choose permission behavior while Draupnir applies permission gates, workspace path checks, and the configured sandbox strategy.
- **Persistent work.** Load and resume sessions with durable history, usage reporting, and context compaction.

## Install and Evaluate

Install with Homebrew on macOS (Apple Silicon and Intel) or Linux (x86-64
and ARM64 glibc):

```bash
brew install brokkai/tap/draupnir
draupnir --version
```

Or install the latest checksum-verified release on macOS, Linux, WSL, or Android/Termux:

```bash
curl -fsSL https://raw.githubusercontent.com/BrokkAi/draupnir/refs/heads/master/install.sh | bash
draupnir --version
```

To review the installer before running it, download `install.sh`, inspect it, then run it with `bash`. You can also download an archive from [GitHub Releases](https://github.com/BrokkAi/draupnir/releases/latest), or install from crates.io:

```bash
rustup target add wasm32-wasip2
cargo install brokk-draupnir --locked --force
draupnir --version
```

Running `draupnir` directly starts a stdio JSON-RPC server; use it through an ACP client. Continue with the [installation guide](https://draupnir.brokk.ai/install/) or the reproducible [ten-minute evaluation](https://draupnir.brokk.ai/evaluate-draupnir/).

For scripts, CI jobs, and Makefiles, `draupnir --print` runs one prompt headlessly with a built-in ACP client and exits — no external client required:

```bash
draupnir -p "summarize the failing test"       # prompt as an argument
echo "fix this test" | draupnir -p             # or on stdin (also: draupnir -p -)
draupnir -p "…" --output-format json           # one JSON object at exit
draupnir -p "…" --output-format stream-json    # newline-delimited records, result last
draupnir -p "…" --permission-mode yolo         # manual (default) | auto | yolo
draupnir -p "…" --cwd /path/to/repo --model "codex::gpt-5-codex+high"
draupnir -p "…" --resume SESSION_ID            # continue an existing session
```

Headless runs never block on a human: `manual` rejects every permission request but still honors agent-side approvals that never reach the client (`read`, `search`, and `fetch`; sandboxed shell commands on the conservative read-only safelist; and remembered repo-scoped **Always allow** grants); `auto` accepts edit/delete/move but rejects shell execution; and `yolo` accepts everything. The final assistant message goes to stdout (`text`), or a machine-readable payload with `session_id`, `resumed`, `result`, `stop_reason`, `usage`, and `error` (`json` / `stream-json`); progress and diagnostics stay on stderr, and the exit code is non-zero on agent or transport failure ([#356](https://github.com/BrokkAi/draupnir/issues/356)).

`draupnir serve` (included in all official binaries; source builds can opt out with `cargo build --no-default-features --features wasm-sandbox`) starts the same runtime as an HTTP daemon on a loopback listener (default `127.0.0.1:26845`), exposing versioned REST endpoints for session lifecycle, model discovery, the tool catalog, and asynchronous prompt runs with Server-Sent-Event streaming and cancellation (`/health`, `/v1/models`, `/v1/tools`, `/v1/sessions`, `/v1/runs`). The daemon supports bearer-token authentication (required for non-loopback binding), server-enforced workspace roots, and interactive permission approval endpoints. The wire contract is versioned and checked in under [`openapi/`](openapi/) (OpenAPI document, SSE event schema, and compatibility policy), with a black-box conformance suite holding the implementation to it. Generate a client in your preferred language from those contract files.

Configure a supported ACP client directly from the installed binary:

```bash
draupnir install zed
draupnir install jetbrains
draupnir install neovim --plugin codecompanion
```

## Supported Clients

- [Zed](https://draupnir.brokk.ai/zed/), [JetBrains](https://draupnir.brokk.ai/jetbrains/), and [Neovim](https://draupnir.brokk.ai/neovim/) have direct installers and verified configuration shapes.
- [Other ACP clients](https://draupnir.brokk.ai/other-acp-clients/) can launch Draupnir as a custom stdio agent.
- The [client-building guide](https://draupnir.brokk.ai/build-acp-client/) and `examples/` show issue triage, review, and issue-drafting automations.

## Documentation and Development

The [Draupnir documentation](https://draupnir.brokk.ai/) is the canonical user reference for providers, commands, tools, permissions, sessions, extensibility, and trust boundaries.

See [CONTRIBUTING.md](CONTRIBUTING.md) for source development, runtime invariants, tests, dependency-license policy, pull requests, and releases.

## License

Draupnir is licensed under `LGPL-3.0-only`. See the practical [License and Use Cases](https://draupnir.brokk.ai/license-use-cases/) guide, the controlling [LICENSE](LICENSE), and [third-party notices](https://draupnir.brokk.ai/third-party-notices/).
