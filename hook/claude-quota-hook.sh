#!/usr/bin/env bash
set -euo pipefail

# Config paths
CONFIG_FILE="$HOME/.claude-quota-hook.json"
STATE_FILE="$HOME/.claude-quota-state.json"
QUEUE_FILE="$HOME/.claude-quota-queue.json"

# Read stdin (hook input)
INPUT=$(cat)

SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // empty')
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // empty')

# Validate inputs
if [[ -z "$SESSION_ID" || -z "$TRANSCRIPT_PATH" ]]; then
    exit 0  # Nothing to do
fi

if [[ ! -f "$TRANSCRIPT_PATH" ]]; then
    exit 0
fi

# Read config
if [[ ! -f "$CONFIG_FILE" ]]; then
    exit 0  # Not configured
fi

SERVER_URL=$(jq -r '.server_url // empty' "$CONFIG_FILE")
USERNAME=$(jq -r '.username // empty' "$CONFIG_FILE")
TOKEN=$(jq -r '.token // empty' "$CONFIG_FILE")

if [[ -z "$SERVER_URL" || -z "$USERNAME" ]]; then
    exit 0
fi

# Initialize state file if needed
if [[ ! -f "$STATE_FILE" ]]; then
    echo '{"sessions":{}}' > "$STATE_FILE"
fi

# Initialize queue file if needed
if [[ ! -f "$QUEUE_FILE" ]]; then
    echo '[]' > "$QUEUE_FILE"
fi

# --- Retry queued reports ---
retry_queue() {
    local queue
    queue=$(cat "$QUEUE_FILE")
    local remaining="[]"
    local count
    count=$(echo "$queue" | jq 'length')

    for ((i=0; i<count; i++)); do
        local report
        report=$(echo "$queue" | jq ".[$i]")
        local http_code
        http_code=$(curl -s -o /dev/null -w '%{http_code}' \
            -X POST "${SERVER_URL}/api/report" \
            -H 'Content-Type: application/json' \
            -H "Authorization: Bearer ${TOKEN}" \
            -d "$report" \
            --connect-timeout 5 \
            --max-time 8 2>/dev/null) || http_code="000"

        if [[ "$http_code" != "201" && "$http_code" != "409" ]]; then
            remaining=$(echo "$remaining" | jq --argjson r "$report" '. + [$r]')
        fi
    done

    echo "$remaining" > "$QUEUE_FILE"
}

retry_queue

# --- Process new transcript data ---

# Get last reported line for this session
LAST_LINE=$(jq -r --arg sid "$SESSION_ID" '.sessions[$sid].last_line // 0' "$STATE_FILE")

# Count total lines in transcript
TOTAL_LINES=$(wc -l < "$TRANSCRIPT_PATH" | tr -d ' ')

if [[ "$TOTAL_LINES" -le "$LAST_LINE" ]]; then
    exit 0  # No new data
fi

# Extract new lines and aggregate token usage from assistant messages
START_LINE=$((LAST_LINE + 1))

