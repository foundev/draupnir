---
title: JetBrains
description: Install, configure, validate, and troubleshoot Draupnir in JetBrains ACP clients.
---

JetBrains ACP clients can launch Draupnir as a custom agent server. Use an absolute path to the installed or locally built executable.

## Automatic Configuration

Run the installed Draupnir binary:

```bash
draupnir install jetbrains
```

Draupnir detects its own executable path and merges an `Draupnir` agent into
`~/.jetbrains/acp.json`. If an `Draupnir` entry already exists, review it and then
pass `--force` to replace only that entry.

## Manual Configuration

Add an entry to `~/.jetbrains/acp.json`:

```json
{
  "agent_servers": {
    "Draupnir": {
      "command": "/absolute/path/to/draupnir",
      "args": [],
      "env": {}
    }
  }
}
```

Preserve unrelated entries already present in the file. Use `draupnir.exe` on Windows. Restart the agent UI or start a fresh session after changing this configuration.

## Source-Checkout Helper

From a Draupnir checkout:

```bash
rustup target add wasm32-wasip2
cargo xtask build-acp-for-jetbrains
```

The helper builds `target/release/draupnir` and writes a `Brokk Code (Rust Local)` entry. Pass `--config <path>` when the active JetBrains ACP configuration is elsewhere. The helper is development tooling and is not packaged by `cargo install`.

## Setup and Validation

Open a new Draupnir session and run `/setup` if no model is ready. Use the editor's session controls to select Permission, behavior, model, reasoning effort, and service tier.

Validate with a source question that requires managed Bifrost:

```text
Use Bifrost search_symbols to find main and get_symbol_sources to read it. Name the tools you called and do not edit files.
```

A visible Bifrost tool card plus repository-specific source output proves the code-intelligence path. If the agent falls back to reading files, confirm the session is using Draupnir and inspect stderr for MCP startup failures.

The [Zed-led ten-minute evaluation](/evaluate-draupnir/) uses the same Draupnir behavior. Substitute this configuration and JetBrains' permission controls for the editor-specific steps.
