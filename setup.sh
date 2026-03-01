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

# Check that hook scripts exist
for _script in "$HOOK_SCRIPT_CLAUDE" "$HOOK_SCRIPT_CODEX" "$HOOK_SCRIPT_ANTIGRAVITY"; do
  if [ ! -f "$_script" ]; then
    echo "ERROR: Hook script not found at: $_script"
    echo "Make sure you are running setup.sh from within the claude-quota repository."
    exit 1
  fi
done

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

read -rp "Personal Access Token (PAT from admin): " TOKEN
while [ -z "$TOKEN" ]; do
  echo "Token cannot be empty. Please obtain a PAT from the server admin panel."
  read -rp "Personal Access Token (PAT from admin): " TOKEN
done

# --- Write config file ---

echo ""
echo "Writing config to $CONFIG_FILE ..."
jq -n \
  --arg server_url "$SERVER_URL" \
  --arg username "$USERNAME" \
  --arg token "$TOKEN" \
  '{server_url: $server_url, username: $username, token: $token}' \
  > "$CONFIG_FILE"
echo "  Done."

# --- Ask which tools to configure ---

echo ""
echo "Which AI tools do you want to configure?"
echo "  1) Claude Code"
echo "  2) Codex CLI"
echo "  3) Antigravity"
echo "  4) All of the above"
read -rp "Selection [1-4, default: 4]: " TOOL_CHOICE
if [ -z "$TOOL_CHOICE" ]; then
  TOOL_CHOICE="4"
fi

CONFIGURE_CLAUDE=false
CONFIGURE_CODEX=false
CONFIGURE_ANTIGRAVITY=false

case "$TOOL_CHOICE" in
  1) CONFIGURE_CLAUDE=true ;;
  2) CONFIGURE_CODEX=true ;;
  3) CONFIGURE_ANTIGRAVITY=true ;;
  4) CONFIGURE_CLAUDE=true; CONFIGURE_CODEX=true; CONFIGURE_ANTIGRAVITY=true ;;
  *)
    echo "Invalid selection. Defaulting to all tools."
    CONFIGURE_CLAUDE=true; CONFIGURE_CODEX=true; CONFIGURE_ANTIGRAVITY=true
    ;;
esac

# --- Make all hook scripts executable ---

chmod +x "$HOOK_SCRIPT_CLAUDE"
chmod +x "$HOOK_SCRIPT_CODEX"
chmod +x "$HOOK_SCRIPT_ANTIGRAVITY"
echo "Hook scripts are executable."

# --- Configure Claude Code ---

if [ "$CONFIGURE_CLAUDE" = true ]; then
  echo ""
  echo "Configuring Stop hook for Claude Code in $SETTINGS_FILE ..."

  # Ensure the directory exists
  mkdir -p "$(dirname "$SETTINGS_FILE")"

  # Build the hook entry object we want to insert
  NEW_HOOK_ENTRY="$(jq -n \
    --arg cmd "$HOOK_SCRIPT_CLAUDE" \
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
      --arg cmd "$HOOK_SCRIPT_CLAUDE" \
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
fi

# --- Configure Codex CLI ---

if [ "$CONFIGURE_CODEX" = true ]; then
  echo ""
  echo "Configuring notify hook for Codex CLI in $CODEX_CONFIG ..."

  mkdir -p "$(dirname "$CODEX_CONFIG")"

  if grep -q '^notify' "$CODEX_CONFIG" 2>/dev/null; then
    echo "  'notify' entry already exists in $CODEX_CONFIG."
    echo "  To update it, edit the file manually and set:"
    echo "    notify = [\"$HOOK_SCRIPT_CODEX\"]"
  else
    printf '\nnotify = ["%s"]\n' "$HOOK_SCRIPT_CODEX" >> "$CODEX_CONFIG"
    echo "  Done."
  fi
fi

# --- Configure Antigravity ---

if [ "$CONFIGURE_ANTIGRAVITY" = true ]; then
  echo ""
  echo "Antigravity hook instructions:"
  echo "  Antigravity does not support automatic hooks."
  echo "  Run the hook manually after each session:"
  echo "    $HOOK_SCRIPT_ANTIGRAVITY"
  echo ""
  echo "  Or add a VS Code task to .vscode/tasks.json:"
  echo '    {'
  echo '      "label": "claude-quota: report usage",'
  echo '      "type": "shell",'
  echo "      \"command\": \"$HOOK_SCRIPT_ANTIGRAVITY\","
  echo '      "runOptions": { "runOn": "folderClose" }'
  echo '    }'
fi

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
echo "  Config file    : $CONFIG_FILE"
echo "  Server URL     : $SERVER_URL"
echo "  Username       : $USERNAME"
echo "  Token          : ${TOKEN:0:6}... (truncated)"
echo ""

if [ "$CONFIGURE_CLAUDE" = true ]; then
  echo "  Claude Code    : hook registered in $SETTINGS_FILE"
  echo "  Hook script    : $HOOK_SCRIPT_CLAUDE"
fi
if [ "$CONFIGURE_CODEX" = true ]; then
  echo "  Codex CLI      : notify set in $CODEX_CONFIG"
  echo "  Hook script    : $HOOK_SCRIPT_CODEX"
fi
if [ "$CONFIGURE_ANTIGRAVITY" = true ]; then
  echo "  Antigravity    : manual invocation required (see instructions above)"
  echo "  Hook script    : $HOOK_SCRIPT_ANTIGRAVITY"
fi

echo ""
echo "Quota usage will be reported to the server after each configured tool session."
echo "Restart any running AI tool instances for the hooks to take effect."
