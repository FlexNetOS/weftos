#!/bin/bash

# WeftOS Health Check Script
# Validates: Binary integrity, version, basic functionality, systemd status (if present)
# Generated from: ai-devops-fork-deploy.prompt.md
#
# The weftos CLI supports: init, boot, status, version.
# This health check verifies the deployed binary is functional and responsive.

set -u

BINARY_PATH="${WEFTOS_BINARY:-/opt/weftos/bin/weftos}"
HEALTH_CHECK_TIMEOUT=${HEALTH_CHECK_TIMEOUT:-60}
FAILED=0

echo "=== WeftOS Health Check Suite ==="
echo "Binary: $BINARY_PATH"
echo "Timeout: ${HEALTH_CHECK_TIMEOUT}s"
echo "Timestamp: $(date)"
echo ""

# Check 1: Binary exists and is executable
echo "[1/4] Binary Integrity..."
if [[ -x "$BINARY_PATH" ]]; then
  echo "✅ Binary found and executable: $BINARY_PATH"
else
  echo "❌ Binary not found or not executable: $BINARY_PATH"
  FAILED=1
fi

# Check 2: Version check
echo "[2/4] Version Check..."
if [[ -x "$BINARY_PATH" ]]; then
  VERSION_OUTPUT=$(timeout "$HEALTH_CHECK_TIMEOUT" "$BINARY_PATH" version 2>/dev/null || echo "")
  if [[ -n "$VERSION_OUTPUT" ]]; then
    echo "✅ Version responds: $VERSION_OUTPUT"
  else
    echo "❌ Version check failed or timed out"
    FAILED=1
  fi
else
  echo "⚠️  Skipping version check (binary not available)"
fi

# Check 3: Functional test (init + status in temp directory)
echo "[3/4] Functional Test..."
if [[ -x "$BINARY_PATH" ]]; then
  TEMP_DIR=$(mktemp -d /tmp/weftos-health-check-XXXXXX)
  if (
    cd "$TEMP_DIR" && \
    timeout "$HEALTH_CHECK_TIMEOUT" "$BINARY_PATH" init . >/dev/null 2>&1 && \
    timeout "$HEALTH_CHECK_TIMEOUT" "$BINARY_PATH" status >/dev/null 2>&1
  ); then
    echo "✅ Functional test passed (init + status in temp directory)"
  else
    echo "❌ Functional test failed (init + status)"
    FAILED=1
  fi
  rm -rf "$TEMP_DIR"
else
  echo "⚠️  Skipping functional test (binary not available)"
fi

# Check 4: systemd service status (optional, only if service exists)
echo "[4/4] Service Status..."
if command -v systemctl >/dev/null 2>&1; then
  SERVICE="weftos"
  if systemctl list-unit-files --type=service 2>/dev/null | grep -q "^${SERVICE}.service"; then
    if systemctl is-active "$SERVICE" >/dev/null 2>&1; then
      echo "✅ systemd service '$SERVICE' is active"
    else
      echo "⚠️  systemd service '$SERVICE' exists but is not active"
      # Non-critical: weftos is primarily a project-local tool, not a system daemon
    fi
  else
    echo "ℹ️  No systemd service '$SERVICE' configured (optional for project-local deployments)"
  fi
else
  echo "ℹ️  systemctl not available (skipping service check)"
fi

# Summary
echo ""
echo "=== Health Check Summary ==="
if [[ $FAILED -eq 0 ]]; then
  echo "✅ All health checks PASSED"
  echo "WeftOS binary is healthy and functional"
  exit 0
else
  echo "❌ Health checks FAILED"
  echo "Please review errors above"
  exit 1
fi
