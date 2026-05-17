#!/bin/bash

# Tests for hardware-scan.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARDWARE_SCAN="$SCRIPT_DIR/../../scripts/hardware-scan.sh"

echo "Testing hardware-scan.sh..."

# Test 1: --env mode produces only KEY=VALUE lines
echo "  [1] --env mode produces valid shell assignments..."
OUTPUT=$(bash "$HARDWARE_SCAN" --env)
while IFS= read -r line; do
  if [[ -n "$line" && ! "$line" =~ ^[A-Za-z_][A-Za-z0-9_]*=.*$ ]]; then
    echo "    ❌ Invalid line: '$line'"
    exit 1
  fi
done <<< "$OUTPUT"
echo "    ✅ All lines are valid KEY=VALUE assignments"

# Test 2: All expected variables are present
echo "  [2] Expected variables are present..."
for var in FORCE_SMALL_MODELS PREFER_SMALL_MODELS USE_NETWORK_FETCH REDUCE_PARALLELISM INCREASE_TIMEOUTS DISABLE_CACHE AGGRESSIVE_CLEANUP CPU_CORES TOTAL_MEM STORAGE_AVAILABLE GPU_TYPE GPU_MEM GPU_COUNT NETWORK; do
  if ! grep -q "^${var}=" <<< "$OUTPUT"; then
    echo "    ❌ Missing variable: $var"
    exit 1
  fi
done
echo "    ✅ All expected variables present"

# Test 3: Numeric variables contain valid values
echo "  [3] Numeric variables contain valid values..."
for var in FORCE_SMALL_MODELS PREFER_SMALL_MODELS USE_NETWORK_FETCH REDUCE_PARALLELISM INCREASE_TIMEOUTS DISABLE_CACHE AGGRESSIVE_CLEANUP CPU_CORES GPU_COUNT; do
  VAL=$(grep "^${var}=" <<< "$OUTPUT" | cut -d= -f2)
  if ! [[ "$VAL" =~ ^[0-9]+$ ]]; then
    echo "    ❌ $var is not numeric: '$VAL'"
    exit 1
  fi
done
echo "    ✅ All numeric variables valid"

# Test 4: Sourcing works without errors
echo "  [4] --env output can be sourced safely..."
eval "$(bash "$HARDWARE_SCAN" --env)"
if [[ -z "${CPU_CORES:-}" ]]; then
  echo "    ❌ CPU_CORES not set after sourcing"
  exit 1
fi
if [[ -z "${TOTAL_MEM:-}" ]]; then
  echo "    ❌ TOTAL_MEM not set after sourcing"
  exit 1
fi
echo "    ✅ Output is source-safe"

# Test 5: Human mode produces different output
echo "  [5] Human mode produces readable output..."
HUMAN_OUTPUT=$(bash "$HARDWARE_SCAN")
if ! grep -q "Hardware Scan" <<< "$HUMAN_OUTPUT"; then
  echo "    ❌ Human mode missing expected header"
  exit 1
fi
echo "    ✅ Human mode produces readable output"

# Test 6: Values are sensible for this machine
echo "  [6] Values are sensible for this machine..."
if [[ "$CPU_CORES" -lt 1 ]]; then
  echo "    ❌ CPU_CORES is unreasonably low: $CPU_CORES"
  exit 1
fi
if [[ "$TOTAL_MEM" -lt 0 ]]; then
  echo "    ❌ TOTAL_MEM is negative: $TOTAL_MEM"
  exit 1
fi
echo "    ✅ Values are sensible"

echo ""
echo "✅ hardware-scan.sh tests passed"
