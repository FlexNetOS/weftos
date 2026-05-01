# WeftOS Knowledge-Base — Manual Review Queue

Items here could not be safely auto-consolidated. See
[`AUDIT_REPORT.md`](AUDIT_REPORT.md) for the methodology.

## 1. Near-duplicate pairs (Jaccard ≥ 0.70)

| # | Jaccard | File A | File B | Recommended action |
| -:| -:      | ---    | ---    | ---                |
| 1 | 0.82 | `gui/README.md` | `clawft-ui/README.md` | Confirm whether `gui/` and `clawft-ui/` are the same surface or two products. If same, merge READMEs. If different, differentiate them. |

## 2. Numeric-claim contradictions worth resolving

### 2.1 Rust MSRV / toolchain (HIGH PRIORITY)

| Version cited | Files |
| ---           | ---   |
| `1.66` | `docs/architecture/wasm-browser-portability-analysis.md` |
| `1.85` | `docs/adr/adr-037-rust-edition-2024-msrv.md` |
| `1.93` | `README.md`<br>`docs/deployment/release.md`<br>`docs/deployment/wasm.md`<br>`docs/development/contributing.md`<br>`docs/getting-started/quickstart.md` |

Action: pick a single canonical Rust toolchain (likely 1.93 to match `rust-toolchain.toml`)
and update the older docs (`adr-037`, `wasm-browser-portability-analysis`) to either match
or explicitly note "as of YYYY-MM" so the historical context is preserved.

### 2.2 Node version cluster (mostly noise)

The `node_version` regex matches numeric tokens that look like Node major versions.
Most hits in the WeftOS docs are false positives (e.g. cluster sizes, lifecycle counts).
The legitimate signals to align:

| Version | Files |
| ---     | ---   |
| `18` | `README.md`<br>`docs/weftos/INSTALL.md` |
| `20` | `docs/ui/developer-guide.md`<br>`docs/weftos/sprint11-symposium/03-release-engineering.md` |

Action: align the install/release docs on a single Node target (likely 20 LTS).