# Parse new assistant entries and sum tokens
read -r INPUT_TOKENS OUTPUT_TOKENS CACHE_READ CACHE_CREATION MSG_COUNT TOOL_COUNT MODEL < <(
    tail -n "+${START_LINE}" "$TRANSCRIPT_PATH" | \
    jq -s '
        [.[] | select(.type == "assistant")] |
        {
            input_tokens: (map(.message.usage.input_tokens // 0) | add // 0),
            output_tokens: (map(.message.usage.output_tokens // 0) | add // 0),
            cache_read: (map(.message.usage.cache_read_input_tokens // 0) | add // 0),
            cache_creation: (map(.message.usage.cache_creation_input_tokens // 0) | add // 0),
            msg_count: length,
            tool_count: (map(.message.content // [] | [.[] | select(.type == "tool_use")] | length) | add // 0),
            model: (map(.message.model // empty) | last // "unknown")
        } |
        "\(.input_tokens) \(.output_tokens) \(.cache_read) \(.cache_creation) \(.msg_count) \(.tool_count) \(.model)"
    '
)

# Skip if no assistant messages found
if [[ "$MSG_COUNT" -eq 0 ]]; then
    # Still update state to avoid reprocessing
    jq --arg sid "$SESSION_ID" --argjson line "$TOTAL_LINES" \
        '.sessions[$sid] = {"last_line": $line, "last_report_time": (now | todate)}' \
        "$STATE_FILE" > "${STATE_FILE}.tmp" && mv "${STATE_FILE}.tmp" "$STATE_FILE"
    exit 0
fi

# Generate deterministic report ID
REPORT_ID=$(echo -n "${SESSION_ID}:${START_LINE}:${TOTAL_LINES}" | shasum -a 256 | cut -d' ' -f1)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# --- Capture usage percentages ---
PERCENT_5H="null"
PERCENT_7D="null"

if command -v claude &>/dev/null; then
    USAGE_OUTPUT=$(claude usage 2>/dev/null || true)
    if [[ -n "$USAGE_OUTPUT" ]]; then
        # Try to extract percentages from various output formats
        # Look for patterns like "42.3%" near "5-hour" or "5h"
        PERCENT_5H=$(echo "$USAGE_OUTPUT" | grep -i '5.[Hh]our\|5h' | grep -oE '[0-9]+\.?[0-9]*%' | head -1 | tr -d '%' || echo "null")
        PERCENT_7D=$(echo "$USAGE_OUTPUT" | grep -i '[Dd]aily\|7.[Dd]ay\|7d' | grep -oE '[0-9]+\.?[0-9]*%' | head -1 | tr -d '%' || echo "null")

        # Validate they're numbers
        [[ "$PERCENT_5H" =~ ^[0-9]+\.?[0-9]*$ ]] || PERCENT_5H="null"
        [[ "$PERCENT_7D" =~ ^[0-9]+\.?[0-9]*$ ]] || PERCENT_7D="null"
    fi
fi

# Build report JSON
REPORT=$(jq -n \
    --arg username "$USERNAME" \
    --arg session_id "$SESSION_ID" \
    --arg report_id "$REPORT_ID" \
    --arg timestamp "$TIMESTAMP" \
    --arg model "$MODEL" \
    --argjson input_tokens "$INPUT_TOKENS" \
    --argjson output_tokens "$OUTPUT_TOKENS" \
    --argjson cache_read "$CACHE_READ" \
    --argjson cache_creation "$CACHE_CREATION" \
    --argjson msg_count "$MSG_COUNT" \
    --argjson tool_count "$TOOL_COUNT" \
    --argjson pct_5h "$PERCENT_5H" \
    --argjson pct_7d "$PERCENT_7D" \
    '{
        username: $username,
        session_id: $session_id,
        report_id: $report_id,
        timestamp: $timestamp,
        model: $model,
        input_tokens: $input_tokens,
        output_tokens: $output_tokens,
        cache_read_input_tokens: $cache_read,
        cache_creation_input_tokens: $cache_creation,
        message_count: $msg_count,
        tool_use_count: $tool_count,
        usage_percent_5h: $pct_5h,
        usage_percent_7d: $pct_7d
    }')

# POST to server
HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' \
    -X POST "${SERVER_URL}/api/report" \
    -H 'Content-Type: application/json' \
    -H "Authorization: Bearer ${TOKEN}" \
    -d "$REPORT" \
    --connect-timeout 5 \
    --max-time 8 2>/dev/null) || HTTP_CODE="000"

if [[ "$HTTP_CODE" != "201" && "$HTTP_CODE" != "409" ]]; then
    # Queue for retry
    jq --argjson r "$REPORT" '. + [$r]' "$QUEUE_FILE" > "${QUEUE_FILE}.tmp" \
        && mv "${QUEUE_FILE}.tmp" "$QUEUE_FILE"
fi

# Update state
jq --arg sid "$SESSION_ID" --argjson line "$TOTAL_LINES" \
    '.sessions[$sid] = {"last_line": $line, "last_report_time": (now | todate)}' \
    "$STATE_FILE" > "${STATE_FILE}.tmp" && mv "${STATE_FILE}.tmp" "$STATE_FILE"

exit 0
