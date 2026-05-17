---
name: mempalace-usage
description: Use the MemPalace MCP server for persistent cross-session memory of decisions, conversations, and learnings. The palace lives at ~/.mempalace/ (per-user, not per-repo) and exposes 30 MCP tools — search, add_drawer, list_drawers, get_drawer, update_drawer, delete_drawer, list_wings, list_rooms, get_taxonomy, check_duplicate, reconnect, status, sync, plus knowledge graph (kg_query, kg_add, kg_invalidate, kg_timeline, kg_stats), graph traversal (traverse, graph_stats, create_tunnel, delete_tunnel, list_tunnels, find_tunnels, follow_tunnels), diary (diary_read, diary_write), and utilities (get_aaak_spec, hook_settings, memories_filed_away). Reach for this skill BEFORE asking the user to re-explain something they already told you, AFTER making a non-obvious decision worth remembering, or when you need verbatim recall of past conversations.
triggers:
  - mempalace
  - mem palace
  - memory palace
  - persistent memory
  - long-term memory
  - cross-session memory
  - remember this
  - what did we decide
  - check memory
  - search memory
  - "/mempalace"
  - .mempalace/
---

# MemPalace Usage

## What it is

MemPalace is a local-first AI memory store. It saves verbatim text,
indexes it with sentence-transformer embeddings stored in ChromaDB, and
exposes search/store/list operations over MCP. **Nothing leaves the
local box** — no API keys, no cloud calls. The palace itself lives at
`~/.mempalace/` (per-user, cross-project).

The data model: **wings** (people / projects) → **rooms** (topics,
drawn from folder structure) → **drawers** (verbatim content). Searches
can be scoped to a wing or room, or run flat across the whole palace.

**Version**: MemPalace 3.3.5 ships 30 MCP tools (verified against
`mempalace/mcp_server.py`). Previous documentation claimed 9 tools —
that was accurate for v2.x but not for the current release.

## When to reach for MemPalace (vs. GitNexus / Understand-Anything)

| Question type | Tool | Why |
|---|---|---|
| "What did we decide about X last week?" | **MemPalace** `search` | Verbatim conversation history with semantic recall |
| "What's the call graph for `fn foo`?" | GitNexus `impact` | Structural code-graph traversal |
| "What does this crate do?" | Understand-Anything `/understand-onboard` | LLM-assembled comprehension graph |
| "Why was this pattern chosen?" | **MemPalace** `search` first, then GitNexus `context` | Memory captures intent, GitNexus captures structure |
| "Has this been discussed before?" | **MemPalace** `check_duplicate` | Pre-commit dedup against existing memories |
| "What facts do we know about entity E?" | **MemPalace** `kg_query` | Temporal knowledge graph with validity windows |
| "What connects project A and project B?" | **MemPalace** `find_tunnels` / `follow_tunnels` | Cross-wing semantic bridges |

The three tools are **complementary**: GitNexus = structure, Understand-Anything = comprehension, MemPalace = history/intent. An ideal pre-edit pass queries all three.

## Install

```bash
scripts/install-mempalace.sh
```

This is idempotent. It:
1. Verifies Python 3.9+ is on PATH.
2. Installs via **pipx** (preferred), **`uv tool install`** (modern alternative), or **`pip install --user`** (fallback).
   To close the dependency-confusion window, the install subprocess runs with:
   - `--index-url https://pypi.org/simple/` (pin the primary index)
   - `PIP_EXTRA_INDEX_URL=''` (strip any inherited extra-index env var)
   - `PIP_CONFIG_FILE=/dev/null` + `--no-config` (ignore `pip.conf` / `pip.ini` `extra-index-url` entries)
   Override with `MEMPALACE_INDEX_URL=...` only if you have a verified internal proxy.
3. Surfaces the Claude Code plugin install command if `claude` is on PATH.

**The bootstrap does NOT call `mempalace init`.** Per upstream issue
[MemPalace/mempalace#185](https://github.com/MemPalace/mempalace/issues/185),
`mempalace init <dir>` writes `<dir>/entities.json` and
`<dir>/mempalace.yaml` into the target directory — running it against
the repo root would dirty every contributor's checkout with two
untracked generated files. Instead, run init manually against a
directory you're willing to dirty (a per-user staging dir, or a
project directory outside the repo):

```bash
mkdir -p ~/.mempalace/projects/weftos
mempalace init ~/.mempalace/projects/weftos   # safe to dirty; per-user
```

The palace database itself lives at `~/.mempalace/` regardless of where
you ran init — the init target only receives the two metadata files.

