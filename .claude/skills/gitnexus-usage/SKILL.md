---
name: gitnexus-usage
description: Use the GitNexus code-graph MCP server to query, impact-analyze, and refactor across the weftos workspace (~40 crates + clawft-* + weft daemon + weaver). Reach for this skill BEFORE doing wide refactors, large grep sweeps, or "where is X called from" questions — the graph is faster and more accurate than reading files. Lives at .gitnexus/ (gitignored). MCP tools: query, context, impact, detect_changes, rename, cypher.
---

# GitNexus Usage — WeftOS

## What it is

GitNexus is a code-intelligence MCP server. The CLI walks the repo, builds a
LadybugDB knowledge graph in `.gitnexus/`, and exposes it to any agent
runtime via MCP. It tracks symbols, call edges, type relationships, and
impact graphs — all on disk, no external service.

## When to use it (vs. grep / ripgrep / reading files)

Reach for GitNexus first when you're about to do any of these:

| Task | GitNexus tool | Why it beats grep |
|---|---|---|
| "Where is `weave_phase` called from?" | `impact` | Resolves through traits, generics, re-exports |
| "Show me everything in `clawft-core`" | `context` | Returns the structural neighborhood, not flat text |
| "Find all impls of trait T" | `query` | Type-aware; grep finds the string `impl T`, GitNexus knows what `T` resolves to |
| "What's affected if I rename `Goal`?" | `rename` (dry-run) | Walks every call/use site at the AST level |
| "What changed since last index?" | `detect_changes` | Uses git HEAD diff against the indexed snapshot |
| Custom traversal | `cypher` | Full Cypher-style query against the graph |

## Install

```bash
scripts/install-gitnexus.sh
```

This is idempotent. It:
1. Verifies Node 20+ is on PATH (GitNexus engine requirement).
2. Runs `npx -y gitnexus@${GITNEXUS_VERSION:-latest} analyze --skip-agents-md`
   to (re)index. `--skip-agents-md` is **required** — without it GitNexus
   rewrites CLAUDE.md and clobbers the hand-curated rules (workspace
   exclusion list, `scripts/build.sh` mandate, etc.).
3. Runs `npx -y gitnexus@${GITNEXUS_VERSION:-latest} setup` to register
   the MCP server with any editor configs on disk.

## Index scope (READ THIS)

GitNexus walks the repo using `.gitignore` + `.gitnexusignore` — **not**
Cargo workspace membership. WeftOS-specific gotcha:

- `gui/src-tauri/` is **out of the cargo workspace** (it's a standalone
  Tauri shell with different tooling) but it IS tracked in git, so
  GitNexus would index it without explicit exclusion.
- The repo ships a top-level `.gitnexusignore` that lists
  `gui/src-tauri/` for this reason. If you want to index the Tauri app
  separately (registered as a distinct repo in the gitnexus registry),
  run from the repo root:

  ```bash
  npx -y gitnexus@latest analyze gui/src-tauri/ --skip-agents-md
  ```

## Index location

| Path | Contents | Status |
|---|---|---|
| `.gitnexus/` | LadybugDB graph DB, embedding cache | **Gitignored.** Regenerable. |
| `~/.gitnexus/registry.json` | Per-machine registry of all indexed repos | Per-machine, never committed. |
| `.claude/skills/gitnexus/` | 6 helper SKILLs auto-installed by `analyze` | The repo's top-level `.gitignore` (`.claude`) already keeps them untracked. |

Re-index manually:

```bash
npx -y gitnexus@latest analyze --skip-agents-md            # incremental
GITNEXUS_FORCE=1 scripts/install-gitnexus.sh               # full re-walk
```

## Smoke test

```bash
npx -y gitnexus@latest status     # confirms graph exists + last index time
npx -y gitnexus@latest list       # lists all indexed repos in your registry
```

## Known gotchas

1. **`.claude/skills/gitnexus/` is auto-clobbered.** Even with
   `--skip-agents-md`, gitnexus 1.x always re-installs 6 SKILL files
   there (verified in upstream `dist/cli/ai-context.js`). The
   repo-wide `.gitignore` covers this via `.claude` exclusion.
2. **Don't run cargo directly to validate gitnexus changes.** Per
   CLAUDE.md (lines 43-47), use `scripts/build.sh check` and friends.
3. **License**: PolyForm Noncommercial 1.0.0. We invoke the upstream
   CLI; nothing is vendored.

## Cross-repo

The matching skill in **ruvector** is at
`.claude/skills/gitnexus-usage/SKILL.md`. RuVector has a larger
workspace (~150 crates) and no equivalent of `gui/src-tauri/`.
