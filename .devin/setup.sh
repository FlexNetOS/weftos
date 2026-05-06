#!/usr/bin/env bash
# Devin workspace setup for FlexNetOS/weftos.
#
# This script bootstraps a fresh Devin VM (or any Ubuntu/Debian host) so that
# `scripts/build.sh check` succeeds. It mirrors the toolchain pinned in
# `rust-toolchain.toml` (Rust 1.93, edition 2024) and adds the WASM targets
# the workspace expects (`wasm32-unknown-unknown` for browser builds and
# `wasm32-wasip2` for server / plugin / edge builds).
#
# IMPORTANT: WeftOS mandates `scripts/build.sh` for ALL build / test / check /
# lint operations (see `CLAUDE.md` lines 43–47). Do not run `cargo build|test|
# check|clippy` directly except for one-off debugging — extend `scripts/build.sh`
# instead.

set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

log() { printf '\033[1;34m[devin-setup]\033[0m %s\n' "$*"; }

# 1. System dependencies. WeftOS pulls in fontconfig (egui shell), openssl,
#    pkg-config, cmake, clang, libclang, and protobuf-compiler. Match the
#    organization-level Devin env config so local + Devin stay aligned.
log "Installing system dependencies"
sudo apt-get update
sudo apt-get install -y \
  libssl-dev \
  pkg-config \
  cmake \
  clang \
  libclang-dev \
  protobuf-compiler \
  libfontconfig1-dev

# 2. Rust toolchain. The workspace pins Rust 1.93 in `rust-toolchain.toml`
#    (edition 2024 requires 1.85+; we pin 1.93). `rustup` honors the pin
#    automatically once invoked from inside the repo, but install the
#    components and targets explicitly so first-run is hermetic.
if ! command -v rustup >/dev/null 2>&1; then
  log "Installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y --default-toolchain none --profile minimal
  # shellcheck disable=SC1091
  . "${HOME}/.cargo/env"
fi

log "Installing Rust 1.93 (pinned in rust-toolchain.toml) with rustfmt + clippy"
rustup toolchain install 1.93 --component rustfmt,clippy
rustup default 1.93

log "Adding WASM targets (wasm32-unknown-unknown, wasm32-wasip2)"
rustup target add wasm32-unknown-unknown --toolchain 1.93
rustup target add wasm32-wasip2 --toolchain 1.93

# 3. Node.js + npm. Needed for the Fumadocs site at `docs/src/`, the
#    clawft-ui frontend, and the VSCode / browser extensions under
#    `extensions/`. Most agents will not need to invoke npm directly, but the
#    toolchain has to exist for `scripts/build.sh browser` and the docs build.
if ! command -v node >/dev/null 2>&1 || ! node --version | grep -qE '^v20\.'; then
  log "Installing Node.js 20.x"
  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
  sudo apt-get install -y nodejs
else
  log "Node.js already installed: $(node --version)"
fi

# 4. Verification. `scripts/build.sh check` is the canonical fast type-check
#    (cargo check --workspace under the hood). Use this and not raw cargo.
log "Verifying workspace compiles (scripts/build.sh check)"
bash scripts/build.sh check

log "Setup complete."