Mining (importing source/docs into the palace) is **OFF by default**.
For weftos's ~40 crates + `weft` daemon + `weaver` it's typically a
1–3 minute pass. Opt in only after you've run `mempalace init`
somewhere:

```bash
mempalace mine "$(pwd)"                                  # code + docs
mempalace mine "$(pwd)" --mode convos                    # conversation exports
mempalace mine ~/chats/ --mode convos --extract general  # auto-classify
```

`mempalace mine <dir>` does NOT write `entities.json`/`mempalace.yaml`
into `<dir>` — it only ingests file content into `~/.mempalace/`. Safe
to run against any directory.

## MCP Tool Reference (30 tools)

### Palace Overview

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_status` | _none_ | Palace health: total drawers, wing/room breakdown, AAAK spec, Memory Protocol |
| `mempalace_reconnect` | _none_ | Force reconnect after external writes; clears stale HNSW caches |
| `mempalace_memories_filed_away` | _none_ | Check if a recent palace checkpoint was saved |
| `mempalace_get_aaak_spec` | _none_ | Get the compressed AAAK memory dialect specification |
| `mempalace_hook_settings` | `silent_save?`, `desktop_toast?` | Get/set hook behavior (silent save, desktop toast) |

### Search & Retrieval

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_search` | `query` (required, max 250 chars), `wing?`, `room?`, `limit?` (1-100), `max_distance?` (0-2), `context?` | Semantic search with cosine + BM25 scoring. `query` = keywords only; `context` = background for re-ranking |
| `mempalace_check_duplicate` | `content` (required), `threshold?` (0-1, default 0.9) | Check if content already exists before filing |
| `mempalace_get_drawer` | `drawer_id` (required) | Fetch a single drawer by ID — full content + metadata |
| `mempalace_list_drawers` | `wing?`, `room?`, `limit?` (1-100), `offset?` | List drawers with pagination, previews, total count |

### Taxonomy Navigation

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_list_wings` | _none_ | List all wings with drawer counts |
| `mempalace_list_rooms` | `wing?` | List rooms within a wing (or all rooms) |
| `mempalace_get_taxonomy` | _none_ | Full wing → room → drawer count tree |

### Write Operations

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_add_drawer` | `wing` (required), `room` (required), `content` (required), `source_file?`, `added_by?` | File verbatim content into a wing/room. Checks duplicates first |
| `mempalace_update_drawer` | `drawer_id` (required), `content?`, `wing?`, `room?` | Update an existing drawer's content or metadata |
| `mempalace_delete_drawer` | `drawer_id` (required) | Delete a drawer by ID. **Irreversible.** |
| `mempalace_sync` | `project_dir?`, `wing?`, `apply?` (default false) | Prune drawers whose source files are gitignored/deleted/moved. Dry-run by default |

### Knowledge Graph

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_kg_query` | `entity` (required), `as_of?` (YYYY-MM-DD), `direction?` (outgoing/incoming/both) | Query facts about an entity with temporal validity |
| `mempalace_kg_add` | `subject` (req), `predicate` (req), `object` (req), `valid_from?`, `valid_to?`, `source_closet?`, `source_file?`, `source_drawer_id?` | Add a fact (subject → predicate → object) with optional time window |
| `mempalace_kg_invalidate` | `subject` (req), `predicate` (req), `object` (req), `ended?` (default today) | Mark a fact as no longer true |
| `mempalace_kg_timeline` | `entity?` | Chronological timeline of facts for an entity (or everything) |
| `mempalace_kg_stats` | _none_ | Knowledge graph overview: entities, triples, current vs expired |

### Graph Traversal & Tunnels (Cross-Wing Connections)

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_graph_stats` | _none_ | Palace graph overview: total rooms, tunnel connections, edges |
| `mempalace_traverse` | `start_room` (required), `max_hops?` (default 2) | Walk the graph from a room, discovering connected ideas across wings |
| `mempalace_create_tunnel` | `source_wing` (req), `source_room` (req), `target_wing` (req), `target_room` (req), `label?`, `source_drawer_id?`, `target_drawer_id?` | Create an explicit cross-wing tunnel |
| `mempalace_delete_tunnel` | `tunnel_id` (required) | Delete a tunnel by ID |
| `mempalace_list_tunnels` | `wing?` | List all explicit tunnels, optionally filtered by wing |
| `mempalace_find_tunnels` | `wing_a?`, `wing_b?` | Find rooms that bridge two wings |
| `mempalace_follow_tunnels` | `wing` (req), `room` (req) | Follow tunnels from a room to see connected rooms in other wings |

