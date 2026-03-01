# Claude Quota Monitor

A lightweight quota monitoring server for teams sharing Claude Code, OpenAI Codex CLI, or Google Antigravity subscriptions. Tracks per-user token consumption across 5-hour and 7-day rolling windows with a Rust backend, SQLite storage, and a React admin panel.

## Quick Start

```bash
# 1. Start the server
cargo build --release
./target/release/claude-quota --port 3000 --database quota.db

# 2. Start the admin panel (optional)
cd admin && npm install && npm run dev

# 3. Set up client hooks (interactive)
./setup.sh
```

Or tell your AI agent: "Read AGENTS.md and set up the quota monitor hook."

---

## Architecture

```
┌─────────────────────┐    POST /api/report    ┌─────────────────────────┐
│  User A (Claude)    │ ──────────────────────>│                         │
│  hook script        │                         │   Rust Server (axum)    │
├─────────────────────┤                         │                         │
│  User B (Codex)     │ ──────────────────────>│   ┌─────────────────┐   │
│  hook script        │                         │   │   SQLite (WAL)  │   │
├─────────────────────┤                         │   └─────────────────┘   │
│  User C (Antigrav.) │ ──────────────────────>│                         │
│  manual reporter    │                         │   GET /  (dashboard)    │
└─────────────────────┘                         │   /api/* (REST API)     │
                                                 └─────────────────────────┘
                                                           │
                                                    ┌──────┴──────┐
                                                    │  Admin UI   │
                                                    │ (React 19)  │
                                                    │  Vite dev   │
                                                    │  proxy :3000│
                                                    └─────────────┘
```

---

## Supported Tools

| Tool | Hook File | Trigger |
|------|-----------|---------|
| Claude Code | `hook/claude-quota-hook.sh` | `Stop` hook (automatic after each response) |
| OpenAI Codex CLI | `hook/codex-hook.sh` | `notify` hook |
| Google Antigravity | `hook/antigravity-hook.sh` | Manual invocation |

---

## Server Setup

