# Browser WASM Features — Sprint 16

## Overview

Three features added to the browser WASM sandbox:

1. **Client Session/Config Persistence** (`docs/src/lib/session-store.ts`)
2. **Browser WASM Assessment Engine** (Rust + TypeScript)
3. **Real ExoChain Boot Log** (Rust `boot_info()` + TypeScript consumer)

---

## Feature 1: Session Store

**File**: `docs/src/lib/session-store.ts`

Storage split by size:
- **IndexedDB** (`clawft-session` database): conversation history, assessment results
- **localStorage** (`clawft-preferences` key): theme, panel visibility, model, KB graph state

API surface:
- `loadPreferences()` / `savePreferences(partial)` — localStorage, synchronous-safe
- `saveConversation(messages)` / `loadConversation()` / `clearConversation()` — IndexedDB async
- `saveAssessment(result)` / `loadAssessments()` / `loadAssessment(id)` / `deleteAssessment(id)` — IndexedDB async

Integration in `WasmSandbox.tsx`:
- Conversation auto-saved on every message change via `useEffect`
- Conversation restored on mount
- Preferences loaded on mount (model, kbGraphExpanded)
- Preferences saved when model changes during LLM connect
- `handleReset` calls `clearConversation()`

---

## Feature 2: Browser Assessment Engine

### Rust side (`crates/clawft-wasm/src/lib.rs`)

New export: `analyze_files(files_json: &str) -> String`

Takes JSON array of `{path, content}` objects. Returns JSON with:
- `summary`: `{file_count, total_lines, languages: [{language, files, lines}]}`
- `findings`: `[{severity, category, file, line?, message}]`

Analysis performed (mirrors native kernel analyzers):
- **ComplexityAnalyzer**: files >500 lines, TODO/FIXME/HACK markers
- **SecurityAnalyzer**: .env file detection, hardcoded secret patterns (14 patterns)
- **DependencyAnalyzer**: Cargo.toml dep counting, package.json section counting
- **TopologyAnalyzer**: Dockerfile detection (base image extraction), docker-compose, k8s manifests

Language detection: 20+ extensions mapped to language names.

### TypeScript side (`WasmSandbox.tsx`)

GitHub fetcher functions (no auth, 60 req/hr public limit):
- `parseGitHubUrl(input)` — accepts `github.com/owner/repo`, `https://github.com/owner/repo`, `owner/repo`
- `fetchRepoTree(owner, repo)` — uses `/git/trees/{sha}?recursive=1`
- `fetchRepoFiles(owner, repo, tree, onProgress)` — prioritizes config files, limits to 30 fetches, 100KB max per file

UI additions:
- Input field + "Assess Repo" button below the chat input
- Assessment results rendered as markdown in the chat area
- Each phase logged to ExoChain: `GITHUB_FETCH`, `TREE_SCAN`, `ANALYZE`, `FINDINGS`
- Results persisted to IndexedDB via `saveAssessment()`

### Rate limit awareness

The public GitHub API allows 60 requests/hour without auth. The fetcher:
- Uses recursive tree endpoint (1 request for full tree)
- Limits file fetches to 30 max
- Prioritizes config/manifest files (Cargo.toml, package.json, Dockerfile)
- Skips files >100KB

---

## Feature 3: Real ExoChain Boot Log

### Rust side (`crates/clawft-wasm/src/lib.rs`)

New export: `boot_info() -> String`

Returns JSON array of `{phase, detail}` objects mirroring the native kernel `BootLog`:
- INIT: version, PID
- CONFIG: platform, max processes, memory model
- SERVICES: registry, IPC, ExoChain
- NETWORK: LLM transport
- READY: final status

Uses `crate::VERSION` so the version stays in sync with Cargo.toml.

### TypeScript side (`WasmSandbox.tsx`)

Boot sequence in the `useEffect` mount handler now:
1. Attempts to load WASM and call `boot_info()`
2. If successful, iterates phases and logs each as `BOOT_{phase}`
3. If WASM unavailable, falls back to hardcoded entries

This means the ExoChain log shows real version numbers and phase data from the Rust kernel.

---

## Files Modified

| File | Change |
|------|--------|
| `docs/src/lib/session-store.ts` | NEW — IndexedDB + localStorage persistence |
| `docs/src/app/clawft/WasmSandbox.tsx` | Session persistence, GitHub assessment UI, boot_info() integration |
| `crates/clawft-wasm/src/lib.rs` | `boot_info()` and `analyze_files()` wasm_bindgen exports |

## Build Verification

- `scripts/build.sh check` passes (workspace-wide cargo check)
- No new crate dependencies required (uses existing `serde`, `serde_json`, `wasm-bindgen`)
