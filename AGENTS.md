# Claude Quota Monitor — Agent Setup Guide

This guide is for AI agents (Claude Code, Codex CLI, Antigravity, etc.) to set up the quota monitoring hook on a user's machine autonomously.

## Prerequisites

Before starting, verify:
- `jq` is installed: `command -v jq` should return a path
- `curl` is installed: `command -v curl` should return a path
- A quota monitor server is running (ask user if not provided)
- The user has a **Personal Access Token (PAT)** issued by the server admin

## Setup Steps

### Step 1: Gather Configuration from User

Ask the user for:
1. **Server URL**: The address where the quota monitor server is running (e.g., `http://192.168.1.100:3000` or `http://localhost:3000`)
2. **Username**: A display name for this machine on the dashboard (e.g., `MacBook Pro` or `dev-machine`)
3. **Token**: A Personal Access Token (PAT) obtained from the server admin panel. The user must request this from their admin.

If the user doesn't have a server running yet, direct them to the Server Setup section at the end.

If the user doesn't have a PAT yet, direct them to request one from the admin via the admin panel (`/admin`).

### Step 2: Create Hook Configuration File

Write the configuration to `~/.claude-quota-hook.json`:

```json
{
  "server_url": "USER_PROVIDED_SERVER_URL",
  "username": "USER_PROVIDED_USERNAME",
  "token": "PAT_FROM_ADMIN"
}
```

**Example:**
```json
{
  "server_url": "http://192.168.1.100:3000",
  "username": "alice",
  "token": "PAT_abc123xyz"
}
```

All three fields are required. The `token` authenticates this client with the server and must be a valid PAT issued by the admin.

### Step 3: Locate and Verify Hook Scripts

Hook scripts are located in `REPO_PATH/hook/`:

| Tool | Script |
|------|--------|
| Claude Code | `hook/claude-quota-hook.sh` |
| Codex CLI | `hook/codex-hook.sh` |
| Antigravity | `hook/antigravity-hook.sh` |

Ensure all scripts are executable:
```bash
chmod +x REPO_PATH/hook/claude-quota-hook.sh
chmod +x REPO_PATH/hook/codex-hook.sh
chmod +x REPO_PATH/hook/antigravity-hook.sh
```

### Step 4: Register Hooks for Each Tool

Register the hook for every AI tool the user has installed. Choose the relevant section(s) below.

---

#### Claude Code

Use `jq` to register the Stop hook in `~/.claude/settings.json`. This runs after every Claude Code session:

```bash
HOOK_PATH="ABSOLUTE_PATH_TO_REPO/hook/claude-quota-hook.sh"
SETTINGS="$HOME/.claude/settings.json"

# Create settings file if it doesn't exist
[ -f "$SETTINGS" ] || echo '{}' > "$SETTINGS"

# Merge hook configuration (preserves existing settings)
jq --arg cmd "$HOOK_PATH" '
  .hooks //= {} |
  .hooks.Stop //= [] |
  if .hooks.Stop | map(select(.hooks[0].command == $cmd)) | length > 0
  then .
  else .hooks.Stop += [{"hooks": [{"type": "command", "command": $cmd, "timeout": 10000}]}]
  end
' "$SETTINGS" > "${SETTINGS}.tmp" && mv "${SETTINGS}.tmp" "$SETTINGS"

echo "Claude Code hook registered"
```

---

#### Codex CLI

Add a `notify` command to `~/.codex/config.toml`. This runs after each Codex session:

```bash
HOOK_PATH="ABSOLUTE_PATH_TO_REPO/hook/codex-hook.sh"
CODEX_CONFIG="$HOME/.codex/config.toml"

mkdir -p "$(dirname "$CODEX_CONFIG")"

if grep -q '^notify' "$CODEX_CONFIG" 2>/dev/null; then
  echo "Codex notify already set — update manually if needed."
else
  echo "" >> "$CODEX_CONFIG"
  echo "notify = [\"$HOOK_PATH\"]" >> "$CODEX_CONFIG"
  echo "Codex hook registered"
fi
```

The resulting config.toml entry looks like:
```toml
notify = ["/absolute/path/to/codex-hook.sh"]
```

---

#### Antigravity

Antigravity does not have a built-in hook mechanism. Register the hook manually using one of these approaches:

**Option A — VS Code Task** (recommended if using VS Code):

Add to `.vscode/tasks.json`:
```json
{
  "label": "claude-quota: report usage",
  "type": "shell",
  "command": "ABSOLUTE_PATH_TO_REPO/hook/antigravity-hook.sh",
  "runOptions": { "runOn": "folderClose" }
}
```

**Option B — Run manually**:
```bash
ABSOLUTE_PATH_TO_REPO/hook/antigravity-hook.sh
```

Inform the user that Antigravity integration requires manual invocation or a VS Code task, as there is no automatic hook point.

---

### Step 5: Verify Installation

Run these verification checks:

