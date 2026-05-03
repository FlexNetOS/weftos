# n8n ⇄ RuVector Knowledge-Base Automation (WeftOS)

This directory wires the WeftOS knowledge base (every `*.md` under
`docs/`, `agents/`, `crates/**/README.md`) into [n8n](https://n8n.io)
so that doc changes trigger automation flows backed by RuVector's
vector-search and memory APIs.

The integration is **identical** to the one shipped under FlexNetOS/ruvector
`scripts/n8n/` — both repos point at the same `RUVECTOR_BASE_URL` so a
single RuVector instance can serve as the cross-repo knowledge brain.

## Architecture

```
Markdown / Wiki / crate docs ─► ingest_docs.py ─► n8n /webhook/kb-ingest ─► RuVector /v1/memories
                                                                             │
            ┌────────────────────────────────────────────────────────────────┘
            ▼
   n8n /webhook/kb-query ─► RuVector /v1/memories/search ─► top-k JSON
```

## Quick start

```bash
# 1. Bring up n8n + the local RuVector-compatible memory shim
docker compose -f scripts/n8n/docker-compose.yml up -d

# 2. Import the workflows into n8n
N8N_HOST=http://localhost:5678 \
RUVECTOR_BASE_URL=http://localhost:8080 \
python3 scripts/n8n/import_workflows.py

# 3. Ingest the knowledge base
RUVECTOR_BASE_URL=http://localhost:8080 \
python3 scripts/n8n/ingest_docs.py --root . --namespace weftos-docs

# 4. Query
curl -sS -X POST http://localhost:5678/webhook/kb-query \
  -H 'content-type: application/json' \
  -d '{"q":"WEAVER tonal architecture","k":3,"namespace":"weftos-docs"}' | jq .
```

## Files

| File | Purpose |
| ---  | ---     |
| `docker-compose.yml`             | Brings up n8n + the local RuVector shim. |
| `workflows/knowledge-base-ingest.json` | n8n workflow accepting ingest webhooks (single + batch) and forwarding to RuVector. |
| `workflows/knowledge-retrieval.json`   | n8n workflow performing RAG-style retrieval over RuVector. |
| `workflows/kb-on-doc-change.json`      | CI hook — fan-out to ingest webhook for every changed doc path. |
| `ingest_docs.py`                 | Walks the repo, chunks markdown, POSTs to either the n8n webhook or directly to RuVector. |
| `ruvector_shim.py`               | Tiny in-memory FastAPI shim implementing the same `/v1/memories*` surface. Useful for local n8n testing without needing the full Rust build. |
| `import_workflows.py`            | Imports the workflow JSON files into a running n8n via its REST API. |
| `verify.sh`                      | End-to-end smoke test: ingest a few docs, query, assert non-empty results. |

## RuVector backends

The same workflows work against three RuVector-compatible backends:

1. **`ruvector_shim.py`** (default for local dev) — Python in-memory shim
   on port 8080. Zero build time, used by the verify suite.
2. **`mcp-brain-server-local`** — the Rust standalone with SQLite-backed
   storage. Long-running and crash-safe; recommended for development.
3. **`mcp-brain-server`** (Cloud Run / pi.ruv.io) — the production REST API
   under `/v1/memories*`. Switch by setting `RUVECTOR_BASE_URL` to
   `https://pi.ruv.io` and adding an `Authorization: Bearer <key>` header.

Switching backends is a single env-var change with no workflow edits.

### Port-conflict note

Both this dev stack and the root `docker-compose.yml` (which runs
`weftos-node`) bind host port `8080`. They are independent compose
projects, so docker won't warn about it — but if you bring both up on
the same host you'll see a bind error. Resolve by remapping the shim:

```yaml
# scripts/n8n/docker-compose.yml — override the host side only
ruvector:
  ports:
    - "18080:8080"
```

…and update `RUVECTOR_BASE_URL` accordingly when running the host-side
helpers (`import_workflows.py`, `ingest_docs.py`, `verify.sh`). Inside
the n8n container the upstream URL stays `http://ruvector:8080` because
the containers share an internal network.

## Trigger surface

| Webhook                                 | Method | Purpose |
| ---                                     | ---    | ---     |
| `POST /webhook/kb-ingest`               | POST   | Ingest a single document. |
| `POST /webhook/kb-ingest/batch`         | POST   | Ingest a batch (`{docs: [...]}`). |
| `POST /webhook/kb-query`                | POST   | Search (`{q, k, namespace}`). |
| `POST /webhook/kb-on-doc-change`        | POST   | Triggered by a CI hook on doc commits. |

## See also

* [`docs/audit/AUDIT_REPORT.md`](../../docs/audit/AUDIT_REPORT.md) — the
  knowledge-base audit summary that produced the items currently flagged
  for manual review.
* [`docs/audit/MANUAL_REVIEW.md`](../../docs/audit/MANUAL_REVIEW.md) — the
  triage queue for items that need a human decision.
* The companion integration in
  [`FlexNetOS/ruvector` `scripts/n8n/`](https://github.com/FlexNetOS/ruvector/tree/main/scripts/n8n)
  is identical and points at the same RuVector instance — both repos can
  share a single brain.
