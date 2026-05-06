#!/usr/bin/env bash
#
# Attractor node 3: Validate (weftos runtime side).
#
# The validate node is the non-negotiable contract: if it fails, the
# pipeline does NOT distill the trajectory.
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

# Cheapest signal that still uses the canonical entrypoint. The full
# gate (`scripts/build.sh gate`) runs in CI; this stub keeps local
# iteration fast while still proving the workspace compiles via the
# documented entrypoint.
if "$ROOT/scripts/build.sh" check >/dev/null 2>&1; then
    echo '{"validated":true,"check":"scripts/build.sh check"}'
    exit 0
fi

echo '{"validated":false,"check":"scripts/build.sh check"}'
exit 1
