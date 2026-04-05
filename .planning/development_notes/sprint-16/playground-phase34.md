# Playground Phase 3-4: Governance + Agent Spawning UI

## What changed

Added two new interactive panels to the WASM sandbox (`docs/src/app/clawft/WasmSandbox.tsx`) as tabs alongside the existing ExoChain log.

### Phase 3: Governance Panel

- **Three-branch visual**: Shows Legislative/Executive/Judicial with blocking/warning/advisory counts per branch, matching the 22 genesis rules from `crates/clawft-kernel/src/governance.rs`.
- **Rules list**: Collapsible list of all 22 genesis rules (GOV-001..007, SOP-L/E/J series) with branch coloring, severity badges, and descriptions.
- **Live decision feed**: Every agent action triggers a mock governance evaluation using the real EffectVector magnitude vs. threshold model. Decisions (Permit/Deny/Warn/Escalate) appear in real time.
- **Chain integration**: All governance decisions log to the ExoChain panel with new op codes: `GOV_GENESIS`, `GOV_PERMIT`, `GOV_WARN`, `GOV_DENY`, `GOV_DEFER`, `GOV_EVAL`.

### Phase 4: Agent Spawning UI

- **Type selector**: 5 agent types (coder, reviewer, researcher, planner, tester) with toggle buttons.
- **Spawn button**: Creates a mock agent with PID assignment, spawning animation, and 1.2-2s lifecycle transition to running state.
- **Agent list**: Shows all agents sorted newest-first with state indicators (green=running, yellow/pulse=spawning, grey=stopped).
- **Stop button**: Stops running agents with governance check logged.
- **Governance gating**: Every spawn/stop action runs through the mock governance evaluator, producing chain log entries and governance panel events.
- **Capacity limit**: Max 8 active agents (matching kernel defaults).

### New op codes added to OP_COLORS

| Code | Color | Category |
|------|-------|----------|
| GOV_GENESIS | indigo-400 | Governance |
| GOV_PERMIT | green-400 | Governance |
| GOV_WARN | yellow-400 | Governance |
| GOV_DENY | red-400 | Governance |
| GOV_DEFER | orange-400 | Governance |
| GOV_EVAL | indigo-300 | Governance |
| AGENT_SPAWN | teal-400 | Agent lifecycle |
| AGENT_READY | teal-300 | Agent lifecycle |
| AGENT_STOP | rose-400 | Agent lifecycle |
| AGENT_LIFECYCLE | teal-400 | Agent lifecycle |

## Design decisions

- All mock data mirrors the real kernel types from `governance.rs`: GovernanceBranch, RuleSeverity, GovernanceDecision, EffectVector magnitude scoring.
- The 22 genesis rules match the test helper `genesis_engine()` in governance.rs exactly (same IDs, branches, severities, SOP categories).
- Governance evaluation uses randomized EffectVector magnitudes against a 0.70 threshold (matching Development environment defaults).
- Right panel uses a 3-tab layout (ExoChain/Governance/Agents) to keep the existing chain log accessible while adding new functionality.
- No new files created -- everything is contained in WasmSandbox.tsx using the same Tailwind + fd-* theme token patterns as the existing UI.

## Boot sequence additions

Two new chain entries during boot:
- `GOV_GENESIS`: "Constitutional governance loaded: 22 rules (3 branches)"
- `GOV_GENESIS`: "Risk threshold: 0.70, environment: Development"

## Files modified

- `docs/src/app/clawft/WasmSandbox.tsx` -- added types, mock data, state, callbacks, and 2 new panel components (~350 lines added)
