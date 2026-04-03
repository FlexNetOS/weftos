# Session Handoff: Sprint 14

**Date**: 2026-04-02
**Version**: v0.3.1 (released, all channels live)
**Branch**: master

---

## Current State

Sprints 11, 12, and 13 complete. All distribution channels live. Full WASI support.

**Completed work archived**: `.planning/development_notes/sprint-11-12/`

## What's Working

| Component | Status |
|-----------|--------|
| 22-crate Rust workspace | Compiles, 2,600+ tests |
| 8 native build targets | glibc x2, musl x2, macOS x2, Windows x86 + ARM (pending CI verify) |
| 2 WASM targets | wasip2 (10/10 crates), browser (@weftos/core) |
| crates.io | 10 crates (v0.2.1 with cross-links) |
| npm | @weftos/core 0.1.1 |
| Docker | ghcr.io/weave-logic-ai/weftos (distroless, multi-arch) |
| Homebrew | weave-logic-ai/homebrew-tap (3 formulae) |
| Docs | weftos.weavelogic.ai (Fumadocs + rustdoc API, auto-deploy on push) |
| GUI | Block engine (11 blocks), 4 themes, KernelDataProvider, ThemeSwitcher, BudgetBlock |
| Agent pipeline | GEPA end-to-end (config-based scorer/learner, mutation in skill_autogen) |
| LLM providers | 11 providers including local (Ollama/vLLM/llama.cpp/LM Studio) |
| Context compression | Sliding window, token counting, summarization (opt-in) |
| Paperclip patterns | Company/OrgChart types, HeartbeatScheduler, GoalTree, HTTP API |
| Testing | Property tests, fuzz harnesses, criterion benchmarks |
| Vercel | Auto-deploy configured (root dir: docs/src) |

## Skills

| Skill | Trigger |
|-------|---------|
| `weftos-build-deploy` | "release", "deploy", "publish" |
| `weftos-api-docs` | "rustdoc", "api docs" |
| `weftos-docs-deploy` | "deploy docs", "docs changed" |

---

## Sprint 13 Completed Items

### GUI Integration
- [x] Wire block engine to real kernel data (KernelDataProvider.tsx)
- [x] Theme persistence (localStorage + Tauri config)
- [x] ThemeSwitcher dropdown component
- [x] BudgetBlock: per-agent cost tracking dashboard

### Agent Pipeline Integration
- [x] Wire TrajectoryLearner into agent loop (factory in bootstrap.rs)
- [x] Wire FitnessScorer into pipeline (factory in llm_adapter.rs)
- [x] Config-based learner/scorer selection (PipelineConfig)
- [x] Prompt mutation integration with skill_autogen.rs

### Paperclip Patterns (Rust-native)
- [x] Company/OrgChart types (12 tests)
- [x] HeartbeatScheduler (19 tests)
- [x] GoalTree (13 tests)

### Paperclip Adapter
- [x] HTTP API handler (execute, govern, health) behind http-api feature (12 tests)
- [x] Paperclip adapter spec (docs/specs/paperclip-adapter-spec.md)
- [x] Governance bridge design (effect vectors, ExoChain logging)

### Testing
- [x] Fix kernel test compilation (Debug impls on CausalGraph, HnswService)
- [x] Fix pre-existing test failure (democritus embedding_error assertion)
- [x] Accept updated config snapshot (PipelineConfig field)
- [x] Property tests: 11 randomized tests (scorer bounds, mutation safety, token counting)
- [x] Fuzz tests: 9 tests (Config/RoutingConfig with random/malformed JSON)
- [x] Criterion benchmarks: 4 groups (count_tokens, compress_context, scorer, mutation)
- [x] Integration tests: 6 context compression tests

### Platform
- [x] Fix clawft-core for wasip2 (bus.rs browser→explicit cfg + no-op fallback)
- [x] Full WASI: clawft-kernel + weftos compile for wasm32-wasip2 (10/10 crates)
- [x] Switch reqwest from rustls-tls to native-tls (removes ring dependency path)
- [x] Re-enable aarch64-pc-windows-msvc target (pending CI verify on next tag)
- [x] Vercel auto-deploy configured (root dir: docs/src)

---

## v0.3.1 Release (2026-04-02)

- [x] Fixed cargo-dist: dropped `aarch64-pc-windows-msvc` (ring/cargo-xwin broken)
- [x] Fixed docs site: added `turbopack.root` in next.config.mjs
- [x] Fixed Vercel: moved domain from "src" project to "clawft", set Root Directory to `docs/src`
- [x] Published to all channels: GitHub Release, crates.io (8 crates), Docker, WASI, Homebrew
- [x] Updated platform badge from 9 → 7

---

## Sprint 14 — Marketing + Playground + SOPs

### Work Streams (parallel via git worktrees)

#### One-Liner Install Paths (highlight on landing page + quickstart)

