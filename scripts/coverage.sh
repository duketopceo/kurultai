#!/usr/bin/env bash
# Local + CI coverage entrypoint (Phase 4 / #23).
# Default floor is 60% line coverage — Phase 4 gate. Override with COVERAGE_MIN.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

COVERAGE_MIN="${COVERAGE_MIN:-60}"
DO_FAIL=1
DO_HTML=0

for arg in "$@"; do
  case "$arg" in
    --no-fail) DO_FAIL=0 ;;
    --html) DO_HTML=1 ;;
    -h|--help)
      echo "Usage: $0 [--html] [--no-fail]"
      echo "  COVERAGE_MIN=${COVERAGE_MIN} (default 60)"
      exit 0
      ;;
    *)
      echo "Unknown arg: $arg" >&2
      exit 2
      ;;
  esac
done

FAIL_ARGS=()
if [[ "$DO_FAIL" -eq 1 ]]; then
  FAIL_ARGS=(--fail-under-lines "$COVERAGE_MIN")
fi

set +e
cargo llvm-cov nextest --locked --lcov --output-path lcov.info "${FAIL_ARGS[@]}"
status=$?
set -e

if [[ "$DO_HTML" -eq 1 ]]; then
  cargo llvm-cov report --html --output-dir coverage-html
fi

cargo llvm-cov report --summary-only
echo "Coverage gate: fail-under-lines=${COVERAGE_MIN} (disabled=$([[ $DO_FAIL -eq 0 ]] && echo yes || echo no))"
exit "$status"
