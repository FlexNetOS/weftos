#!/bin/bash

# WeftOS Hardware Detection & Optimization Adapter
# Dynamically adjusts build and runtime configuration based on hardware
# Generated from: ai-devops-fork-deploy.prompt.md
#
# Usage: ./hardware-scan.sh [--env]
#   --env   Output only KEY=VALUE pairs (safe for sourcing in CI)

set -u

MODE="human"
if [[ "${1:-}" == "--env" ]]; then
  MODE="env"
fi

# ── CPU Info ──────────────────────────────────────────────────────────
CPU_ARCH=$(uname -m 2>/dev/null || echo "unknown")
CPU_CORES=$(nproc 2>/dev/null || echo "1")
CPU_MODEL=$(lscpu 2>/dev/null | grep "Model name" | cut -d: -f2 | sed 's/^[[:space:]]*//' || echo "Unknown")

# ── Memory ─────────────────────────────────────────────────────────────
if command -v free >/dev/null 2>&1; then
  TOTAL_MEM_RAW=$(free -g 2>/dev/null | awk 'NR==2{printf "%.0f", $2}')
  AVAILABLE_MEM_RAW=$(free -g 2>/dev/null | awk 'NR==2{printf "%.0f", $7}')
  TOTAL_MEM=${TOTAL_MEM_RAW:-0}
  AVAILABLE_MEM=${AVAILABLE_MEM_RAW:-0}
else
  TOTAL_MEM="0"
  AVAILABLE_MEM="0"
fi

# ── Storage (for models) ───────────────────────────────────────────────
MODEL_PATH="/opt/weftos/models"
if [[ -d "$MODEL_PATH" ]] && command -v df >/dev/null 2>&1; then
  STORAGE_AVAIL=$(df -BG "$MODEL_PATH" 2>/dev/null | tail -1 | awk '{print $4}' | sed 's/G//' || echo "0")
else
  STORAGE_AVAIL=""
fi

# ── GPU Detection ─────────────────────────────────────────────────────
GPU_TYPE="none"
GPU_MEM="0"
GPU_COUNT="0"
if command -v nvidia-smi >/dev/null 2>&1; then
  GPU_TYPE="nvidia"
  GPU_MEM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1 || echo "0")
  GPU_COUNT=$(nvidia-smi --list-gpus 2>/dev/null | wc -l || echo "0")
elif command -v rocm-smi >/dev/null 2>&1; then
  GPU_TYPE="amd"
elif [[ -d /dev/dri ]]; then
  GPU_TYPE="intel"
fi

# ── Network ────────────────────────────────────────────────────────────
NETWORK="offline"
if ping -c 1 -W 2 8.8.8.8 >/dev/null 2>&1; then
  NETWORK="connected"
fi

# ── Adaptive Optimization Logic ────────────────────────────────────────
FORCE_SMALL_MODELS=0
PREFER_SMALL_MODELS=0
USE_NETWORK_FETCH=0
REDUCE_PARALLELISM=0
INCREASE_TIMEOUTS=0
DISABLE_CACHE=0
AGGRESSIVE_CLEANUP=0

if [[ "$TOTAL_MEM" =~ ^[0-9]+$ && "$TOTAL_MEM" -lt 4 ]]; then
  FORCE_SMALL_MODELS=1
  DISABLE_CACHE=1
  REDUCE_PARALLELISM=1
  INCREASE_TIMEOUTS=1
elif [[ "$TOTAL_MEM" =~ ^[0-9]+$ && "$TOTAL_MEM" -lt 8 ]]; then
  PREFER_SMALL_MODELS=1
  REDUCE_PARALLELISM=1
fi

if [[ -z "$STORAGE_AVAIL" || "$STORAGE_AVAIL" == "?" || "$STORAGE_AVAIL" == "0" ]] || \
   [[ "$STORAGE_AVAIL" =~ ^[0-9]+$ && "$STORAGE_AVAIL" -lt 10 ]]; then
  USE_NETWORK_FETCH=1
  AGGRESSIVE_CLEANUP=1
fi

if [[ "$CPU_CORES" =~ ^[0-9]+$ && "$CPU_CORES" -lt 4 ]]; then
  REDUCE_PARALLELISM=1
  INCREASE_TIMEOUTS=1
fi

# ── Machine-Readable Output (safe for sourcing) ────────────────────────
if [[ "$MODE" == "env" ]]; then
  cat <<EOF
FORCE_SMALL_MODELS=$FORCE_SMALL_MODELS
PREFER_SMALL_MODELS=$PREFER_SMALL_MODELS
USE_NETWORK_FETCH=$USE_NETWORK_FETCH
REDUCE_PARALLELISM=$REDUCE_PARALLELISM
INCREASE_TIMEOUTS=$INCREASE_TIMEOUTS
DISABLE_CACHE=$DISABLE_CACHE
AGGRESSIVE_CLEANUP=$AGGRESSIVE_CLEANUP
CPU_CORES=$CPU_CORES
TOTAL_MEM=$TOTAL_MEM
STORAGE_AVAILABLE=${STORAGE_AVAIL:-0}
GPU_TYPE=$GPU_TYPE
GPU_MEM=$GPU_MEM
GPU_COUNT=$GPU_COUNT
NETWORK=$NETWORK
EOF
  exit 0