### Agent Diary

| Tool | Parameters | Description |
|---|---|---|
| `mempalace_diary_write` | `agent_name` (req), `entry` (req, AAAK format), `topic?`, `wing?` | Write to your personal agent diary (per-agent, full history) |
| `mempalace_diary_read` | `agent_name` (req), `last_n?` (default 10), `wing?` | Read recent diary entries in AAAK format |

### Recommended pre-edit pattern

```
1. mempalace_search    "<task summary>"          # has anyone done this before?
2. gitnexus.context    <symbol>                  # structural neighborhood
3. /understand-explain <symbol>                  # comprehension layer
4. (do the work — via scripts/build.sh per CLAUDE.md mandate)
5. mempalace_add_drawer  wing=weftos room=<topic> content=<decision + rationale>
```

## MemPalace Memory Protocol

Every call to `mempalace_status` returns the **Memory Protocol** alongside palace stats. This protocol governs how agents should behave when working with memory:

1. **ON WAKE-UP**: Call `mempalace_status` to load palace overview + AAAK spec.
2. **BEFORE RESPONDING** about any person, project, or past event: call `mempalace_kg_query` or `mempalace_search` FIRST. Never guess — verify.
3. **IF UNSURE** about a fact (name, gender, age, relationship): say "let me check" and query the palace. Wrong is worse than slow.
4. **AFTER EACH SESSION**: call `mempalace_diary_write` to record what happened, what you learned, what matters.
5. **WHEN FACTS CHANGE**: call `mempalace_kg_invalidate` on the old fact, `mempalace_kg_add` for the new one.

This protocol ensures the AI KNOWS before it speaks. Storage is not memory — but storage + this protocol = memory.

## AAAK Dialect

MemPalace uses **AAAK** (Adaptive Agent Archive Kernel) as a compressed memory dialect:

- **ENTITIES**: 3-letter uppercase codes. ALC=Alice, JOR=Jordan, etc.
- **EMOTIONS**: *action markers*. *warm*=joy, *fierce*=determined, *raw*=vulnerable, *bloom*=tenderness.
- **STRUCTURE**: Pipe-separated fields. FAM: family | PROJ: projects | ⚠: warnings/reminders.
- **DATES**: ISO format (2026-03-31). COUNTS: Nx = N mentions.
- **IMPORTANCE**: ★ to ★★★★★ (1-5 scale).
- **HALLS**: hall_facts, hall_events, hall_discoveries, hall_preferences, hall_advice.
- **WINGS**: wing_user, wing_agent, wing_team, wing_code, wing_myproject, etc.
- **ROOMS**: Hyphenated slugs (e.g., chromadb-setup, gpu-pricing).

Example: `FAM: ALC→♡JOR | 2D(kids): RIL(18,sports) MAX(11,chess+swimming) | BEN(contributor)`

Call `mempalace_get_aaak_spec` for the full specification when you need to read or write AAAK.

## CLI Commands (v3.3.5)

These are the **actual** CLI commands. Do NOT confuse MCP tools with CLI commands — some operations exist only in one interface.

| Command | Description |
|---|---|
| `mempalace --version` | Show version |
| `mempalace init <dir>` | Initialize palace metadata in `<dir>` (writes `entities.json` + `mempalace.yaml`) |
| `mempalace mine <dir>` | Ingest code/docs into palace (`--mode convos` for chat exports) |
| `mempalace sweep <target>` | Process JSONL transcript files |
| `mempalace sync` | Gitignore-aware drawer prune (dry-run by default) |
| `mempalace search "<query>"` | Semantic search (`--wing`, `--room` filters) |
| `mempalace compress` | Compress drawers with AAAK (`--wing`, `--dry-run`) |
| `mempalace wake-up` | Print identity + essential story summary |
| `mempalace split <dir>` | Split transcript files by session (`--min-sessions`, `--dry-run`) |
| `mempalace hook run --hook <name> --harness <harness>` | Execute hooks (`session-start`, `stop`, `precompact`) |
| `mempalace instructions <topic>` | Output context-aware help (`init`, `search`, `mine`, `help`, `status`) |
| `mempalace repair` | Rebuild palace from SQLite (backs up first, prompts unless `--yes`) |
| `mempalace repair-status` | Check HNSW/SQLite consistency (harmless "metadata not flushed" for tiny collections) |
| `mempalace migrate` | Schema migration between ChromaDB versions (`--dry-run`, `--yes`) |
| `mempalace mcp` | Print MCP server setup instructions |
| `mempalace status` | Palace health: drawer counts, wing/room breakdown |

