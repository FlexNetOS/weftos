#!/usr/bin/env bash
#
# Attractor node 3: Validate (weftos runtime side).
#
# The validate node is the contract gate: if it fails, the trajectory is
# still distilled (verdict=fail, see distill.sh "failures train the bank
# too") but optimize is skipped — there's nothing to optimize when the
# build is broken. The runner enforces this; nodes don't decide routing.
#
# Per CLAUDE.md (lines 43-47), this node MUST go through
# `scripts/build.sh`, not raw cargo. The full Phase gate is:
#
#     scripts/build.sh gate
#
# That runs the canonical 11-step gate (workspace tests + release
# binaries + WASI + browser-WASM checks per crate + UI build + voice
# feature) — see `.claude/skills/self-learning-loop/SKILL.md` for the
# step-by-step listing.
#
# In stub mode we run the cheapest signal — `scripts/build.sh check` —
# so a green run still proves the workspace compiles.
#
# Defensively unset RUSTC_WRAPPER if it points at a binary that isn't
# on PATH — happens in fresh dev environments that have sccache
# configured in `.cargo/config.toml` but haven't installed it yet. This
# is a pure environment fix; real CI runs install sccache before
# invoking us.

set -euo pipefail

readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ -n "${RUSTC_WRAPPER:-}" ]] && ! command -v "${RUSTC_WRAPPER}" >/dev/null 2>&1; then
    echo "validate: ${RUSTC_WRAPPER} not on PATH; unsetting RUSTC_WRAPPER" >&2
    unset RUSTC_WRAPPER
fi

if [ ! -x "$ROOT/scripts/build.sh" ]; then
    echo "validate: scripts/build.sh missing or not executable; cannot run gate" >&2
    echo '{"validated":false,"check":"scripts/build.sh"}'
    exit 1
fi

# Capture build.sh's stdout+stderr to a sidecar build log so the operator
# (and the self-learning loop's identify node) can diagnose failures.
# This script's own stdout stays the pure JSON contract.
#
# When invoked by `scripts/attractor.sh run`, the runner exports
# ATTRACTOR_RUN_DIR pointing at the per-run stdout dir, which gives
# every concurrent or back-to-back pipeline run its own isolated
# build_log. When invoked standalone (`scripts/attractor.sh node
# validate`), we fall back to the fixed runs/ path.
#
# IMPORTANT: pass --verbose so build.sh's run_cmd does NOT pipe cargo
# through `tail -5`. Without --verbose, the sidecar log captures only
# the final 5 lines of cargo output — useless for diagnosing real
# failures (the actual error usually lives further up). With --verbose,
# we get the full diagnostics.
build_log="${ATTRACTOR_RUN_DIR:-${ROOT}/.attractor/runs}/validate.stderr"
mkdir -p "$(dirname "$build_log")"

# JSON-escape the build_log path before embedding in the contract so a
# repo root containing \ or " doesn't corrupt the JSONL audit record
# the runner parses. Mirrors the escape in distill.sh.
escaped_log="${build_log//\\/\\\\}"
escaped_log="${escaped_log//\"/\\\"}"

# Cheapest signal that still uses the canonical entrypoint. The full
# gate (`scripts/build.sh gate`) runs in CI; this stub keeps local
# iteration fast while still proving the workspace compiles via the
# documented entrypoint.
if "$ROOT/scripts/build.sh" check --verbose >"$build_log" 2>&1; then
    printf '{"validated":true,"check":"scripts/build.sh check --verbose","stderr_log":"%s"}\n' "$escaped_log"
    exit 0
fi

printf '{"validated":false,"check":"scripts/build.sh check --verbose","stderr_log":"%s","hint":"see stderr_log for build.sh check diagnostics"}\n' "$escaped_log"
exit 1
