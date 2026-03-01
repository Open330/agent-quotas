# Claude Code Quota Monitor

A lightweight monitoring system for teams sharing Claude Code (Max/Pro subscriptions) to track per-user quota consumption across 5-hour and 7-day rolling windows.

## Quick Start

```bash
# Set up the client hook (interactive)
./setup.sh
```

Or tell your AI agent: "Read AGENTS.md and set up the quota monitor hook."

## Architecture

```
┌─────────────────┐     POST /api/report     ┌──────────────────┐
│  User A machine  │ ──────────────────────> │                  │
│  (hook script)   │                          │   Rust Server    │
├─────────────────┤                          │   (axum)         │
│  User B machine  │ ──────────────────────> │                  │
│  (hook script)   │                          │  ┌────────────┐  │
├─────────────────┤                          │  │  SQLite DB  │  │
│  User C machine  │ ──────────────────────> │  └────────────┘  │
│  (hook script)   │                          │                  │
└─────────────────┘                          │  GET / (dashboard)│
                                              └──────────────────┘
```

## Server Setup

**Prerequisites:** Rust 1.75+ (install via [rustup](https://rustup.rs/))

```bash
cargo build --release
./target/release/claude-quota --port 3000 --database quota.db
```

Open the dashboard at `http://localhost:3000`.

| Flag | Description | Default |
|------|-------------|---------|
| `--port` / `-p` | HTTP server port | `3000` |
| `--database` / `-d` | SQLite database path | `claude-quota.db` |

## Client Hook Setup

Each user's machine needs the hook script and a config file.

### 1. Create config file

```bash
cat > ~/.claude-quota-hook.json << 'EOF'
{
  "server_url": "http://your-server:3000",
  "username": "alice"
}
EOF
```

### 2. Register the Stop hook

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/absolute/path/to/claude-quota/hook/claude-quota-hook.sh",
            "timeout": 10000
          }
        ]
      }
    ]
  }
}
```

Or run `./setup.sh` to do this automatically.

### 3. Verify

```bash
curl -s http://your-server:3000/api/users | jq .
```

## How It Works

1. Claude Code fires the **Stop hook** after every response
2. The hook script reads the transcript JSONL, extracts token usage from new assistant messages
3. It runs `claude usage` to capture the current 5h/7d quota percentages
4. A report is POSTed to the server with a deterministic `report_id` (SHA-256 of session + line range)
5. The server stores the report in SQLite (duplicates return 409 and are ignored)
6. Failed uploads are queued in `~/.claude-quota-queue.json` and retried next invocation

## API Reference

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/report` | Submit usage report. Returns `201` or `409` (duplicate) |
| `GET` | `/api/users` | List all users with last active time |
| `GET` | `/api/usage?window=5h` | Usage summaries by window (`5h`, `24h`, `7d`, `all`) |
| `GET` | `/api/summary` | Both 5h and 7d summaries with latest quota percentages |
| `GET` | `/api/hourly` | Hourly token breakdown for timeline chart |
| `GET` | `/` | Dashboard |

### Report Payload

```json
{
  "username": "alice",
  "session_id": "uuid",
  "report_id": "sha256-hash",
  "timestamp": "2026-03-02T12:00:00Z",
  "model": "claude-opus-4-6",
  "input_tokens": 15000,
  "output_tokens": 3000,
  "cache_read_input_tokens": 20000,
  "cache_creation_input_tokens": 5000,
  "message_count": 5,
  "tool_use_count": 3,
  "usage_percent_5h": 42.3,
  "usage_percent_7d": 12.1
}
```

## Dashboard

The dashboard shows per-user circular gauges for 5h and 7d quota usage percentages, color-coded:

- **Green**: < 50%
- **Yellow**: 50-75%
- **Orange**: 75-90%
- **Red**: > 90%

Plus a timeline chart of hourly token consumption. Auto-refreshes every 30 seconds.

## File Structure

```
claude-quota/
├── Cargo.toml                        # Rust dependencies
├── src/
│   ├── main.rs                       # axum server, CLI args, router
│   ├── db.rs                         # SQLite (WAL mode, Mutex<Connection>)
│   ├── api.rs                        # HTTP handlers
│   ├── models.rs                     # Serde request/response structs
│   └── dashboard.rs                  # Inline HTML/CSS/JS with Chart.js
├── hook/
│   └── claude-quota-hook.sh          # Client-side Stop hook (bash)
├── setup.sh                          # Interactive setup script
├── AGENTS.md                         # Setup guide for AI agents
└── README.md
```

## Troubleshooting

| Issue | Check |
|-------|-------|
| Hook not firing | Verify path in `~/.claude/settings.json` is absolute and correct |
| No data on dashboard | Check `~/.claude-quota-hook.json` has correct `server_url` |
| Connection refused | Ensure server is running: `curl http://server:3000/api/users` |
| Queued reports | Check `~/.claude-quota-queue.json` for pending uploads |
| Duplicate 409s | Expected behavior — the hook safely retries without double-counting |
