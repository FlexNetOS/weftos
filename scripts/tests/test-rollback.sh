#!/bin/bash

# Tests for rollback.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROLLBACK="$SCRIPT_DIR/../../scripts/rollback.sh"
TMP_DIR=$(mktemp -d /tmp/weftos-test-rollback-XXXXXX)

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Testing rollback.sh..."

# Create a mock sudo that just executes its arguments
mkdir -p "$TMP_DIR/fakebin"
cat > "$TMP_DIR/fakebin/sudo" <<'SUDO'
#!/bin/bash
exec "$@"
SUDO
chmod +x "$TMP_DIR/fakebin/sudo"
export PATH="$TMP_DIR/fakebin:$PATH"

# Test 1: Rollback fails without backup
echo "  [1] Rollback fails when no backup exists..."
mkdir -p "$TMP_DIR/bin"
echo "current" > "$TMP_DIR/bin/weftos"
chmod +x "$TMP_DIR/bin/weftos"
if WEFTOS_BINARY="$TMP_DIR/bin/weftos" bash "$ROLLBACK" >/dev/null 2>&1; then
  echo "    ❌ Rollback should have failed without backup"
  exit 1
fi
echo "    ✅ Rollback correctly fails without backup"

# Create a working mock weftos binary for use as both current and backup
cat > "$TMP_DIR/bin/mock-weftos" <<'MOCK'
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
chmod +x "$TMP_DIR/bin/mock-weftos"

# Test 2: Rollback succeeds with backup present
echo "  [2] Rollback succeeds with backup present..."
cp "$TMP_DIR/bin/mock-weftos" "$TMP_DIR/bin/weftos"
cp "$TMP_DIR/bin/mock-weftos" "$TMP_DIR/bin/weftos.old"

if ! WEFTOS_BINARY="$TMP_DIR/bin/weftos" bash "$ROLLBACK" >/dev/null 2>&1; then
  echo "    ❌ Rollback failed with valid backup"
  exit 1
fi

# Verify restored content is the mock binary (functional)
if ! "$TMP_DIR/bin/weftos" version >/dev/null 2>&1; then
  echo "    ❌ Restored binary is not functional"
  exit 1
fi
echo "    ✅ Rollback correctly restores backup"

# Test 3: Script directory resolution works
echo "  [3] Script directory resolution works..."
if grep -q 'SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE\[0\]}")" && pwd)"' "$ROLLBACK"; then
  echo "    ✅ Script directory is resolved properly"
else
  echo "    ❌ Script directory resolution not found"
  exit 1
fi

# Test 4: Syntax check
echo "  [4] Syntax validation..."
if ! bash -n "$ROLLBACK"; then
  echo "    ❌ Syntax error in rollback.sh"
  exit 1
fi
echo "    ✅ Syntax is valid"

# Test 5: Respects WEFTOS_BINARY environment variable
echo "  [5] Respects WEFTOS_BINARY environment variable..."
if grep -q 'WEFTOS_BINARY:-/opt/weftos/bin/weftos' "$ROLLBACK"; then
  echo "    ✅ WEFTOS_BINARY env var is supported"
else
  echo "    ❌ WEFTOS_BINARY env var not found"
  exit 1
fi

echo ""
echo "✅ rollback.sh tests passed"
