#!/usr/bin/env bash
set -euo pipefail

# Claude Quota Monitor - Setup Script
# Sets up the client hook on this machine.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK_SCRIPT_CLAUDE="$SCRIPT_DIR/hook/claude-quota-hook.sh"
HOOK_SCRIPT_CODEX="$SCRIPT_DIR/hook/codex-hook.sh"
HOOK_SCRIPT_ANTIGRAVITY="$SCRIPT_DIR/hook/antigravity-hook.sh"
CONFIG_FILE="$HOME/.claude-quota-hook.json"
SETTINGS_FILE="$HOME/.claude/settings.json"
CODEX_CONFIG="$HOME/.codex/config.toml"

echo "Claude Quota Monitor - Setup"
echo "============================"
echo ""

# Check for required dependencies
if ! command -v jq &>/dev/null; then
  echo "ERROR: 'jq' is required but not installed."
  echo "  macOS: brew install jq"
  echo "  Ubuntu/Debian: sudo apt-get install jq"
  echo "  Fedora/RHEL: sudo dnf install jq"
  exit 1
fi

# Check that hook script exists
if [ ! -f "$HOOK_SCRIPT" ]; then
  echo "ERROR: Hook script not found at: $HOOK_SCRIPT"
  echo "Make sure you are running setup.sh from within the claude-quota repository."
  exit 1
fi

# --- Prompt for configuration ---

read -rp "Server URL [default: http://localhost:3000]: " SERVER_URL
if [ -z "$SERVER_URL" ]; then
  SERVER_URL="http://localhost:3000"
fi

read -rp "Username: " USERNAME
while [ -z "$USERNAME" ]; do
  echo "Username cannot be empty."
  read -rp "Username: " USERNAME
done

# --- Write config file ---

echo ""
echo "Writing config to $CONFIG_FILE ..."
jq -n \
  --arg server_url "$SERVER_URL" \
  --arg username "$USERNAME" \
  '{server_url: $server_url, username: $username}' \
  > "$CONFIG_FILE"
echo "  Done."

# --- Update ~/.claude/settings.json ---

echo "Configuring Stop hook in $SETTINGS_FILE ..."

# Ensure the directory exists
mkdir -p "$(dirname "$SETTINGS_FILE")"

# Build the hook entry object we want to insert
NEW_HOOK_ENTRY="$(jq -n \
  --arg cmd "$HOOK_SCRIPT" \
  '{
    hooks: [
      {
        type: "command",
        command: $cmd,
        timeout: 10000
      }
    ]
  }')"

if [ ! -f "$SETTINGS_FILE" ]; then
  # File does not exist — create it from scratch
  echo "  settings.json not found, creating ..."
  jq -n \
    --argjson entry "$NEW_HOOK_ENTRY" \
    '{hooks: {Stop: [$entry]}}' \
    > "$SETTINGS_FILE"
else
  # File exists — work with its current contents
  CURRENT="$(cat "$SETTINGS_FILE")"

  # Validate it is valid JSON
  if ! echo "$CURRENT" | jq empty 2>/dev/null; then
    echo "ERROR: $SETTINGS_FILE exists but is not valid JSON. Please fix it manually."
    exit 1
  fi

  # Check whether this exact command is already registered (idempotency)
  ALREADY_EXISTS="$(echo "$CURRENT" | jq \
    --arg cmd "$HOOK_SCRIPT" \
    '[.hooks.Stop[]?.hooks[]? | select(.type == "command" and .command == $cmd)] | length > 0')"

  if [ "$ALREADY_EXISTS" = "true" ]; then
    echo "  Hook already registered — skipping (idempotent)."
  else
    # Merge: if .hooks.Stop exists, append; otherwise build the structure
    HAS_STOP="$(echo "$CURRENT" | jq '.hooks.Stop != null')"

    if [ "$HAS_STOP" = "true" ]; then
      # Append to existing Stop array
      echo "  Appending to existing Stop hooks ..."
      echo "$CURRENT" | jq \
        --argjson entry "$NEW_HOOK_ENTRY" \
        '.hooks.Stop += [$entry]' \
        > "$SETTINGS_FILE"
    else
      HAS_HOOKS="$(echo "$CURRENT" | jq '.hooks != null')"
      if [ "$HAS_HOOKS" = "true" ]; then
        # hooks key exists but no Stop key
        echo "  Adding Stop hook to existing hooks section ..."
        echo "$CURRENT" | jq \
          --argjson entry "$NEW_HOOK_ENTRY" \
          '.hooks.Stop = [$entry]' \
          > "$SETTINGS_FILE"
      else
        # No hooks key at all — add it
        echo "  Adding hooks section ..."
        echo "$CURRENT" | jq \
          --argjson entry "$NEW_HOOK_ENTRY" \
          '.hooks = {Stop: [$entry]}' \
          > "$SETTINGS_FILE"
      fi
    fi
  fi
fi

echo "  Done."

# --- Make hook script executable ---

chmod +x "$HOOK_SCRIPT"
echo "Hook script is executable: $HOOK_SCRIPT"

# --- Optionally build the server ---

echo ""
if [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
  read -rp "Cargo.toml found. Build the server with 'cargo build --release'? [y/N]: " BUILD_SERVER
  if [[ "$BUILD_SERVER" =~ ^[Yy]$ ]]; then
    echo "Building server ..."
    (cd "$SCRIPT_DIR" && cargo build --release)
    echo "Build complete. Binary available at: $SCRIPT_DIR/target/release/"
  else
    echo "Skipping server build."
  fi
fi

# --- Summary ---

echo ""
echo "============================"
echo "Setup complete!"
echo ""
echo "  Config file   : $CONFIG_FILE"
echo "  Server URL    : $SERVER_URL"
echo "  Username      : $USERNAME"
echo "  Hook script   : $HOOK_SCRIPT"
echo "  Claude settings: $SETTINGS_FILE"
echo ""
echo "The Stop hook will report quota usage to the server after each Claude session."
echo "Start (or restart) your Claude instance for the hook to take effect."
