#!/usr/bin/env bash
set -euo pipefail

CONFIG_FILE="$HOME/.claude-quota-hook.json"
STATE_FILE="$HOME/.claude-quota-state.json"
QUEUE_FILE="$HOME/.claude-quota-queue.json"

INPUT=$(cat)

# Read config
if [[ ! -f "$CONFIG_FILE" ]]; then exit 0; fi
SERVER_URL=$(jq -r '.server_url // empty' "$CONFIG_FILE")
USERNAME=$(jq -r '.username // empty' "$CONFIG_FILE")
TOKEN=$(jq -r '.token // empty' "$CONFIG_FILE")
if [[ -z "$SERVER_URL" || -z "$USERNAME" ]]; then exit 0; fi

# Initialize queue
if [[ ! -f "$QUEUE_FILE" ]]; then echo '[]' > "$QUEUE_FILE"; fi

# Retry queue (same as claude hook)
retry_queue() {
    local queue remaining count
    queue=$(cat "$QUEUE_FILE")
    remaining="[]"
    count=$(echo "$queue" | jq 'length')
    for ((i=0; i<count; i++)); do
        local report http_code
        report=$(echo "$queue" | jq ".[$i]")
        http_code=$(curl -s -o /dev/null -w '%{http_code}' \
            -X POST "${SERVER_URL}/api/report" \
            -H 'Content-Type: application/json' \
            -H "Authorization: Bearer ${TOKEN}" \
            -d "$report" --connect-timeout 5 --max-time 8 2>/dev/null) || http_code="000"
        if [[ "$http_code" != "201" && "$http_code" != "409" ]]; then
            remaining=$(echo "$remaining" | jq --argjson r "$report" '. + [$r]')
        fi
    done
    echo "$remaining" > "$QUEUE_FILE"
}
retry_queue

# Parse Codex notification payload
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // .id // "codex-session"')
INPUT_TOKENS=$(echo "$INPUT" | jq -r '.usage.input_tokens // .input_tokens // .prompt_tokens // 0')
OUTPUT_TOKENS=$(echo "$INPUT" | jq -r '.usage.output_tokens // .output_tokens // .completion_tokens // 0')
MODEL=$(echo "$INPUT" | jq -r '.model // "codex-unknown"')
MSG_COUNT=$(echo "$INPUT" | jq -r '.message_count // .turns // 1')
TOOL_COUNT=$(echo "$INPUT" | jq -r '.tool_use_count // .tool_calls // 0')

# Skip if no meaningful data
if [[ "$INPUT_TOKENS" == "0" && "$OUTPUT_TOKENS" == "0" ]]; then exit 0; fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
REPORT_ID=$(echo -n "codex:${SESSION_ID}:${TIMESTAMP}" | shasum -a 256 | cut -d' ' -f1)

REPORT=$(jq -n \
    --arg username "$USERNAME" \
    --arg session_id "$SESSION_ID" \
    --arg report_id "$REPORT_ID" \
    --arg timestamp "$TIMESTAMP" \
    --arg model "$MODEL" \
    --argjson input_tokens "$INPUT_TOKENS" \
    --argjson output_tokens "$OUTPUT_TOKENS" \
    --argjson msg_count "$MSG_COUNT" \
    --argjson tool_count "$TOOL_COUNT" \
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
        message_count: $msg_count,
        tool_use_count: $tool_count
    }')

HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' \
    -X POST "${SERVER_URL}/api/report" \
    -H 'Content-Type: application/json' \
    -H "Authorization: Bearer ${TOKEN}" \
    -d "$REPORT" --connect-timeout 5 --max-time 8 2>/dev/null) || HTTP_CODE="000"

if [[ "$HTTP_CODE" != "201" && "$HTTP_CODE" != "409" ]]; then
    jq --argjson r "$REPORT" '. + [$r]' "$QUEUE_FILE" > "${QUEUE_FILE}.tmp" \
        && mv "${QUEUE_FILE}.tmp" "$QUEUE_FILE"
fi

exit 0