```bash
# Shell (Linux/macOS) — downloads pre-built binary
curl -fsSL https://github.com/weave-logic-ai/weftos/releases/latest/download/clawft-cli-installer.sh | sh

# Homebrew
brew install weave-logic-ai/tap/clawft-cli

# Docker (no install needed)
docker run --rm -it ghcr.io/weave-logic-ai/weftos:latest weft --help

# Cargo (from source)
cargo install weftos

# PowerShell (Windows)
irm https://github.com/weave-logic-ai/weftos/releases/latest/download/clawft-cli-installer.ps1 | iex
```

All install `weft` on PATH. Show expected output after each.

---

#### WS1: Docs Site Rewrite (weftos.weavelogic.ai)
SPARC plan: `.planning/weftos.weavelogic.ai/01-05`

- [x] Fix `<title>` from "clawft" to "WeftOS" (`layout.tsx`)
- [x] Rewrite hero section — benefit-driven headline + CTAs (`page.tsx`)
- [x] Rewrite layer cards — benefits not features (`page.tsx`)
- [x] Rewrite feature highlights — no jargon (`page.tsx`)
- [x] Add footer with GitHub, license, WeaveLogic link (`layout.tsx`)
- [x] Move badges below the fold (`page.tsx`)
- [x] Create /docs index page — currently 404 (`content/docs/index.mdx`)
- [x] Add previous/next page navigation (Fumadocs config)
- [x] Add "Edit on GitHub" links (Fumadocs config)
- [x] Create glossary page (`content/docs/glossary.mdx`)
- [x] Rewrite getting-started — three-tier quickstart (Docker/curl/source)
- [x] Add expected output blocks to all command examples
- [x] Add troubleshooting section to getting-started

#### WS2: Interactive Playground
Plan: `.planning/weftos.weavelogic.ai/06-interactive-playground-plan.md`

- [x] Build `build-kb` tool (Rust binary: MDX → chunk → embed → RVF)
- [x] Generate weftos-docs.rvf from 68 doc pages (1,160 segments, 384-dim)
- [x] Build browser WASM (`scripts/build.sh browser`)
- [x] WASM + KB served via CDN (GitHub Releases cdn-assets tag) + Vercel proxy
- [x] Create `WasmSandbox.tsx` component at `/clawft/` route (load WASM, load KB, chat UI)
- [x] RAG prompt with keyword search, acronym awareness, hallucination guards
- [x] CDN proxy via Vercel API route (`app/api/cdn/[...path]/route.ts`) with edge caching
- [x] "Try in Browser" CTA on landing page + "Sandbox" link in docs nav
- [x] Local mode — works without API key (pure KB retrieval, no LLM needed)
- [x] ExoChain log panel — two-column layout with live audit trail
- [x] Runtime introspection — users can inspect their own WASM instance
- [x] "New chat" reset button

#### WS3: SOPs + Assessment Foundation
Plan: `docs/guides/weftos-deployment-sops.md` (45KB, 1,226 lines)

- [x] `weft assess` CLI command (run, status, init, link, peers, compare)
- [ ] AI Assessor agent — move to kernel service (blocked by WS5)
- [x] Validate SOP 1 against clawft project (2,986 files, 412K LOC assessed)
- [x] Initialize WeftOS on weavelogic.ai project (2,549 files, 801K LOC assessed)
- [x] Cross-project coordination — link/peers/compare with local + HTTP URL support
- [ ] Run SOP 2 Phase 1-2 (tree-sitter + git mining) — needs kernel agent spawning
- [x] Assessment docs published: /docs/weftos/guides/assessment + deployment-sops

#### WS5: CLI Kernel Compliance — All Commands Must Go Through Daemon

**Rule:** Every CLI command that performs operations (file scanning, analysis, state changes,
agent spawning, network calls) MUST go through the kernel daemon via RPC. The CLI is a thin
client. Only pure-local display (help, completions, config show) is exempt.

**Security rationale:** Bypassing the kernel means bypassing governance gates, capability
checks, ExoChain audit logging, and sandboxing. A `weft security scan` that reads files
directly has no audit trail, no capability restriction, and no governance oversight.

**weaver (clawft-weave):** 51/52 commands PASS — only `weaver init` bypasses (runs shell script).

**weft (clawft-cli):** 29/61 PASS, **32 commands BYPASS the kernel.** Offenders:

| Category | Offending Commands | Issue |
|----------|-------------------|-------|
| **Cron** | `list`, `add`, `remove`, `enable`, `disable`, `run` | Directly reads/writes cron.jsonl — race conditions, no audit |
| **Assess** | `run`, `init`, `link`, `compare` | Scans filesystem directly, no governance, no ExoChain logging |
| **Security** | `scan` | Reads files directly, no capability check, no audit trail |
| **Skills** | `list`, `show`, `install`, `remove`, `search`, `publish`, `remote-install` | Direct filesystem + network ops — no daemon gateway |
| **Tools** | `list`, `show`, `mcp`, `search`, `deny`, `allow` | Direct registry access — no live kernel state |
| **Agents** | `list`, `show`, `use` | Direct filesystem — no daemon agent registry |
| **Workspace** | `create`, `list`, `load`, `status`, `delete`, `config set/get/reset` | Direct filesystem — no coordination |
| **Other** | `onboard`, `ui`, `voice setup/test/talk`, `mcp-server` | Service management done CLI-side, not by kernel |