fi

# ── Human-Readable Output ─────────────────────────────────────────────
echo "=== WeftOS Hardware Scan for Optimization ==="
echo ""
echo "CPU: $CPU_ARCH ($CPU_MODEL), $CPU_CORES cores"
echo "RAM: ${TOTAL_MEM}GB total, ${AVAILABLE_MEM}GB available"

if [[ -n "$STORAGE_AVAIL" && "$STORAGE_AVAIL" != "?" ]]; then
  echo "Model Storage: ${STORAGE_AVAIL}GB available at $MODEL_PATH"
else
  echo "Model Storage: Not mounted or unknown (will use network fetch)"
fi

case "$GPU_TYPE" in
  nvidia) echo "GPU: NVIDIA, ${GPU_COUNT}x GPU with ${GPU_MEM}MB VRAM each" ;;
  amd)    echo "GPU: AMD ROCm detected" ;;
  intel)  echo "GPU: Intel iGPU or generic GPU detected" ;;
  *)      echo "GPU: None detected (CPU-only inference)" ;;
esac

case "$NETWORK" in
  connected) echo "Network: Connected" ;;
  *)         echo "Network: Limited/offline connectivity" ;;
esac

echo ""
echo "=== Optimization Recommendations ==="
echo ""

if [[ "$TOTAL_MEM" =~ ^[0-9]+$ && "$TOTAL_MEM" -lt 4 ]]; then
  echo " CRITICAL: Very Low RAM ($TOTAL_MEM GB)"
  echo "   Actions:"
  echo "   ├─ Force small models only"
  echo "   ├─ Disable inference caching"
  echo "   ├─ Reduce batch size to 1"
  echo "   └─ Consider swap: swapon /swapfile"
elif [[ "$TOTAL_MEM" =~ ^[0-9]+$ && "$TOTAL_MEM" -lt 8 ]]; then
  echo " WARNING: Limited RAM ($TOTAL_MEM GB)"
  echo "   Actions:"
  echo "   ├─ Prefer small models (7B or smaller)"
  echo "   ├─ Limit batch size to 2"
  echo "   └─ Monitor swap usage"
else
  echo " RAM: OK (${TOTAL_MEM}GB available for inference)"
fi

if [[ -z "$STORAGE_AVAIL" || "$STORAGE_AVAIL" == "?" || "$STORAGE_AVAIL" == "0" ]] || \
   [[ "$STORAGE_AVAIL" =~ ^[0-9]+$ && "$STORAGE_AVAIL" -lt 10 ]]; then
  echo " WARNING: Low/Unavailable Storage"
  echo "   Actions:"
  echo "   ├─ Fetch models on-demand from HuggingFace/S3"
  echo "   ├─ Enable model cleanup after use"
  echo "   └─ Cache in RAM if possible"
else
  echo " Storage: OK (${STORAGE_AVAIL}GB available for models)"
fi

if [[ "$CPU_CORES" =~ ^[0-9]+$ && "$CPU_CORES" -lt 4 ]]; then
  echo " WARNING: Low CPU Cores ($CPU_CORES)"
  echo "   Actions:"
  echo "   ├─ Reduce parallel workers to $((CPU_CORES - 1))"
  echo "   ├─ Increase inference timeout to 60s"
  echo "   └─ Consider GPU acceleration if available"
elif [[ "$CPU_CORES" =~ ^[0-9]+$ && "$CPU_CORES" -lt 8 ]]; then
  echo " INFO: Moderate CPU ($CPU_CORES cores)"
  echo "   → Setting parallel workers to $((CPU_CORES / 2))"
else
  echo " CPU: OK ($CPU_CORES cores available)"
fi

echo ""
echo "=== Export Optimization Variables ==="
echo ""

cat <<EOF
FORCE_SMALL_MODELS=$FORCE_SMALL_MODELS          (Force 7B or smaller models)
PREFER_SMALL_MODELS=$PREFER_SMALL_MODELS        (Recommend small models)
USE_NETWORK_FETCH=$USE_NETWORK_FETCH            (Fetch models on-demand)
REDUCE_PARALLELISM=$REDUCE_PARALLELISM          (Limit concurrent requests)
INCREASE_TIMEOUTS=$INCREASE_TIMEOUTS            (Extend latency budgets)
DISABLE_CACHE=$DISABLE_CACHE                    (No inference caching)
AGGRESSIVE_CLEANUP=$AGGRESSIVE_CLEANUP          (Clean models after use)

CPU_CORES=$CPU_CORES
TOTAL_MEM=$TOTAL_MEM GB
STORAGE_AVAILABLE=${STORAGE_AVAIL:-0} GB
EOF

echo ""
echo " Hardware scan complete. Variables ready for deployment."
