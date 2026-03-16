#!/bin/bash
# Claude Code Hook Script - Submit Usage Reports
#
# This script is intended to be called by Claude Code at the end of each session
# or periodically during long sessions to report token usage.
#
# Installation:
# 1. Copy this script to ~/.claude/hooks/
# 2. Make executable: chmod +x ~/.claude/hooks/submit-quota.sh
# 3. Configure Claude Code to call on session end
#
# Usage:
# submit-quota.sh <username> <session-id> <model> <input-tokens> <output-tokens>

set -euo pipefail

# Configuration
QUOTA_SERVER="${QUOTA_SERVER:-http://localhost:3000}"
QUOTA_API_ENDPOINT="${QUOTA_SERVER}/api/report"

# Arguments
USERNAME="${1:-unknown}"
SESSION_ID="${2:-$(uuidgen)}"
MODEL="${3:-claude-opus-4}"
INPUT_TOKENS="${4:-0}"
OUTPUT_TOKENS="${5:-0}"
CACHE_READ_TOKENS="${6:-0}"
CACHE_CREATION_TOKENS="${7:-0}"
MESSAGE_COUNT="${8:-0}"
TOOL_USE_COUNT="${9:-0}"

# Generate unique report ID
REPORT_ID="$(uuidgen)"
TIMESTAMP="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"

# Build JSON payload
PAYLOAD=$(cat <<PAYLOAD_EOF
{
  "username": "$USERNAME",
  "session_id": "$SESSION_ID",
  "report_id": "$REPORT_ID",
  "timestamp": "$TIMESTAMP",
  "model": "$MODEL",
  "input_tokens": $INPUT_TOKENS,
  "output_tokens": $OUTPUT_TOKENS,
  "cache_read_input_tokens": $CACHE_READ_TOKENS,
  "cache_creation_input_tokens": $CACHE_CREATION_TOKENS,
  "message_count": $MESSAGE_COUNT,
  "tool_use_count": $TOOL_USE_COUNT
}
PAYLOAD_EOF
)

# Submit report with retry logic
MAX_RETRIES=3
RETRY_DELAY=2

for attempt in $(seq 1 $MAX_RETRIES); do
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] Submitting quota report (attempt $attempt/$MAX_RETRIES)" >&2
    echo "[DEBUG] Endpoint: $QUOTA_API_ENDPOINT" >&2
    echo "[DEBUG] Payload: $PAYLOAD" >&2
    
    HTTP_STATUS=$(curl -s -w "%{http_code}" -o /tmp/quota-response.txt \
        -X POST "$QUOTA_API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "$PAYLOAD" 2>/dev/null || echo "000")
    
    RESPONSE_BODY=$(cat /tmp/quota-response.txt 2>/dev/null || echo "{}")
    rm -f /tmp/quota-response.txt
    
    echo "[DEBUG] Response Status: $HTTP_STATUS" >&2
    echo "[DEBUG] Response Body: $RESPONSE_BODY" >&2
    
    case "$HTTP_STATUS" in
        201)
            echo "[OK] Report submitted successfully (201 Created)" >&2
            echo "$RESPONSE_BODY"
            exit 0
            ;;
        409)
            echo "[INFO] Report already exists (409 Conflict) - duplicate submission" >&2
            echo "$RESPONSE_BODY"
            exit 0
            ;;
        200)
            echo "[OK] Report accepted (200 OK)" >&2
            echo "$RESPONSE_BODY"
            exit 0
            ;;
        *)
            if [ $attempt -lt $MAX_RETRIES ]; then
                echo "[WARN] Server error ($HTTP_STATUS) - retrying in ${RETRY_DELAY}s..." >&2
                sleep $RETRY_DELAY
            else
                echo "[ERROR] Failed to submit report after $MAX_RETRIES attempts" >&2
                echo "[ERROR] Last response: $HTTP_STATUS - $RESPONSE_BODY" >&2
                exit 1
            fi
            ;;
    esac
done

exit 1
