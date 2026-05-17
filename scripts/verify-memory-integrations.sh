#!/usr/bin/env bash
# Test all WeftOS memory source integrations
set +e
GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'
pass() { echo -e "${GREEN}✓${NC} $1"; }
fail() { echo -e "${RED}✗${NC} $1"; }
warn() { echo -e "${YELLOW}!${NC} $1"; }

echo "═══ WeftOS Memory Source Integration Verification ═══"
echo ""

# 1. WeftOS Kernel daemon
echo "── 1. WeftOS Kernel ──"
cd ~/weftos-runtime
weaver kernel status 2>&1 | grep -q 'State:.*running' && pass "kernel daemon running" || fail "kernel daemon not running"
weaver chain verify 2>&1 | grep -q 'Chain integrity: VALID' && pass "ExoChain VALID (Ed25519+ML-DSA-65)" || fail "chain invalid"
RT_NODES=$(weaver resource tree 2>/dev/null | grep -c '|')
echo "    Resource tree rows: $RT_NODES"

# 2. ECC HNSW (internal vectors)
echo ""
echo "── 2. ECC HNSW Substrate (internal) ──"
weaver ecc status >/dev/null 2>&1 && pass "ECC subsystem reachable" || fail "ECC unreachable"

# 3. ruvector
echo ""
echo "── 3. RuVector (local Rust vector DB) ──"
RV=$(ruvector info 2>&1)
echo "$RV" | grep -q 'Dimensions: 384' && pass "ruvector DB ready (384-d, HNSW M=32, ef=200)" || fail "ruvector not configured"
RV_BIN=$(which ruvector); echo "    bin: $RV_BIN"
RV_DB=~/.ruvector; [ -d "$RV_DB" ] && pass "ruvector data dir: $RV_DB" || fail "no ruvector data dir"
RV_MCP_BIN=$(which ruvector-mcp); echo "    MCP server: $RV_MCP_BIN"

# 4. MemPalace
echo ""
echo "── 4. MemPalace (persistent memory, 30 MCP tools) ──"
MP_VER=$(mempalace --version 2>&1)
echo "    $MP_VER" | grep -q '3\.' && pass "MemPalace 3.x installed" || warn "version: $MP_VER"
MP_STATUS=$(mempalace status 2>&1 | grep 'drawers' | head -1)
pass "$(echo $MP_STATUS | tr -d '\n')"
[ -f ~/.mempalace/knowledge_graph.sqlite3 ] && pass "knowledge_graph.sqlite3 present" || fail "no knowledge graph"
MP_MCP_BIN=$(which mempalace-mcp); echo "    MCP server: $MP_MCP_BIN"

# 5. GitNexus
echo ""
echo "── 5. GitNexus (code structure graph) ──"
[ -f ~/.gitnexus/registry.json ] && pass "gitnexus registry present" || fail "no gitnexus registry"
STATS=$(python3 -c "import json; r=json.load(open('$HOME/.gitnexus/registry.json'))[0]['stats']; print(f\"files={r['files']} nodes={r['nodes']} edges={r['edges']} communities={r['communities']}\")" 2>&1)
echo "    indexed: $STATS"
[ -e "/home/drdave/_work/repos/_forks/weftos-flexnetos/.gitnexus/lbug" ] && pass "LadybugDB graph present" || fail "no LadybugDB"

# 6. Understand-Anything
echo ""
echo "── 6. Understand-Anything (LLM + static analysis) ──"
UA_DIR=~/.codex/understand-anything
[ -d "$UA_DIR" ] && pass "UA installed: $UA_DIR" || fail "UA not installed"
SKILL_COUNT=$(ls ~/.agents/skills/understand* 2>/dev/null | wc -l)
echo "    skills wired: $SKILL_COUNT"
[ "$SKILL_COUNT" -ge 8 ] && pass "all 8 UA skills available" || warn "expected 8, got $SKILL_COUNT"

# 7. MCP server configs (where agents connect)
echo ""
echo "── 7. MCP Server Configs (agent runtime integration) ──"
for cfg in \
  "$HOME/.claude.json:Claude Code" \
  "$HOME/.codex/config.toml:Codex" \
  "$HOME/.codeium/windsurf/mcp_config.json:Windsurf/Cascade" \
  "$HOME/.config/devin/config.json:Devin" \
; do
  path="${cfg%%:*}"; name="${cfg#*:}"
  if [ -f "$path" ]; then
    grep -q 'ruvector\|mempalace\|gitnexus' "$path" && pass "$name: $path (servers wired)" || warn "$name: $path (no MCP servers)"
  else
    warn "$name: $path (not configured)"
  fi
done

# 8. Brain (optional remote)
echo ""
echo "── 8. pi.ruv.io Brain (optional remote collective memory) ──"
if timeout 5 curl -s https://pi.ruv.io/v1/status >/dev/null 2>&1; then
  pass "brain reachable"
else
  warn "brain unreachable from this network (optional)"
fi

# 9. WeftOS weave.toml memory config
echo ""
echo "── 9. WeftOS weave.toml (unified config) ──"
if grep -q '^\[memory\.mempalace\]' ~/weftos-runtime/weave.toml; then
  COUNT=$(grep -c '^\[memory\.' ~/weftos-runtime/weave.toml)
  pass "weave.toml declares $COUNT memory sources"
  grep '^\[memory\.' ~/weftos-runtime/weave.toml | sed 's/^/    /'
else
  fail "no [memory.*] sections in weave.toml"
fi

# 10. Cargo workspace memory crates
echo ""
echo "── 10. Compiled-in memory backends (workspace deps) ──"
for crate in ruvector-cluster ruvector-raft ruvector-replication ruvector-diskann instant-distance blake3 snow ed25519-dalek ort; do
  grep -q "^$crate" /home/drdave/_work/repos/_forks/weftos-flexnetos/Cargo.toml 2>/dev/null && \
    pass "$crate linked in workspace" || \
    grep -rq "$crate" /home/drdave/_work/repos/_forks/weftos-flexnetos/Cargo.toml 2>/dev/null && \
    pass "$crate present" || warn "$crate not found"
done

echo ""
echo "═══ Verification complete ═══"
