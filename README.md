# clawscope

A CLI tool to inspect [OpenClaw](https://openclaw.dev) agent sessions.

## Features

- List all OpenClaw sessions with their status and last active time
- Highlights active sessions (updated within the last 5 minutes)
- Human-readable relative timestamps ("2 minutes ago", "1 hour ago", etc.)

## Installation

### From source

```bash
git clone https://github.com/conao3/rust-clawscope
cd rust-clawscope
cargo build --release
# Binary available at: ./target/release/clawscope
```

### With Nix

```bash
nix run github:conao3/rust-clawscope -- sessions
```

Or install to your profile:

```bash
nix profile install github:conao3/rust-clawscope
```

## Usage

### List sessions

```bash
clawscope sessions
```

**Example output:**

```
SESSION                                          STATUS    LAST ACTIVE
---------------------------------------------------------------------------
agent:main:main                                  active    2 minutes ago
agent:main:telegram:group:-1003884066398:topic:1 stopped   1 hour ago
agent:codex-usage:main                           stopped   8 hours ago
```

Sessions are sorted with active sessions first, then by most recently updated.

**Status values:**
- `active` — session was updated within the last 5 minutes
- `stopped` — session has not been updated recently

## Data Source

Reads from `~/.openclaw/agents/main/sessions/sessions.json`.

## Requirements

- OpenClaw must be installed and have been run at least once
- Rust 1.70+ (for building from source)

## License

MIT
