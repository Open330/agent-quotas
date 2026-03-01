#!/usr/bin/env bash
set -euo pipefail

# Google Antigravity usage reporter
# Run manually, via VS Code task, or periodically
# Usage: ./antigravity-hook.sh [--input-tokens N] [--output-tokens N] [--model NAME] [--session-id ID]

CONFIG_FILE="$HOME/.claude-quota-hook.json"
QUEUE_FILE="$HOME/.claude-quota-queue.json"

# Defaults
INPUT_TOKENS=0
OUTPUT_TOKENS=0
MODEL="antigravity-gemini"
SESSION_ID="antigravity-$(date +%s)"

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --input-tokens) INPUT_TOKENS="$2"; shift 2 ;;
        --output-tokens) OUTPUT_TOKENS="$2"; shift 2 ;;
        --model) MODEL="$2"; shift 2 ;;
        --session-id) SESSION_ID="$2"; shift 2 ;;
        *) shift ;;
    esac
done

# Read config
if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "Error: Config not found at $CONFIG_FILE"
    echo "Run setup.sh first or create the config manually."
    exit 1
fi
SERVER_URL=$(jq -r '.server_url // empty' "$CONFIG_FILE")
USERNAME=$(jq -r '.username // empty' "$CONFIG_FILE")
TOKEN=$(jq -r '.token // empty' "$CONFIG_FILE")
if [[ -z "$SERVER_URL" || -z "$USERNAME" ]]; then
    echo "Error: server_url and username required in $CONFIG_FILE"
    exit 1
fi

# Initialize queue
if [[ ! -f "$QUEUE_FILE" ]]; then echo '[]' > "$QUEUE_FILE"; fi

# Try to auto-detect Antigravity usage from its data directory
AG_DATA_DIR="$HOME/.antigravity"
if [[ -d "$AG_DATA_DIR" && "$INPUT_TOKENS" == "0" && "$OUTPUT_TOKENS" == "0" ]]; then
    # Look for recent session data files
    LATEST_SESSION=$(find "$AG_DATA_DIR" -name "*.json" -newer "$CONFIG_FILE" 2>/dev/null | head -1)
    if [[ -n "$LATEST_SESSION" ]]; then
        INPUT_TOKENS=$(jq -r '.usage.input_tokens // .prompt_tokens // 0' "$LATEST_SESSION" 2>/dev/null || echo 0)
        OUTPUT_TOKENS=$(jq -r '.usage.output_tokens // .completion_tokens // 0' "$LATEST_SESSION" 2>/dev/null || echo 0)
        MODEL=$(jq -r '.model // "antigravity-unknown"' "$LATEST_SESSION" 2>/dev/null || echo "antigravity-unknown")
        SESSION_ID=$(jq -r '.session_id // .id // "ag-session"' "$LATEST_SESSION" 2>/dev/null || echo "ag-session")
    fi
fi

if [[ "$INPUT_TOKENS" == "0" && "$OUTPUT_TOKENS" == "0" ]]; then
    echo "No usage data found. Use --input-tokens and --output-tokens to report manually."
    echo "Example: $0 --input-tokens 5000 --output-tokens 1000 --model gemini-3-pro"
    exit 0
fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
REPORT_ID=$(echo -n "ag:${SESSION_ID}:${TIMESTAMP}" | shasum -a 256 | cut -d' ' -f1)

REPORT=$(jq -n \
    --arg username "$USERNAME" \
    --arg session_id "$SESSION_ID" \
    --arg report_id "$REPORT_ID" \
    --arg timestamp "$TIMESTAMP" \
    --arg model "$MODEL" \
    --argjson input_tokens "$INPUT_TOKENS" \
    --argjson output_tokens "$OUTPUT_TOKENS" \
    '{
        username: $username,
        session_id: $session_id,
        report_id: $report_id,
        timestamp: $timestamp,
        model: $model,
        input_tokens: $input_tokens,
        output_tokens: $output_tokens,
        cache_read_input_tokens: 0,
        cache_creation_input_tokens: 0,
        message_count: 1,
        tool_use_count: 0
    }')

echo "Reporting: ${INPUT_TOKENS} input + ${OUTPUT_TOKENS} output tokens (${MODEL})"

HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' \
    -X POST "${SERVER_URL}/api/report" \
    -H 'Content-Type: application/json' \
    -H "Authorization: Bearer ${TOKEN}" \
    -d "$REPORT" --connect-timeout 5 --max-time 8 2>/dev/null) || HTTP_CODE="000"

if [[ "$HTTP_CODE" == "201" ]]; then
    echo "Success: Report submitted."
elif [[ "$HTTP_CODE" == "409" ]]; then
    echo "Info: Duplicate report (already recorded)."
else
    echo "Warning: Server returned $HTTP_CODE. Queuing for retry."
    jq --argjson r "$REPORT" '. + [$r]' "$QUEUE_FILE" > "${QUEUE_FILE}.tmp" \
        && mv "${QUEUE_FILE}.tmp" "$QUEUE_FILE"
fi

exit 0
