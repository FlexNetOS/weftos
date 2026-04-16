# Feature request — WeftOS white-label bootstrap + logs

**Filed**: 2026-04-15
**Source**: Valtech client engagement (DR-010d in `.planning/clients/valtech/DECISION-RECORD.md`)
**Priority**: P1 for Valtech engagement; P2 for general release
**Estimated effort**: one sprint on the kernel + CLI

## Problem

WeftOS is MIT-licensed and consultancies (e.g. Valtech) may want to deploy
it under their own brand. Today the string "WeftOS" is hard-coded in
banners, log prefixes, CLI help text, and the WebUI header. White-labeling
requires grep-and-sed across the repo and breaks on the next release.

## Desired behavior

1. `weft init --brand "Valtech Agentic Mesh"` sets a project-wide brand
   token in `.weft/config.toml` (or equivalent)
2. All log output, boot banners, CLI `--help`, and WebUI headers read the
   brand token and fall back to "WeftOS" when unset
3. `weft config set brand "Valtech Agentic Mesh"` for post-init changes
4. `weft config get brand` returns the current value

## Non-goals

- Not removing WeftOS attribution entirely (the MIT license requires
  attribution — the brand feature is about display name, not license
  compliance)
- Not a full theming system (colors, logos, etc.) — just the name string
- Not changing the crate name, package name, or binary name — only
  runtime-display strings

## Implementation notes

- Brand token lives in the kernel config service (see `clawft-kernel/src/config_service.rs`)
- Add a `brand()` accessor returning `&str` with the "WeftOS" default
- Replace hard-coded strings in:
  - `clawft-kernel/src/console.rs` (boot banner, log prefixes)
  - `clawft-weave/src/` (CLI help text, version output)
  - `docs/src/app/` (where applicable — the docs site stays WeftOS-branded)
- License compliance: render "powered by WeftOS (MIT)" in an "about" /
  "version" output that's always visible, regardless of brand token

## Test plan

- Unit: brand token round-trips through config service
- Integration: `weft init --brand X && weft status` shows "X" not "WeftOS"
- Compliance: `weft version --long` shows "powered by WeftOS (MIT)"
  regardless of brand setting

## Tracker

Open as GitHub issue in the clawft repo once this note is reviewed.
Blocks: nothing (Valtech can use manual rebranding in the meantime).
Unblocks: cleaner Valtech white-label story, reusable for any future
consultancy client.
