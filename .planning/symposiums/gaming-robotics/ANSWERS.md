# Pre-Symposium Answers

**Date**: 2026-04-11
**Respondent**: WeftOS Creator

---

## Strategy

1. **Parallel** — gaming and robotics simultaneously. Simple robotics hardware first.
2. **Unity first**, possibly Unreal. Need to test which has better hook/harness integration.
3. **Custom 3D-printed** robots. Industrial platform for simulated/synthetic testing if available.
4. **All three demos in parallel**: servo calibration first, gait learning on mini robot, 3D print QC simultaneously. Work through them in order of readiness.

## Hardware & Budget

5. **Keep it cheap.** Lots of small parts already on hand. Building sensor kits.
6. **Yes** — Bambu Lab printer + Prusa (PS1). Two printers available.
7. **ESP32s for sensors, Pi 5s for brains.** Gaming runs on desktop (no dedicated hardware). 3D printing = combo. Robotics = multiple ESP32s + Pi 5. A typical robot will have multiple ESP32s and a Pi 5.

## Timeline & Resources

8. **No fixed timeline.** Work through things as fast as practical, in the right order.
9. **~10 human hours/week + many bot hours.** Agent-heavy development.

## Product & Business

10. **Not a distraction.** Proves out WeftOS capabilities. These are commercially viable products being built. Need to find the best applications of WeftOS. Sesame = hobby project, good starting point, has crab-like element.
11. **Open source**: not decided yet, evaluate per product.
12. **Sesame = hobby starting point**, not a product. Good platform to learn on.

## Safety, Legal & IP

13. **No safety certification yet.** Will address per-product as designs mature.
14. **Nemesis patent: design around it.** Want to understand what the patent covers.
15. **Blog posts primarily.** LinkedIn publishing as testing/building progresses. Some academic papers and conference demos later.

## Technical Architecture

16. **Best-effort first.** Add real-time guarantees as a feature later (like the DiskANN feature flag pattern). Some platforms can't run Rust — handle gracefully.
17. **Many small trees, not one big forest.** Evaluate per application. Smaller graphs when possible.
18. **Binaries as sidecars, not WASM.** Linux and Windows gaming platforms already have native binaries. Use WeftOS as a sidecar process, game connects via TCP. WASM is a fallback, not primary.

---

## Key Decisions

- **Sidecar model for gaming**: WeftOS runs as separate process, game engine connects via TCP. Not embedded in the engine.
- **Multi-ESP32 robot architecture**: one robot = multiple ESP32 sensor nodes + Pi 5 brain.
- **Agent-heavy development**: 10 human hours + extensive bot hours per week.
- **Parallel tracks**: gaming and robotics advance simultaneously.
- **Two 3D printers available**: Bambu + Prusa for QC experiments.
