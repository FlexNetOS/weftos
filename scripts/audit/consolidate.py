#!/usr/bin/env python3
"""Replace exact-duplicate doc files with redirect stubs.

For each (canonical, duplicate) pair, the duplicate file is overwritten
with a 4-line stub linking to the canonical file. Original content is
*not* deleted from history (it remains in `git log`) and is reproduced
verbatim in `docs/audit/AUDIT_REPORT.md` so no information is lost.
"""
from __future__ import annotations

import os
import re
import sys
from pathlib import Path

from _common import EXCLUDE_DIRS


def stub_for(canonical_rel_to_root: str, repo_label: str) -> str:
    return (
        f"<!-- consolidated -->\n"
        f"# Moved\n\n"
        f"This document was consolidated during the {repo_label} knowledge-base\n"
        f"audit (see `docs/audit/AUDIT_REPORT.md`).\n\n"
        f"**Canonical location:** [`{canonical_rel_to_root}`](/{canonical_rel_to_root})\n\n"
        f"All previous content is preserved in git history and at the canonical\n"
        f"location. If you believe consolidation was incorrect, see\n"
        f"`docs/audit/MANUAL_REVIEW.md`.\n"
    )


def _iter_md_files(root: Path) -> list[Path]:
    out: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in EXCLUDE_DIRS]
        for fn in filenames:
            if fn.lower().endswith(".md"):
                out.append(Path(dirpath) / fn)
    return out


# Markdown-style link: `](href)`. Captured group is the href.
_MD_LINK_RE = re.compile(r"\]\(([^)#?\s]+)")
# Backtick-quoted .md path reference, e.g. `` `agents/weftos-ecc/WEAVER.md` ``.
# We require the path to end in .md so plain inline-code spans like
# `_lock` or `serde_json` don't match.
_BACKTICK_MD_RE = re.compile(r"`([^`\s]+\.md)`")


def _iter_link_hrefs(line: str) -> list[tuple[str, str]]:
    """Yield ``(kind, href)`` for markdown links and backtick .md refs on a line.

    ``kind`` is ``"md"`` for ``](href)`` matches and ``"bt"`` for backtick
    refs. The two are resolved differently: markdown links are relative to
    the linking file (or repo-root if ``/``-prefixed), while backtick refs
    are typically repo-root paths used as prose labels and should be resolved
    that way first.
    """
    out: list[tuple[str, str]] = []
    out.extend(("md", h) for h in _MD_LINK_RE.findall(line))
    out.extend(("bt", h) for h in _BACKTICK_MD_RE.findall(line))
    return out


def _resolve_candidates(root: Path, md: Path, kind: str, href: str) -> list[Path]:
    """Return possible filesystem resolutions of ``href`` from file ``md``."""
    if href.startswith(("http://", "https://", "mailto:")):
        return []
    candidates: list[Path] = []
    try:
        if href.startswith("/"):
            candidates.append((root / href.lstrip("/")).resolve())
        elif kind == "bt":
            # Backtick refs are usually repo-root paths in prose. Try root
            # first, then fall back to relative-to-file in case the author
            # actually meant a sibling path.
            candidates.append((root / href).resolve())
            candidates.append((md.parent / href).resolve())
        else:
            candidates.append((md.parent / href).resolve())
    except Exception:
        return []
    return candidates


def find_incoming_links(root: Path, target: Path) -> list[tuple[Path, int, str]]:
    """Return [(file, line_number, link_text)] of references pointing at *target*.

    Detects both markdown link syntax ``](href)`` and backtick-quoted .md
    path references ``` `path/to/x.md` ```. Backtick refs are resolved as
    repo-root paths first (the dominant convention in our docs), then as
    paths relative to the linking file's directory.
    """
    root = root.resolve()
    target = target.resolve()
    hits: list[tuple[Path, int, str]] = []
    for md in _iter_md_files(root):
        if md.resolve() == target:
            continue
        try:
            for lineno, line in enumerate(md.read_text(errors="replace").splitlines(), 1):
                for kind, href in _iter_link_hrefs(line):
                    for resolved in _resolve_candidates(root, md, kind, href):
                        if resolved == target:
                            hits.append((md, lineno, line.strip()))
                            break
        except Exception:
            continue
    return hits