**Not CLI commands** (MCP-only): `list-wings`, `list-rooms`, `get-taxonomy`, `add-drawer`, `delete-drawer`, `check-duplicate`, `reconnect` — these are MCP tools, not CLI subcommands.

## Storage location & scope

| Path | Contents | Status |
|---|---|---|
| `~/.mempalace/` | Palace database (ChromaDB + SQLite) | **Per-user.** Never committed; cross-project. |
| `~/.mempalace/config.json` | User config (topics, keywords) | Per-user. Note: `config.json`, not `config.yaml`. |
| `~/.mempalace/wal/` | Write-ahead log (JSONL audit trail) | Per-user. Restricted permissions (0o700). |
| In-repo state | _none_ | MemPalace does **not** write into the repo. |

Multiple repos share one palace by design — that's how agents remember
the cross-cutting context that spans repos (e.g., a decision made while
working on `weftos` also matters when you switch to `ruvector` next
week).

## When NOT to use MemPalace

- For **code-structure queries** ("where is X called from?", "rename Foo")
  — use **GitNexus**. MemPalace stores conversational memory, not AST.
- For **fresh-codebase comprehension** ("what does this crate do?") —
  use **Understand-Anything** `/understand-onboard <crate>`. MemPalace
  is empty on first run; it's only useful once the palace has history.
- For **secrets, credentials, PII**. MemPalace stores verbatim — never
  pass raw API keys, customer data, or anything that would harm someone
  if leaked locally.
- For **build/test/lint operations** — those go through `scripts/build.sh`
  per `CLAUDE.md`, not through MemPalace.

## Repo-specific notes

- **`gui/src-tauri/`** is excluded from the cargo workspace and from
  `.gitnexusignore`. `mempalace mine` walks the filesystem, so it WILL
  pick up the Tauri shell unless you pass `--exclude gui/src-tauri/`
  or maintain a `.mempalace_ignore` file. For most cross-repo agent
  work, including the Tauri shell is fine — it's a small fraction of
  the total drawer count.
- **`scripts/build.sh` mandate**: every build/test/check/lint operation
  goes through `scripts/build.sh`. MemPalace doesn't override this —
  it just remembers conversational context, not build state.
- **Naming taxonomy**: when filing drawers, prefer wing-name `weftos`
  even though crates are named `clawft-*`. Searches against `weftos`
  catch decisions about both layers; searches against `clawft-*` only
  catch crate-internal details.

## Smoke test

```bash
# After bootstrap (CLI installed, no palace yet):
mempalace --version       # confirms CLI is on PATH

# After you have run `mempalace init <some-dir>` once anywhere:
mempalace status          # confirms palace exists + connected
mempalace search "test"   # semantic search smoke test
mempalace repair-status   # check palace health (expect "HNSW metadata not flushed" for tiny collections — harmless)
```

If `status` / `search` error with `palace not initialized`, you haven't
run `mempalace init <dir>` yet — run it against a directory you're
willing to dirty (see Install above). The bootstrap script intentionally
skips this step.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `palace not initialized` | Never ran `mempalace init` | `mempalace init ~/.mempalace/projects/<name>` |
| `HNSW capacity unavailable: metadata has not been flushed` | Tiny collection (< 50K items), ChromaDB hasn't flushed metadata pickle | **Harmless.** Vector search is still enabled. Will resolve as collection grows. |
| Search returns only drawer with very low scores | Single drawer in palace, nothing else to rank against | Normal for tiny collections. Add more drawers or ignore low-score results. |
| `Error finding id` during search | HNSW flush window after bulk mine | Self-heals in 30-60s. Call `mempalace_reconnect` to force cache rebuild. |
| `mempalace-mcp` not found | MCP server module not installed | Reinstall: `uv tool install mempalace` or `pipx install mempalace` |

## License & provenance

MemPalace ships under MIT. Upstream:
- GitHub: https://github.com/MemPalace/mempalace
- PyPI:   https://pypi.org/project/mempalace/
- Docs:   https://mempalaceofficial.com/

**Do NOT install from `mempalace.tech`** — that's a known impostor
domain that distributes malware. The bootstrap script's install
subprocess pins `--index-url https://pypi.org/simple/`, clears
`PIP_EXTRA_INDEX_URL`, and ignores `pip.conf` / `pip.ini` so that no
ambient pip configuration can redirect resolution to a malicious
mirror.
