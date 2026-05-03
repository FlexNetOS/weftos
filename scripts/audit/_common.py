"""Shared constants for the audit scripts.

Both ``detect_dups.py`` (duplicate / near-duplicate scanning) and
``consolidate.py`` (stub-rewriting + incoming-link warnings) walk the same
markdown tree, so they MUST agree on which directories to skip — otherwise
``consolidate.py`` walks into ``.cargo`` / ``.pnpm-store`` / ``.planning``
caches that ``detect_dups.py`` skipped, scanning thousands of irrelevant
files and producing spurious "incoming link" warnings.
"""
from __future__ import annotations

# Directories the audit scripts skip when walking a repo:
#   - ".git", "node_modules", "dist", "build" — universal cache/output dirs
#   - "target" — Rust target/
#   - ".cargo", ".pnpm-store" — package-manager caches
#   - ".next", ".turbo" — Next.js / Turbo caches
#   - ".planning" — weftos session-scratch (not knowledge)
EXCLUDE_DIRS: frozenset[str] = frozenset({
    ".git",
    "node_modules",
    "target",
    ".cargo",
    "dist",
    "build",
    ".next",
    ".turbo",
    ".pnpm-store",
    ".planning",
})
