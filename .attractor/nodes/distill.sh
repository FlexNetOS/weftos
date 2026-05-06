#!/usr/bin/env bash
#
# Attractor node 5: Distill (weftos runtime side).
#
# Promote the validated trajectory back into ruvector's ReasoningBank
# over the `pi-brain` MCP, so the next iteration starts from a richer
# memory. This is the closing of the self-learning loop and it is
# CROSS-REPO: weftos is the source, ruvector is the sink.
#
# Full implementation:
#
#   weaver mcp call pi-brain brain_share \
#       --category solution \
#       --title "$title" \
#       --content "$trajectory_json" \
#       --tags 'weftos,gate,attractor'
#
# Stub: writes a single record into .attractor/runs/<stamp>.distill.jsonl
# so the run history is auditable even before the cross-repo bridge is
# wired (see Phase 4 + Phase 7).

set -euo pipefail

readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# When invoked by `scripts/attractor.sh run`, ATTRACTOR_RUN_DIR points at
# the per-run stdout dir; landing the distill record there keeps every
# artifact for that iteration co-located. Standalone invocation
# (`scripts/attractor.sh node distill`) falls back to the shared runs/
# dir so the audit trail is still preserved.
readonly OUT_DIR="${ATTRACTOR_RUN_DIR:-$ROOT/.attractor/runs}"
mkdir -p "$OUT_DIR"

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
out="$OUT_DIR/${stamp}.distill.jsonl"
printf '{"distilled":true,"stub":true,"stamp":"%s"}\n' "$stamp" > "$out"

# JSON-escape the output path. If $ROOT contains a quote or backslash
# (rare but possible on dev hosts) the unescaped form would emit invalid
# JSON and break the JSONL parser in the runner.
escaped_out="${out//\\/\\\\}"
escaped_out="${escaped_out//\"/\\\"}"
printf '{"distilled":true,"stub":true,"output":"%s"}\n' "$escaped_out"