**Prerequisites:** Rust 1.83+ (install via [rustup](https://rustup.rs/))

```bash
cargo build --release
./target/release/claude-quota --port 3000 --database quota.db
```

The server auto-generates an admin PAT on first run and prints it to stdout. Save it — it cannot be recovered, but can be regenerated via the admin API.

Open the built-in dashboard at `http://localhost:3000`.

### Server Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--port` / `-p` | HTTP server port | `3000` |
| `--database` / `-d` | SQLite database path | `claude-quota.db` |

---

## Admin Panel Setup

The admin panel is a Vite + React 19 app that provides user management and statistics.

```bash
cd admin
npm install        # Node 24+ required
npm run dev        # Starts at http://localhost:5173 with proxy to :3000
```

For production, build and serve statically:

```bash
npm run build      # Outputs to admin/dist/
```

### Admin Panel Pages

| Page | Path | Description |
|------|------|-------------|
| Login | `/login` | PAT authentication |
| Stats | `/stats` | Usage gauges and system statistics |
| Users | `/users` | Create, delete, regenerate tokens |

---

## Client Hook Setup

Each user's machine needs the hook script and a config file. Run `./setup.sh` for interactive setup, or follow the manual steps below.

### Config File

```bash
cat > ~/.claude-quota-hook.json << 'EOF'
{
  "server_url": "http://your-server:3000",
  "username": "alice",
  "token": "your-48-char-pat-here"
}
EOF
```

| Field | Description |
|-------|-------------|
| `server_url` | Base URL of the claude-quota server |
| `username` | Display name for this user |
| `token` | 48-character PAT issued by admin |

### Claude Code Hook

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

The hook fires automatically after every Claude Code response. It reads the session JSONL transcript, extracts token counts from new assistant messages, runs `claude usage` for quota percentages, and POSTs a report to the server.

### OpenAI Codex CLI Hook

Add to your Codex CLI configuration:

```json
{
  "hooks": {
    "notify": "/absolute/path/to/claude-quota/hook/codex-hook.sh"
  }
}
```

### Google Antigravity Hook

Run manually after a session:

```bash
/absolute/path/to/claude-quota/hook/antigravity-hook.sh
```

---

## How It Works

1. A tool fires a hook after each response or session.
2. The hook reads usage data (tokens, model, quota percentages).
3. A report is POSTed to `/api/report` with a deterministic `report_id` (SHA-256 of session + line range) to prevent duplicates.
4. The server stores the report in SQLite — duplicate `report_id`s return `409` and are ignored.
5. Failed uploads are queued in `~/.claude-quota-queue.json` and retried on next invocation.

---

## Authentication

The server uses **Personal Access Tokens (PATs)**:

- Tokens are 48 random characters (URL-safe base64).
- An admin token is auto-generated on first run.
- Regular users are created by an admin via the API or admin UI.
- Tokens are sent as `Authorization: Bearer <token>` headers.
- Two roles: **user** (can submit reports) and **admin** (full access).

---

## API Reference

### Public Endpoints (no auth required)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Built-in HTML dashboard with SVG quota gauges |
| `GET` | `/api/users` | List all users with last-active timestamps |
| `GET` | `/api/usage?window=5h` | Usage summaries by window (`5h`, `24h`, `7d`, `all`) |
| `GET` | `/api/summary` | Both 5h and 7d windows with latest quota percentages |
| `GET` | `/api/hourly` | Hourly token breakdown for timeline chart |

### Authenticated Endpoints (Bearer token required)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/report` | Submit a usage report. Returns `201` or `409` (duplicate) |

### Admin Endpoints (admin Bearer token required)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/admin/users` | List all users |
| `POST` | `/api/admin/users` | Create a user (returns generated PAT) |
| `DELETE` | `/api/admin/users/:id` | Delete a user |
| `POST` | `/api/admin/users/:id/regenerate-token` | Issue a new PAT for a user |
| `GET` | `/api/admin/stats` | System statistics |

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

---

## Dashboard

The built-in dashboard (served at `/`) shows per-user circular SVG gauges for 5h and 7d quota usage percentages, color-coded:

| Color | Range |
|-------|-------|
| Green | < 50% |
| Yellow | 50–75% |
| Orange | 75–90% |
| Red | > 90% |

Also includes an hourly token timeline. Auto-refreshes every 30 seconds.

The admin panel (React app) at `http://localhost:5173` provides user management and richer statistics.

---

## Configuration Reference

### `~/.claude-quota-hook.json`

```json
{
  "server_url": "http://your-server:3000",
  "username": "alice",
  "token": "your-48-char-pat-here"
}
```

### `~/.claude-quota-queue.json`

Auto-managed retry queue. Holds reports that failed to upload. Processed on the next hook invocation.

---

## File Structure

```
claude-quota/
├── Cargo.toml                        # Rust 2024 edition, axum, tokio, rusqlite, rand
├── src/
│   ├── main.rs                       # axum server, CLI args (--port, --database), router
│   ├── db.rs                         # SQLite WAL mode, Mutex<Connection>, schema
│   ├── models.rs                     # Serde structs: UsageReport, UserRecord, AdminStats
│   ├── api.rs                        # Public API handlers: report, users, usage, summary, hourly
│   ├── admin.rs                      # Admin API: user CRUD, stats, token management
│   ├── auth.rs                       # PAT middleware (48-char tokens), require_auth, require_admin
│   └── dashboard.rs                  # Inline HTML dashboard with SVG quota gauges
├── admin/                            # Vite + React 19 admin UI
│   ├── package.json                  # Node 24+, React 19, Vite 7, TypeScript 5.9
│   ├── vite.config.ts                # ESNext target, dev proxy to :3000
│   ├── tsconfig.json                 # Strict, ESNext
│   └── src/
│       ├── pages/Login.tsx           # PAT login page
│       ├── pages/Stats.tsx           # Statistics dashboard with gauges
│       └── pages/Users.tsx          # User management (CRUD, token regeneration)
├── hook/
│   ├── claude-quota-hook.sh          # Claude Code Stop hook
│   ├── codex-hook.sh                 # OpenAI Codex CLI notify hook
│   └── antigravity-hook.sh          # Google Antigravity manual reporter
├── setup.sh                          # Interactive client setup script
├── AGENTS.md                         # AI agent setup guide
└── README.md
```

---

## Troubleshooting

| Issue | Check |
|-------|-------|
| Hook not firing | Verify the path in `~/.claude/settings.json` is absolute and correct |
| No data on dashboard | Verify `~/.claude-quota-hook.json` has the correct `server_url` and `token` |
| Connection refused | Ensure server is running: `curl http://server:3000/api/users` |
| 401 Unauthorized | Check `token` field in `~/.claude-quota-hook.json` matches a valid PAT |
| Queued reports | Inspect `~/.claude-quota-queue.json` for pending uploads |
| Duplicate 409s | Expected — the hook safely retries without double-counting |
| Admin token lost | Use `POST /api/admin/users/:id/regenerate-token` via another admin account |
| Admin panel blank | Ensure server is running on `:3000`; check browser console for CORS/proxy errors |