```bash
# 1. Check config file exists and has all required fields
echo "Checking config..."
jq '{server_url, username, token}' ~/.claude-quota-hook.json

# 2. Check hook is registered in Claude settings (if using Claude Code)
echo "Checking Claude Code hook registration..."
jq '.hooks.Stop' ~/.claude/settings.json

# 3. Check Codex config (if using Codex)
echo "Checking Codex config..."
grep 'notify' ~/.codex/config.toml 2>/dev/null || echo "(not configured)"

# 4. Test server connectivity
echo "Testing server connectivity..."
SERVER_URL=$(jq -r '.server_url' ~/.claude-quota-hook.json)
curl -s "${SERVER_URL}/api/users" > /dev/null && echo "Server is reachable" || echo "Server is not reachable"

# 5. Check hook scripts are executable
echo "Checking hook scripts..."
for script in claude-quota-hook.sh codex-hook.sh antigravity-hook.sh; do
  HOOK_PATH="ABSOLUTE_PATH_TO_REPO/hook/$script"
  [ -x "$HOOK_PATH" ] && echo "  OK: $script" || echo "  MISSING/NOT EXECUTABLE: $script"
done
```

### Step 6: Test the Hook (Optional)

To manually test that the hook works:

```bash
# Run the Claude Code hook directly
ABSOLUTE_PATH_TO_REPO/hook/claude-quota-hook.sh

# Check if data was sent to the server
SERVER_URL=$(jq -r '.server_url' ~/.claude-quota-hook.json)
curl -s "${SERVER_URL}/api/users" | jq '.'
```

## Server Setup

### Main Server

If the user needs to run the quota monitor server:

```bash
# Navigate to repo
cd REPO_PATH

# Build release binary
cargo build --release

# Run server (default port 3000, database auto-creates)
./target/release/claude-quota --port 3000 --database quota.db

# Or specify different port
./target/release/claude-quota --port 8080 --database quota.db
```

The server will:
- Start listening on the specified port
- Create a SQLite database if it doesn't exist
- Be accessible at `http://localhost:PORT`

### Admin Panel

The admin panel is a separate Next.js app used to manage users and issue PATs.

```bash
# Navigate to admin directory
cd REPO_PATH/admin

# Install dependencies
npm install

# Start development server (default port 3001)
npm run dev
```

The admin panel will be available at `http://localhost:3001` (or whichever port Next.js assigns).

From the admin panel, admins can:
- View all registered users and their quota usage
- Issue Personal Access Tokens (PATs) for client machines
- Revoke tokens
- Manage user accounts

**Users must obtain a PAT from the admin before they can configure a client machine.**

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "jq: command not found" | Install jq: `brew install jq` (macOS) or `apt-get install jq` (Linux) |
| Hook not firing (Claude Code) | Verify hook path in `~/.claude/settings.json` is correct and absolute |
| Hook not firing (Codex) | Verify `notify` entry in `~/.codex/config.toml` has correct absolute path |
| No data on dashboard | Check `~/.claude-quota-hook.json` has correct `server_url` and `token` |
| "Connection refused" | Ensure server is running and reachable at the configured URL |
| "Unauthorized" / 401 error | Token is invalid or expired — request a new PAT from the admin |
| Hook script permission denied | Run: `chmod +x REPO_PATH/hook/*.sh` |
| jq merge fails | Check `~/.claude/settings.json` is valid JSON: `jq '.' ~/.claude/settings.json` |
| Missing token field | Re-run setup and provide the PAT when prompted |

## What Happens After Setup

Once configured:
1. Each time a supported tool (Claude Code, Codex, Antigravity) finishes a session, the hook script runs automatically
2. The hook reads `~/.claude-quota-hook.json` for server credentials and the PAT token
3. It collects system metrics (CPU, memory, disk) and token usage
4. Data is sent to the server at the configured URL, authenticated with the PAT
5. The dashboard displays aggregated usage per username/machine

## Configuration Reference

**Config file:** `~/.claude-quota-hook.json`

```json
{
  "server_url": "http://host:port",
  "username": "machine-name",
  "token": "PAT_issued_by_admin"
}
```

- `server_url`: HTTP(S) endpoint of the quota monitor server
- `username`: Display name for this client (used for grouping on dashboard)
- `token`: Personal Access Token issued by the admin — required for authentication

**Claude Code hook registration:** `~/.claude/settings.json`

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/absolute/path/to/claude-quota-hook.sh",
            "timeout": 10000
          }
        ]
      }
    ]
  }
}
```

**Codex CLI hook registration:** `~/.codex/config.toml`

```toml
notify = ["/absolute/path/to/codex-hook.sh"]
```

## Questions for the User

Before starting setup, clarify:

1. Do you have a quota monitor server already running? If yes, what's the URL?
2. If no server: Do you want to run it on this machine, or on another host (e.g., NAS, shared server)?
3. What display name would you like for this machine?
4. Do you have a Personal Access Token (PAT) from the admin? If not, the admin must issue one via the admin panel.
5. Which AI tools do you use on this machine? (Claude Code / Codex CLI / Antigravity / multiple)
6. Are there any firewall or network restrictions between this machine and the server?