def consolidate(root: Path, label: str, pairs: list[tuple[str, str]]) -> None:
    for canonical, duplicate in pairs:
        c = root / canonical
        d = root / duplicate
        if not c.exists():
            print(f"  SKIP missing canonical: {canonical}")
            continue
        if not d.exists():
            print(f"  SKIP missing duplicate: {duplicate}")
            continue
        # Warn about incoming markdown links so the operator can repoint them
        # before / after stubbing. We do not auto-rewrite to avoid silently
        # touching files outside the duplicate set.
        incoming = find_incoming_links(root, d)
        if incoming:
            print(f"  WARN {duplicate} has {len(incoming)} incoming markdown link(s):")
            for src, lineno, snippet in incoming:
                print(f"        - {src.relative_to(root)}:{lineno}: {snippet}")
            print(f"        (repoint them at {canonical} after consolidation)")
        d.write_text(stub_for(canonical, label))
        print(f"  stub  {duplicate} -> {canonical}")


RUVECTOR_PAIRS = [
    # examples/dragnes/docs/* mirrors docs/research/DrAgnes/*
    ("docs/research/DrAgnes/architecture.md",       "examples/dragnes/docs/architecture.md"),
    ("docs/research/DrAgnes/competitive-analysis.md","examples/dragnes/docs/competitive-analysis.md"),
    ("docs/research/DrAgnes/data-sources.md",       "examples/dragnes/docs/data-sources.md"),
    ("docs/research/DrAgnes/deployment.md",         "examples/dragnes/docs/deployment.md"),
    ("docs/research/DrAgnes/dermlite-integration.md","examples/dragnes/docs/dermlite-integration.md"),
    ("docs/research/DrAgnes/future-vision.md",      "examples/dragnes/docs/future-vision.md"),
    ("docs/research/DrAgnes/HAM10000_analysis.md",  "examples/dragnes/docs/HAM10000_analysis.md"),
    ("docs/research/DrAgnes/hipaa-compliance.md",   "examples/dragnes/docs/hipaa-compliance.md"),
    ("docs/research/DrAgnes/README.md",             "examples/dragnes/docs/README.md"),
    # patches/hnsw_rs duplicated under scripts/patches
    ("patches/hnsw_rs/README.md",  "scripts/patches/hnsw_rs/README.md"),
    ("patches/hnsw_rs/Changes.md", "scripts/patches/hnsw_rs/Changes.md"),
    # research paper mirrored in example
    ("docs/research/cognitive-frontier/delta-behavior-computational-paradigm.md",
     "examples/delta-behavior/research/THEORETICAL-FOUNDATIONS.md"),
]


WEFTOS_PAIRS: list[tuple[str, str]] = [
    # The agents/ copy mirrors the canonical .claude/skills/ copy.
    (".claude/skills/weftos-ecc/WEAVER.md", "agents/weftos-ecc/WEAVER.md"),
]


def _default_root() -> Path:
    """Repo root inferred from this script's location: scripts/audit/consolidate.py."""
    return Path(__file__).resolve().parent.parent.parent


def main() -> int:
    if len(sys.argv) < 2 or len(sys.argv) > 3 or sys.argv[1] not in {"ruvector", "weftos"}:
        print("usage: consolidate.py {ruvector|weftos} [repo_root]", file=sys.stderr)
        return 2
    label = sys.argv[1]
    root = Path(sys.argv[2]).resolve() if len(sys.argv) == 3 else _default_root()
    if not root.is_dir():
        print(f"repo root not a directory: {root}", file=sys.stderr)
        return 2
    pairs = RUVECTOR_PAIRS if label == "ruvector" else WEFTOS_PAIRS
    print(f"# consolidating {len(pairs)} pair(s) in {label} (root={root})")
    consolidate(root, label, pairs)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
