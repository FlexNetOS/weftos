# Session Handoff: Sprint 16

**Date**: 2026-04-05
**Version**: v0.5.0 (released, all channels deploying)
**Branch**: master

---

## Current State

Sprint 16 architecture sprint complete. v0.5.0 released with wasmtime v33, security audit, ServiceApi, wasip2, governance UI, mesh K6 transport, and browser WASM features. 25 crates, 48 ADRs, 1,248+ tests passing.

**Previous sprints archived**: `.planning/development_notes/sprint-13-14/`, `.planning/development_notes/sprint-16/`

## What's Working

| Component | Status |
|-----------|--------|
| 25-crate Rust workspace | Compiles, 1,248+ tests passing |
| 7 native build targets | Linux (glibc x2, musl x2), macOS x2, Windows x86 |
| 2 WASM targets | wasip2 (migrated from wasip1), browser |
| WASM sandbox | /clawft/ — local + LLM mode, governance panel, agent spawning, GitHub repo analysis |
| Assessment | 8 pluggable analyzers + LLM assessor + assessment diff + browser WASM engine |
| Cross-project mesh | K6 real-time transport with AssessmentSync frames, gossip, TCP integration |
| ServiceApi | KernelServiceApi concrete impl — uniform interface for GUI/CLI/mesh/WASM |
| Security | 1.0 audit passed — 0 critical, 2 high (fixed), 5 medium (documented) |
| CLI compliance | 32 commands use daemon-first RPC |
| 48 ADRs | docs/adr/adr-001 through adr-047 |
| Docs site | weftos.weavelogic.ai — 77+ pages, Sprint 14-15 coverage pass complete |
| All distribution channels | GitHub Releases, crates.io (12 crates), npm, Docker (Alpine), Homebrew, WASI |
| Dependabot | 9 alerts (down from 16) — wasmtime upgrade closed several |
| Docker | Alpine 3.21 + pre-built musl binaries — ~2min build, ~15MB image |
| `ui/` renamed | `clawft-ui/` (agent web dashboard) vs `gui/` (WeftOS kernel desktop) |

---

## Sprint 16 — Completed Work

### v0.4.2 (2026-04-04)
- Full ExoChain boot sequence in browser sandbox (INIT → CONFIG → SERVICES → NETWORK → READY)
- Future feature items added to backlog (browser assessment engine, client session persistence)

### v0.4.3 (2026-04-04)
- Documentation coverage pass: assessment (8 analyzers), GUI (Tauri commands, 12 blocks), browser sandbox (modes, providers, guided tour), plugins (npm/CI), releases (v0.4.2 entry)
- Docker optimization: Alpine + pre-built binaries (~30min → ~2min, ~50MB → ~15MB)
- crates.io pipeline fixed: 12 crates publish on tag (added clawft-services, clawft-plugin-treesitter)
- "already exists" error handling fixed in publish workflow

### v0.5.0 (2026-04-05) — Architecture Sprint
Six parallel workstreams:

1. **wasmtime v29 → v33** — 2 import path changes, closes Dependabot alerts
2. **Security audit (1.0 gate)** — Ed25519 key file permissions, TokenStore lock panic fixed
3. **ServiceApi (ADR-035) + wasip2 (ADR-044)** — KernelServiceApi impl + 9 tests, wasip1→wasip2 across 9 files
4. **Playground Phase 3-4** — Governance panel (22 genesis rules, 3-branch, EffectVector), agent spawning UI (lifecycle animations, max 8)
5. **Mesh K6 transport (SOP 3)** — AssessmentTransport adapter, FrameType::AssessmentSync, gossip/broadcast/request, 17 tests
6. **Browser WASM features** — session persistence (IndexedDB), GitHub repo analysis engine (analyze_files wasm_bindgen), real boot log (boot_info wasm_bindgen)

### Housekeeping
- Renamed `ui/` → `clawft-ui/` (agent dashboard vs kernel desktop — not redundant)
- Closed stale PR #1, deleted feature/three-workstream-implementation branch
- Fixed version_output test (was hardcoded to "0.1.0")
- Updated service count assertions for new ServiceApi registration

---

## Sprint 16 — Remaining

### weavelogic.ai site (WS6)
- [ ] ROI calculator
- [ ] /about, /contact with Calendly
- [ ] Sitemaps, PostHog analytics
- [ ] Restructure /services as post-assessment flow
- [ ] Consolidate CTAs to 2 variants

### Deferred (post-launch)
- [ ] Block drag-and-drop layout editing (GUI)
- [ ] Post-quantum key exchange implementation (ADR-028 Phase 2)
- [ ] BLAKE3 hash migration from SHAKE-256 (ADR-043)
- [ ] N-dimensional EffectVector refactor (ADR-034 C9)
- [ ] ChainAnchor blockchain integration (ADR-041)

---

## Known Issues

| Issue | Severity | Notes |
|-------|----------|-------|
| 9 Dependabot alerts | Medium | 7 moderate, 2 low — down from 16 after wasmtime upgrade |
| Security medium findings | Medium | Disabled auth middleware, permissive CORS, no rate limiting, no token revocation, no chain replay protection — see `.planning/development_notes/sprint-16/security-audit.md` |
| clawft-ui/ not updated | Low | Renamed from ui/ but no feature work since Sprint 10 |
| Assessment service stubs | Low | Daemon RPC returns acknowledgments, not full results yet |

---

## Key References

| Resource | Location |
|----------|----------|
| ADR catalog (48) | `docs/adr/adr-001 through adr-047` + `docs/adr/PROPOSED.md` |
| ADR causal graph | `.weftos/memory/adr-graph.json` |
| Deployment SOPs | `docs/guides/weftos-deployment-sops.md` |
| Sprint 13-14 archive | `.planning/development_notes/sprint-13-14/` |
| Sprint 16 dev notes | `.planning/development_notes/sprint-16/` (6 files) |
| WASM sandbox | `docs/src/app/clawft/WasmSandbox.tsx` |
| Assessment service | `crates/clawft-kernel/src/assessment/` |
| Mesh transport | `crates/clawft-kernel/src/mesh_assess.rs` |
| ServiceApi | `crates/clawft-kernel/src/service.rs` (KernelServiceApi) |
| Security audit | `.planning/development_notes/sprint-16/security-audit.md` |
| Session store | `docs/src/lib/session-store.ts` |
| RPC crate | `crates/clawft-rpc/` |
| Release notes | `CHANGELOG.md`, `docs/src/content/docs/weftos/vision/releases.mdx` |
| Build script | `scripts/build.sh` |
| Pull assets (dev) | `scripts/pull-assets.sh` |
