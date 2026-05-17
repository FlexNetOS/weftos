#!/bin/bash

# WeftOS DevOps Script Test Runner
# Runs all test-*.sh files in this directory

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== WeftOS DevOps Script Tests ==="
echo ""

FAILED=0

for test_file in "$SCRIPT_DIR"/test-*.sh; do
  if [[ -f "$test_file" ]]; then
    echo "--- Running $(basename "$test_file") ---"
    if bash "$test_file"; then
      echo "✅ $(basename "$test_file") PASSED"
    else
      echo "❌ $(basename "$test_file") FAILED"
      FAILED=1
    fi
    echo ""
  fi
done

if [[ $FAILED -eq 0 ]]; then
  echo "✅ All tests passed"
  exit 0
else
  echo "❌ Some tests failed"
  exit 1
fi
