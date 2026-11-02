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
nix run github:conao3/rust-clawscope -- session list-sessions --output text
```

Or install to your profile:

```bash
nix profile install github:conao3/rust-clawscope
```

## Usage

The CLI takes input in the AWS CLI grammar: `clawscope [global options] <command> <subcommand> [--parameter value ...]`.

### List sessions

```bash
clawscope session list-sessions --output text
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

Without `--output text`, clawscope prints the same list as JSON:

```json
{
  "Sessions": [
    {
      "SessionKey": "agent:main:main",
      "Status": "active",
      "UpdatedAt": "2026-02-23T06:53:08+00:00",
      "LastActive": "2 minutes ago"
    }
  ]
}
```

**Status values:**
- `active` — session was updated within the last 5 minutes
- `stopped` — session has not been updated recently

### Parameters

`clawscope session list-sessions`:

- `--sessions-file <PATH>`: OpenClaw `sessions.json` to read (default `~/.openclaw/agents/main/sessions/sessions.json`).

Common to every subcommand:

- A value starting with `file://` is replaced by the content of that file. Read stdin with `file:///dev/stdin`.
- `--cli-input-json file://input.json` passes all parameters as a JSON object keyed by the PascalCase parameter names (`SessionsFile`). Parameters given on the command line take precedence.
- `--generate-cli-skeleton input|output` prints the input or output skeleton.
- Global options: `--output json|text|table` (default `json`; `text` prints the session table), `--query <JMESPath>` (for example `--query "Sessions[?Status=='active'].SessionKey" --output text`), `--debug`, `--version`.
- `help` works at every level: `clawscope help`, `clawscope session help`, `clawscope session list-sessions help`.
- Exit codes: `0` success, `252` syntax or parameter error, `254` the sessions file cannot be read or parsed, `255` other failures.

## Data Source

Reads from `~/.openclaw/agents/main/sessions/sessions.json`, or the file given with `--sessions-file`.

## Requirements

- OpenClaw must be installed and have been run at least once
- Rust 1.70+ (for building from source)

## License

MIT
