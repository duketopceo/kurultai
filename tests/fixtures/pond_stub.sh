#!/usr/bin/env bash
# Fake `pond` binary for Phase 4 connector fixture tests.
# Emits one NDJSON message when invoked as: pond sql --format ndjson ...
set -euo pipefail

# Ignore args; always return a searchable fixture line.
cat <<'EOF'
{"session_id":"s1","message_id":"s1:fixture","timestamp":"2026-07-25T12:00:00Z","role":"user","source_agent":"fixture","project":"/tmp","search_text":"KNOWN_POND_PHRASE_77 pond connector e2e","content":null}
EOF
