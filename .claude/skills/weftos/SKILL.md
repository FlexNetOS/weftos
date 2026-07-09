```markdown
# weftos Development Patterns

> Auto-generated skill from repository analysis

## Overview

This skill teaches you the core development patterns, coding conventions, and collaborative workflows used in the `weftos` Rust codebase. The repository is organized into modular crates and follows a clear, conventional commit style. It features a strong planning and documentation culture, robust backend and GUI extension workflows, and a focus on cross-platform reliability. The codebase is primarily Rust, with some TypeScript for testing and auxiliary tooling.

---

## Coding Conventions

### File Naming

- **Rust source files:** Use `camelCase`
  - Example: `substrateService.rs`, `nodeRegistry.rs`
- **Test files (TypeScript):** Use `*.test.ts`
  - Example: `substrateService.test.ts`

### Import Style

- **Relative imports** are preferred.
  - Example:
    ```rust
    mod nodeRegistry;
    use crate::substrateService::SubstrateService;
    ```

### Export Style

- **Named exports** are used for modules and types.
  - Example:
    ```rust
    pub struct SubstrateService { /* ... */ }
    pub fn register_node() { /* ... */ }
    ```

### Commit Messages

- **Conventional commits** with prefixes: `feat`, `docs`, `fix`, `merge`, `refactor`
- Typical format:
  ```
  feat: add substrate.list RPC and wire through daemon (closes #123)
  ```

---

## Workflows

### Planning Track Initiation

**Trigger:** When a new feature, phase, or architectural direction is being planned and needs documentation before code lands.  
**Command:** `/new-planning-track`

1. Create or update `.planning/*/*.md` files with detailed plans, research, or sequencing.
2. Commit multiple related planning docs together (e.g., plan, adoption, spike, journal).
3. Reference these docs in subsequent implementation commits.

**Example:**
```shell
git add .planning/explorer/PROJECT-PLAN.md .planning/explorer/PHASE-2-PLAN.md
git commit -m "docs: initial planning docs for Explorer phase 2"
```

---

### Backend RPC Surface Extension

**Trigger:** When a new backend capability or API endpoint is needed for GUI or firmware integration.  
**Command:** `/add-backend-rpc`

1. Implement new method in kernel/service (e.g., `substrate_service.rs`, `node_registry.rs`, `control.rs`).
2. Add protocol types and wire handler in `daemon.rs`.
3. Update `protocol.rs` with params/result structs.
4. Add or update integration/unit tests (`tests/*.rs`).
5. Optionally update GUI or VSCode extension to use new RPC.

**Example:**
```rust
// crates/clawft-kernel/src/substrate_service.rs
pub fn list_substrates() -> Vec<Substrate> { /* ... */ }

// crates/clawft-weave/src/protocol.rs
pub struct SubstrateListParams { /* ... */ }
pub struct SubstrateListResult { /* ... */ }
```

---

### GUI Explorer Viewer Addition

**Trigger:** When a new substrate data shape needs a specialized GUI representation.  
**Command:** `/add-explorer-viewer`

1. Create a new viewer module (`src/explorer/viewers/*.rs`) implementing `SubstrateViewer`.
2. Register the viewer in `viewers/mod.rs` using marker comments (`[[VIEWERS_MODULES_INSERT]]`, `[[VIEWERS_REGISTRATIONS_INSERT]]`).
3. Write unit tests for `matches()`, edge cases, and rendering.
4. Update `mod.rs` and `lib.rs` as needed.
5. Optionally update integration tests.

**Example:**
```rust
// crates/clawft-gui-egui/src/explorer/viewers/myShapeViewer.rs
pub struct MyShapeViewer;
impl SubstrateViewer for MyShapeViewer { /* ... */ }
```

---

### GUI Explorer Panel Extension

**Trigger:** When a new composition primitive or control surface is needed in the Explorer.  
**Command:** `/add-explorer-primitive`

1. Create new module(s) under `src/explorer/` (e.g., `workshop.rs`, `control_toggle.rs`).
2. Update `explorer/mod.rs` to mount the new primitive.
3. Write integration/unit tests for new UI logic.
4. Optionally add example TOML/specs for dev tools.
5. Update GUI shell (`desktop.rs`) if new entry points are needed.

**Example:**
```rust
// crates/clawft-gui-egui/src/explorer/workshop.rs
pub struct Workshop { /* ... */ }
```

---

### Merge Feature Track

**Trigger:** When a feature branch is ready to land after review and possibly conflict resolution.  
**Command:** `/merge-feature-track`

1. Merge feature branch into main/dev (commit message: `merge: phaseX-...`).
2. Resolve conflicts, especially in `mod.rs` or `viewers/mod.rs`.
3. Ensure all related files (modules, tests, protocol) are included.
4. Workspace clippy/test sweep before merge.
5. Document in handoff or planning docs as needed.

**Example:**
```shell
git checkout main
git merge feature/explorer-backend
# Resolve conflicts, run tests, then:
git commit -m "merge: phase2-explorer-backend"
```

---

### Fix Cross-Platform or Performance Bug

**Trigger:** When a platform-specific or performance regression is reported and needs a targeted fix.  
**Command:** `/fix-platform-bug`

1. Identify and isolate the bug (often with a minimal repro in `examples/`).
2. Patch the affected module (e.g., `response.rs`, `json_fallback.rs`).
3. Add or update example/repro files for regression testing.
4. Document the fix in commit message and/or handoff docs.
5. Run platform-specific tests/checks.

**Example:**
```rust
// crates/clawft-gui-egui/src/canon/response.rs
// Fix for Windows deadlock
```

---

### Handoff or Journal Update

**Trigger:** When major merges land or phase transitions occur, or after bugfixes that require operational notes.  
**Command:** `/update-handoff`

1. Edit `docs/handoff.md` or `.planning/sensors/PIPELINE-PRIMITIVE-JOURNAL.md` with details of recent changes.
2. Summarize what landed, what is pending, and known issues.
3. Commit updated doc(s) as a standalone or follow-up to merges/fixes.

**Example:**
```shell
git add docs/handoff.md
git commit -m "docs: update handoff after phase2 merge"
```

---

## Testing Patterns

- **Framework:** [Playwright](https://playwright.dev/) (for TypeScript-based integration/UI tests)
- **File pattern:** `*.test.ts`
- **Rust tests:** Standard Rust unit/integration tests in `tests/*.rs`
- **Test Example (TypeScript):**
    ```typescript
    // substrateService.test.ts
    import { test, expect } from '@playwright/test';

    test('should list substrates', async ({ page }) => {
      // test logic here
    });
    ```
- **Test Example (Rust):**
    ```rust
    #[test]
    fn test_substrate_list() {
        let result = list_substrates();
        assert!(!result.is_empty());
    }
    ```

---

## Commands

| Command                | Purpose                                                        |
|------------------------|----------------------------------------------------------------|
| /new-planning-track    | Start a new planning/architecture track                        |
| /add-backend-rpc       | Add a new backend RPC method and wire through the stack        |
| /add-explorer-viewer   | Add a new shape-matched viewer to the Explorer panel           |
| /add-explorer-primitive| Extend the Explorer panel with new UI primitives               |
| /merge-feature-track   | Merge a major feature branch/track after review                |
| /fix-platform-bug      | Fix a cross-platform or performance bug                        |
| /update-handoff        | Update handoff or journal documentation                        |
```
