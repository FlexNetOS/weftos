---
name: understand-anything-usage
description: Use Understand-Anything (LLM + static analysis dashboard generator) when starting on an unfamiliar area of weftos, onboarding to a clawft-* crate or weft daemon module, or producing an architecture map for a refactor. Provides 8 slash-commands (/understand, /understand-onboard, /understand-explain, /understand-diff, /understand-chat, /understand-domain, /understand-knowledge, /understand-dashboard) that emit structured artifacts under `.understand-anything/`. Skill is plugin-style — runs through Claude Code, Codex, Gemini, OpenCode, or pi-coder.
---

# Understand-Anything Usage — WeftOS

## What it is

Upstream: https://github.com/Lum1104/Understand-Anything

A pnpm-workspace tool that combines tree-sitter static analysis with LLM
prompts to produce interactive code-comprehension dashboards. Plugin-style
distribution — slash-commands resolve from `~/.agents/skills/` and point
into a single clone at `~/.{platform}/understand-anything`.

## When to use it

Reach for an `/understand-*` command when you're about to:

| Situation | Command | What you get |
|---|---|---|
| Onboarding to a clawft crate or weft module | `/understand-onboard <crate>` | Guided walkthrough + key entry points |
| "What does this codebase do?" | `/understand` (full pass) | Knowledge graph + dashboard JSON |
| "Why does X work this way?" | `/understand-explain <symbol>` | Explanation grounded in the static graph |
| "What changed in this PR?" | `/understand-diff <ref>` | Diff-aware impact analysis |
| Domain-specific question | `/understand-domain <topic>` | Filtered view of the graph |
| Q&A over the codebase | `/understand-chat` | Persistent chat with graph context |
| Pre-existing graph already? | `/understand-dashboard` | Render the dashboard from cached JSON |
| Cross-link with knowledge base | `/understand-knowledge` | Bridges code graph ↔ memory store |

## Install

```bash
scripts/install-understand-anything.sh
```

Idempotent. Picks the first agent runtime found on PATH as the canonical
clone (under `~/.<runtime>/understand-anything/`) and symlinks the others
to it. Refuses to silently overwrite local edits or diverged commits in
the clone (uses `merge --ff-only`, fails loudly on dirty trees).

## WeftOS-specific scope

- The full pass `/understand` walks the cargo workspace. `gui/src-tauri/`
  is **out of the workspace** and won't be picked up automatically. Run
  `/understand` from inside `gui/src-tauri/` if you want the Tauri shell
  analyzed separately.
- Per CLAUDE.md (lines 43-47), all build/test/check/lint operations go
  through `scripts/build.sh`. Understand-Anything won't override that —
  it just reads source.

## Output location

| Path | Contents | Status |
|---|---|---|
| `.understand-anything/knowledge-graph.json` | Per-repo knowledge graph | **Tracked.** The canonical artifact — re-`/understand` only when source has materially shifted. |
| `.understand-anything/onboarding.md`, `tours/` | `/understand-onboard` + dashboard tours output | **Tracked.** Curated, hand-edited later. |
| `.understand-anything/intermediate/`, `diff-overlay.json`, `file-content.cache.json` | Per-agent scratch / dashboard cache | **Gitignored.** Transient. |

Selective tracking is enforced by `.understand-anything/.gitignore`
(ignore-everything pattern with explicit `!knowledge-graph.json`,
`!onboarding.md`, `!tours/` allow-listings).

## Versioning

Upstream pins `pnpm` and `node` via `packageManager` field — `node>=22`
and `pnpm>=10` are required to run the dashboard. The bootstrap script
covers the runtime side; the dashboard itself is `pnpm dev:dashboard`
inside the clone if you want to launch it.

## Cross-repo

The matching skill in **ruvector** is at
`.claude/skills/understand-anything-usage/SKILL.md`. WeftOS has fewer
crates (~40 vs. ruvector's ~150) so passes complete faster.
