#!/bin/bash

# Tests for health-check.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HEALTH_CHECK="$SCRIPT_DIR/../../scripts/health-check.sh"
TMP_DIR=$(mktemp -d /tmp/weftos-test-health-XXXXXX)

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Testing health-check.sh..."

# Create a mock weftos binary
cat > "$TMP_DIR/weftos" <<'MOCK'
#!/bin/bash
case "${1:-}" in
  version) echo "weftos 0.6.19" ;;
  init)
    mkdir -p "${2:-.}/.weftos"
    touch "${2:-.}/weave.toml"
    echo "Initialized"
    ;;
  status)
    if [[ -f "weave.toml" ]]; then
      echo "WeftOS initialized in current directory"
    else
      echo "WeftOS not initialized. Run: weftos init"
      exit 1
    fi
    ;;
  *) exit 1 ;;
esac
MOCK
chmod +x "$TMP_DIR/weftos"

# Test 1: Health check passes with mock binary
echo "  [1] Health check passes with valid binary..."
if ! WEFTOS_BINARY="$TMP_DIR/weftos" bash "$HEALTH_CHECK" >/dev/null 2>&1; then
  echo "    ❌ Health check failed with mock binary"
  exit 1
fi
echo "    ✅ Health check passes with valid binary"

# Test 2: Health check fails with missing binary
echo "  [2] Health check fails with missing binary..."
if WEFTOS_BINARY="$TMP_DIR/nonexistent" bash "$HEALTH_CHECK" >/dev/null 2>&1; then
  echo "    ❌ Health check should have failed with missing binary"
  exit 1
fi
echo "    ✅ Health check fails correctly with missing binary"

# Test 3: Health check fails with broken binary (version command fails)
echo "  [3] Health check fails with broken binary..."
cat > "$TMP_DIR/broken-weftos" <<'BROKEN'
#!/bin/bash
exit 1
BROKEN
chmod +x "$TMP_DIR/broken-weftos"
if WEFTOS_BINARY="$TMP_DIR/broken-weftos" bash "$HEALTH_CHECK" >/dev/null 2>&1; then
  echo "    ❌ Health check should have failed with broken binary"
  exit 1
fi
echo "    ✅ Health check fails correctly with broken binary"

# Test 4: Syntax check
echo "  [4] Syntax validation..."
if ! bash -n "$HEALTH_CHECK"; then
  echo "    ❌ Syntax error in health-check.sh"
  exit 1
fi
echo "    ✅ Syntax is valid"

# Test 5: Default binary path is set
echo "  [5] Default binary path is set..."
if grep -q 'WEFTOS_BINARY:-/opt/weftos/bin/weftos' "$HEALTH_CHECK"; then
  echo "    ✅ Default binary path is /opt/weftos/bin/weftos"
else
  echo "    ❌ Default binary path not found"
  exit 1
fi

echo ""
echo "✅ health-check.sh tests passed"
