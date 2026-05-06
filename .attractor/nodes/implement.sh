#!/usr/bin/env bash
#
# Attractor node 2: Implement (weftos runtime side).
#
# weftos IS the runtime that applies code edits — unlike ruvector,
# which is advisory. The implement node hands the candidate from
# `identify` to a coding agent through `weaver`:
#
#   weaver agent run --prompt "$prompt" --max-retries 2
#
# Real wiring:
#   * `crates/clawft-weave/src/commands/` (weaver subcommands)
#   * `crates/clawft-llm` (provider abstraction; 11 builtins)
#   * Optional: route through `clawft-cognitum` for cognitum-aware edits
#
# This is currently a STUB.

set -euo pipefail

readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo '{"applied":false,"reason":"phase3-scaffold","stub":true}'