**Remediation plan:**
1. Add `DaemonClient` to clawft-cli (port from clawft-weave or extract to shared crate)
2. For each bypassing command, add daemon RPC endpoint in kernel + CLI thin client
3. Support graceful fallback: if no daemon running, print error with `weaver kernel start` hint
4. Priority order: assess → security → cron → skills → tools → agents → workspace → other
5. Each command migration creates ExoChain audit events for the operations it performs

**Exceptions to evaluate:**
- `weft onboard` — bootstrap problem (no daemon exists yet to talk to)
- `weft mcp-server` — runs as a server itself, not a client
- `weft agent` / `weft gateway` — these bootstrap their own AppContext (by design?)
- Pure display commands (status, config show, help) — no state changes, possibly exempt

#### WS4: weavelogic.ai Rewrite (separate project)
Plan: `weavelogic.ai/docs/planning/rewrite/15-marketing-review-2026-04-02.md`

*Worked on separately — see weavelogic.ai project*

- [ ] Remaining TODOs: /about, /contact Calendly, ROI calculator, sitemaps, PostHog
- [ ] Restructure /services as post-assessment flow
- [ ] Consolidate CTAs to 2 variants
- [ ] Fix/remove empty blog

### Sprint 14 Planning Docs

| Document | Location |
|----------|----------|
| Marketing review (weavelogic.ai) | `weavelogic.ai/docs/planning/rewrite/15-marketing-review-2026-04-02.md` |
| Marketing review (weftos) | `.planning/weftos.weavelogic.ai/00-marketing-review-2026-04-02.md` |
| SPARC Specification | `.planning/weftos.weavelogic.ai/01-sparc-specification.md` |
| SPARC Pseudocode | `.planning/weftos.weavelogic.ai/02-sparc-pseudocode.md` |
| SPARC Architecture | `.planning/weftos.weavelogic.ai/03-sparc-architecture.md` |
| SPARC Refinement | `.planning/weftos.weavelogic.ai/04-sparc-refinement.md` |
| SPARC Completion | `.planning/weftos.weavelogic.ai/05-sparc-completion.md` |
| Playground plan | `.planning/weftos.weavelogic.ai/06-interactive-playground-plan.md` |
| RVF knowledge base plan | `.planning/weftos.weavelogic.ai/07-rvf-knowledge-base-plan.md` |
| Learner model plan | `.planning/weftos.weavelogic.ai/08-learner-model-plan.md` |
| Assessment knowledge model | `.planning/weftos.weavelogic.ai/09-assessment-knowledge-model.md` |
| Kolbe conative integration | `docs/research/kolbe-conative-integration.md` (790 lines) |
| Deployment SOPs | `docs/guides/weftos-deployment-sops.md` |

---

## Deferred to Sprint 15+
- [ ] Block drag-and-drop layout editing (large GUI scope)
- [ ] Playground set pieces (security bug, provider race, governance wall, etc.)
- [ ] Playground Phase 2-4 (provenance panel, knowledge graph viz, governance panel)
- [ ] `clawft-plugin-npm` (Node.js dependency parsing)
- [ ] `clawft-plugin-ci` (GitHub Actions / Vercel config parsing)
- [ ] Rustdoc JSON-to-MDX converter for native Fumadocs API pages
- [ ] Plugin marketplace + create-weftos-plugin scaffolding
- [ ] Multi-company namespace isolation
- [ ] Security audit (1.0 gate)
- [ ] Cross-project mesh coordination (SOP 3)
- [ ] Continuous assessment pipeline (SOP 4)
- [ ] SOP improvement loop (SOP 5)



---

## Key References

| Resource | Location |
|----------|----------|
| Paperclip research | `docs/research/paperclip-integration-analysis.md` |
| Paperclip adapter spec | `docs/specs/paperclip-adapter-spec.md` |
| Release strategy | `.planning/sparc/weftos/0.1/11-release-strategy.md` |
| GUI design spec | `.planning/sparc/weftos/0.1/weftos-gui-design-notes.md` |
| Theming spec | `docs/weftos/specs/theming-system.md` |
| GEPA research | `docs/research/gepa-prompt-evolution-analysis.md` |
| Block descriptor | `docs/weftos/specs/block-descriptor-schema.json` |
| Web rewrite plan | `weavelogic.ai/docs/planning/rewrite/` |
| Build-deploy skill | `.claude/skills/weftos-build-deploy/SKILL.md` |
| Completed sprints | `.planning/development_notes/sprint-11-12/` |
