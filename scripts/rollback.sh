#!/bin/bash

# WeftOS Automatic Rollback Script
# Reverts to previous binary if deployment fails or health checks fail
# Generated from: ai-devops-fork-deploy.prompt.md
#
# Usage: WEFTOS_BINARY=/path/to/weftos bash scripts/rollback.sh

set -u

SERVICE="weftos"
BINARY_PATH="${WEFTOS_BINARY:-/opt/weftos/bin/weftos}"
BACKUP_PATH="${BINARY_PATH}.old"

# Resolve script directory for relative path lookups
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HEALTH_CHECK_SCRIPT="${SCRIPT_DIR}/health-check.sh"

echo "=== WeftOS Automatic Rollback Procedure ==="
echo "Timestamp: $(date)"
echo "Binary: $BINARY_PATH"
echo ""

# Verify backup exists
if [[ ! -f "$BACKUP_PATH" ]]; then
  echo "❌ CRITICAL: No backup binary found at $BACKUP_PATH"
  echo "Cannot perform rollback. Manual intervention required:"
  echo "   1. Check $(dirname "$BINARY_PATH") for previous versions"
  echo "   2. Restore manually: sudo cp /path/to/previous/weftos $BINARY_PATH"
  echo "   3. Restart service if configured: sudo systemctl restart $SERVICE"
  exit 1
fi

echo "[1/4] Restoring previous binary..."
if sudo cp "$BACKUP_PATH" "$BINARY_PATH"; then
  sudo chmod +x "$BINARY_PATH"
  echo "✅ Binary restored from backup"
else
  echo "❌ Failed to restore binary"
  exit 1
fi

echo "[2/4] Restarting service (if configured)..."
if command -v systemctl >/dev/null 2>&1; then
  if systemctl list-unit-files --type=service 2>/dev/null | grep -q "^${SERVICE}.service"; then
    if sudo systemctl restart "$SERVICE" 2>/dev/null; then
      echo "✅ Service restarted"
    else
      echo "⚠️  Service restart failed (binary restored but service may need manual start)"
    fi
  else
    echo "ℹ️  No systemd service configured; binary restored only"
  fi
else
  echo "ℹ️  systemctl not available; binary restored only"
fi

# Give service time to start
echo "[3/4] Waiting for service to stabilize..."
sleep 5

# Verify binary works
echo "[4/4] Verifying restored binary..."
if [[ -x "$BINARY_PATH" ]]; then
  if timeout 10 "$BINARY_PATH" version >/dev/null 2>&1; then
    echo "✅ Restored binary responds to version check"
  else
    echo "⚠️  Restored binary does not respond to version check"
  fi
else
  echo "⚠️  Restored binary not executable"
fi

# Optional: Run health check script if available
if [[ -f "$HEALTH_CHECK_SCRIPT" ]]; then
  echo ""
  echo "Running comprehensive health check..."
  if WEFTOS_BINARY="$BINARY_PATH" bash "$HEALTH_CHECK_SCRIPT"; then
    echo ""
    echo "✅ ROLLBACK SUCCESSFUL"
    echo "Previous version has been restored and is operational"
    exit 0
  else
    echo ""
    echo "⚠️  Rollback completed but health checks indicate issues"
    echo "Review logs and consider manual intervention"
    exit 1
  fi
else
  echo ""
  echo "✅ ROLLBACK COMPLETED"
  echo "Previous version restored. Manual health check recommended."
  echo "Run: $BINARY_PATH version"
  exit 0
fi
