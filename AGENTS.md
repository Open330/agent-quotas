# Claude Quota Monitor — Agent Setup Guide

This guide is for AI agents (Claude Code, Cursor, etc.) to set up the quota monitoring hook on a user's machine autonomously.

## Prerequisites

Before starting, verify:
- `jq` is installed: `command -v jq` should return a path
- `curl` is installed: `command -v curl` should return a path
- A quota monitor server is running (ask user if not provided)

## Setup Steps

### Step 1: Gather Configuration from User

Ask the user for:
1. **Server URL**: The address where the quota monitor server is running (e.g., `http://192.168.1.100:3000` or `http://localhost:3000`)
2. **Username**: A display name for this machine on the dashboard (e.g., `MacBook Pro` or `dev-machine`)

If the user doesn't have a server running yet, direct them to the Server Setup section at the end.

### Step 2: Create Hook Configuration File

Write the configuration to `~/.claude-quota-hook.json`:

```json
{
  "server_url": "USER_PROVIDED_SERVER_URL",
  "username": "USER_PROVIDED_USERNAME"
}
```

**Example:**
```json
{
  "server_url": "http://192.168.1.100:3000",
  "username": "MacBook Pro"
}
```

### Step 3: Locate and Verify Hook Script

The hook script is located at: `REPO_PATH/hook/claude-quota-hook.sh`

Ensure it's executable:
```bash
chmod +x REPO_PATH/hook/claude-quota-hook.sh
```

### Step 4: Register Stop Hook in Claude Settings

Use `jq` to register the hook in `~/.claude/settings.json`. This ensures the hook runs when Claude Code or other agents stop:

```bash
HOOK_PATH="ABSOLUTE_PATH_TO_REPO/hook/claude-quota-hook.sh"
SETTINGS="$HOME/.claude/settings.json"

# Create settings file if it doesn't exist
[ -f "$SETTINGS" ] || echo '{}' > "$SETTINGS"

# Merge hook configuration (preserves existing settings)
jq --arg cmd "$HOOK_PATH" '
  .hooks //= {} |
  .hooks.Stop //= [] |
  (.hooks.Stop | length) as $len |
  if .hooks.Stop | map(select(.hooks[0].command == $cmd)) | length > 0
  then .
  else .hooks.Stop += [{"hooks": [{"type": "command", "command": $cmd, "timeout": 10000}]}]
  end
' "$SETTINGS" > "${SETTINGS}.tmp" && mv "${SETTINGS}.tmp" "$SETTINGS"

echo "Hook registered successfully"
```

**Note:** Replace `ABSOLUTE_PATH_TO_REPO` with the full path to the claude-quota repository (e.g., `/Users/hletrd/git/claude-quota`).

### Step 5: Verify Installation

Run these verification checks:

```bash
# 1. Check config file exists and is readable
echo "Checking config..."
cat ~/.claude-quota-hook.json

# 2. Check hook is registered in settings
echo "Checking hook registration..."
jq '.hooks.Stop' ~/.claude/settings.json

# 3. Test server connectivity
echo "Testing server connectivity..."
SERVER_URL=$(jq -r '.server_url' ~/.claude-quota-hook.json)
curl -s "${SERVER_URL}/api/users" > /dev/null && echo "✓ Server is reachable" || echo "✗ Server is not reachable"

# 4. Check hook script is executable
echo "Checking hook script..."
HOOK_PATH="ABSOLUTE_PATH_TO_REPO/hook/claude-quota-hook.sh"
[ -x "$HOOK_PATH" ] && echo "✓ Hook script is executable" || echo "✗ Hook script is not executable"
```

### Step 6: Test the Hook (Optional)

To manually test that the hook works:

```bash
# Run the hook script directly
ABSOLUTE_PATH_TO_REPO/hook/claude-quota-hook.sh

# Check if data was sent to the server
SERVER_URL=$(jq -r '.server_url' ~/.claude-quota-hook.json)
curl -s "${SERVER_URL}/api/users" | jq '.'
```

## Server Setup (If User Needs to Run Server)

If the user needs to run the quota monitor server locally or on another machine:

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

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "jq: command not found" | Install jq: `brew install jq` (macOS) or `apt-get install jq` (Linux) |
| Hook not firing | Verify hook path in `~/.claude/settings.json` is correct and absolute |
| No data on dashboard | Check `~/.claude-quota-hook.json` has correct `server_url` |
| "Connection refused" | Ensure server is running and reachable at the configured URL |
| Hook script permission denied | Run: `chmod +x REPO_PATH/hook/claude-quota-hook.sh` |
| jq merge fails | Check `~/.claude/settings.json` is valid JSON: `jq '.' ~/.claude/settings.json` |

## What Happens After Setup

Once configured:
1. Each time Claude Code or a similar agent stops, the hook script runs automatically
2. The hook reads `~/.claude-quota-hook.json` for server credentials
3. It collects system metrics (CPU, memory, disk) and token usage
4. Data is sent to the server at the configured URL
5. The dashboard displays aggregated usage per username/machine

## Configuration Reference

**Config file:** `~/.claude-quota-hook.json`

```json
{
  "server_url": "http://host:port",
  "username": "machine-name"
}
```

- `server_url`: HTTP(S) endpoint of the quota monitor server
- `username`: Display name for this client (used for grouping on dashboard)

**Hook registration:** `~/.claude/settings.json`

The Stop hook ensures the quota script runs when agents exit:
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

## Questions for the User

Before starting setup, clarify:

1. Do you have a quota monitor server already running? If yes, what's the URL?
2. If no server: Do you want to run it on this machine, or on another host (e.g., NAS, shared server)?
3. What display name would you like for this machine?
4. Are there any firewall or network restrictions between this machine and the server?
