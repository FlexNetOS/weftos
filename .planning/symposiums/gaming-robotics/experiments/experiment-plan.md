# WeftOS Gaming and Robotics Symposium -- Experiments Plan

**Version**: 1.0
**Date**: 2026-04-04
**Authors**: Experiments Design Team
**Status**: Draft

---

## Overview

This document defines eight experiments that demonstrate the Embodied Causal
Cognition (ECC) loop running on the WeftOS kernel in physical-robotics and
game-simulation contexts. Each experiment is self-contained, reproducible, and
designed to produce a clear quantitative result within a one-day to one-week
window.

All experiments share a common software foundation:

- **WeftOS kernel** (`clawft-kernel`) with the `ecc` feature enabled
- **CausalGraph** -- concurrent DAG with typed/weighted edges
  (`Causes`, `Inhibits`, `Correlates`, `Enables`, `Follows`,
  `Contradicts`, `TriggeredBy`, `EvidenceFor`)
- **HnswService** -- approximate nearest-neighbor index for embedding
  storage and retrieval
- **EccCalibration** -- boot-time benchmarking that measures per-tick
  latency (p50/p95), auto-tunes cadence, and reports spectral feasibility
- **BLAKE3** Merkle commits for every tick (tamper-evident audit trail)

Kernel APIs referenced throughout (from `clawft-kernel/src`):

| Module          | Key API                                           |
|-----------------|---------------------------------------------------|
| `causal.rs`     | `CausalGraph::add_node`, `link`, `traverse_forward`, `spectral_analysis`, `predict_changes`, `compute_coupling` |
| `hnsw_service.rs` | `HnswService::insert`, `search`, `search_batch`, `save_to_file`, `load_from_file` |
| `calibration.rs`  | `run_calibration` -> `EccCalibration` (p50/p95/headroom/spectral_capable) |

The ESP32 edge benchmark (`clawft-edge-bench`) provides the scoring
framework (throughput, latency, scalability, stability, endurance) that
several experiments reuse for hardware characterization.

---

## Experiment 1: Servo Calibration via ECC

### 1.1 Hypothesis

ECC can characterize servo response curves faster than manual calibration
while achieving equivalent or superior positional accuracy. Specifically:
the causal graph discovers the command-to-position transfer function in
less than 1/10th the time a human operator requires.

### 1.2 Background

Hobby servos (SG90, MG996R) exhibit non-linear response, dead-band
regions, and load-dependent droop that must be characterized before
precision work. Traditional calibration requires a human to command
discrete positions, measure actual angles with a protractor or encoder,
and build a lookup table by hand. ECC automates the entire loop: command
a position, observe the result via computer vision, record the error in
a causal graph, and converge on a correction curve.

### 1.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Raspberry Pi 5 (8 GB) | BCM2712, 2.4 GHz quad-core | 1 | $80 | $80 |
| PCA9685 16-channel PWM driver | I2C, 12-bit resolution | 1 | $6 | $6 |
| TowerPro SG90 micro servo | 0-180 deg, 0.12s/60deg | 4 | $3 | $12 |
| ArUco marker sheet | 4x4_50 dictionary, 20mm markers | 1 | $2 | $2 |
| Raspberry Pi Camera Module 3 | 12 MP, autofocus | 1 | $25 | $25 |
| Servo mounting bracket (3D printed) | PLA, M2 hardware | 4 | $1 | $4 |
| 5V 3A power supply | Barrel jack for servo rail | 1 | $8 | $8 |
| Jumper wires, breadboard | Standard kit | 1 | $5 | $5 |
| Calibration weights (10g, 50g, 100g) | Brass, slotted | 1 set | $12 | $12 |
| **Total** | | | | **$154** |

### 1.4 Software Requirements

| Component | Crate / Package | Version |
|-----------|----------------|---------|
| WeftOS kernel | `clawft-kernel` (ecc feature) | workspace |
| PWM control | `rppal` (Rust Pi GPIO) | >= 0.18 |
| PCA9685 driver | `pwm-pca9685` | >= 0.4 |
| Camera capture | `opencv` (Rust bindings) or `nokhwa` | >= 0.14 |
| ArUco detection | `opencv::aruco` | via opencv |
| Data logging | `clawft-exochain` | workspace |
| Visualization | `plotters` | >= 0.3 |
| TCP bridge | `clawft-daemon` RPC | workspace |

### 1.5 Protocol

**Phase A -- Baseline characterization (no load)**

1. Flash Pi 5 with WeftOS image; boot kernel with ECC enabled.
2. Run `run_calibration()` to establish tick cadence and spectral
   feasibility on this hardware. Record `EccCalibration` result.
3. Attach ArUco marker (ID 0-3) to each servo horn.
4. Position camera 30 cm above servo mounting plate, calibrate intrinsics
   with a checkerboard (8x6, 25mm squares).
5. For each servo `s` in `[0..3]`:
   a. Sweep command from 0 to 180 degrees in 1-degree increments.
   b. At each step:
      - Send PWM command via PCA9685.
      - Wait 200 ms for servo to settle.
      - Capture frame, detect ArUco marker, compute angle from marker
        pose relative to reference.
      - Compute error: `e = actual_angle - commanded_angle`.
      - Insert observation into CausalGraph:
        ```
        node_cmd  = causal.add_node("cmd_{s}_{deg}", {servo: s, deg: deg})
        node_obs  = causal.add_node("obs_{s}_{deg}", {actual: measured, err: e})
        causal.link(node_cmd, node_obs, CausalEdgeType::Causes, weight=1.0)
        ```
      - Insert embedding vector into HNSW:
        `[servo_id, command_deg/180, actual_deg/180, error, 0..0]`
        padded to 16 dimensions.
   c. After full sweep, run `spectral_analysis(100)` on the subgraph for
      servo `s` to identify clusters of high-error regions.
   d. Run `predict_changes()` from the dead-band region nodes to verify
      the graph correctly identifies the dead band.
6. Record total wall-clock time for all 4 servos (expected: ~15 min).

**Phase B -- Load characterization**

7. Repeat Phase A with 10g, 50g, and 100g weights attached to each servo
   horn via a clip at 30mm from the pivot.
8. For each load level, create new causal nodes and link them to the
   unloaded nodes with `CausalEdgeType::Correlates`.
9. Run `compute_coupling()` between the load subgraph and the no-load
   subgraph to quantify how load shifts the transfer function.

**Phase C -- ECC-driven adaptive correction**

10. Use the causal graph to generate a piecewise-linear correction table:
    for each commanded angle, look up the HNSW-nearest observation and
    apply the inverse error.
11. Re-sweep all 4 servos using the corrected commands.
12. Measure residual error after correction.

**Phase D -- Human baseline**

13. A human operator calibrates the same 4 servos by hand using a
    digital protractor. Record wall-clock time and residual errors.

### 1.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Mean absolute error (MAE) | avg(\|actual - commanded\|) after correction | < 1.0 deg |
| Max error | worst-case single-point error | < 3.0 deg |
| Calibration time (ECC) | wall-clock for full 4-servo sweep + correction | < 20 min |
| Calibration time (human) | wall-clock for human to achieve same MAE | > 120 min |
| Speed ratio | human_time / ecc_time | > 6x (target 10x) |
| Drift detection | error change after 1000 command cycles | measurable |
| Dead-band identification | ECC correctly flags dead-band region width | within 2 deg |
| Causal graph nodes | total nodes created | ~1,440 (4 servos x 181 x 2) |
| HNSW entries | total embeddings stored | ~720 |

### 1.7 Expected Results

- ECC produces a calibration curve with MAE below 1.0 degree within
  15-20 minutes of automated measurement, compared to 2-3 hours for a
  human performing the same task manually.
- The spectral analysis correctly identifies dead-band regions (typically
  0-5 degrees and 175-180 degrees for SG90 servos).
- Load-dependent droop is captured in the causal graph coupling scores,
  enabling predictive compensation for unknown loads.
- After 1000 repeated command cycles, the HNSW-based drift detector
  flags positional degradation before it exceeds the 3-degree threshold.

### 1.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Hardware assembly + camera calibration | 1 day | 1 |
| Software integration (PWM + camera + kernel) | 2 days | 1 |
| Phase A + B data collection | 1 day | 1 |
| Phase C correction + Phase D human baseline | 1 day | 1 |
| Analysis + visualization | 1 day | 1 |
| **Total** | **6 days** | |

### 1.9 Success Criteria

- PASS: Speed ratio >= 6x AND MAE <= 1.5 deg
- STRONG PASS: Speed ratio >= 10x AND MAE <= 1.0 deg
- FAIL: Speed ratio < 3x OR MAE > 3.0 deg

### 1.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| ArUco detection fails under servo vibration | Medium | High | Use 500ms settle time; average 3 frames |
| PCA9685 jitter exceeds servo resolution | Low | Medium | Use hardware smoothing capacitor on PWM lines |
| Camera autofocus hunts during sweep | Medium | Low | Lock focus manually after initial calibration |
| Pi 5 thermal throttling during long runs | Low | Medium | Add heatsink; monitor `vcgencmd measure_temp` |

### 1.11 Publication Potential

- **Venue**: IEEE International Conference on Robotics and Automation
  (ICRA), workshop track on automated calibration
- **Audience**: Robotics engineers, hobbyist-to-professional pipeline
- **Angle**: First demonstration of causal-graph-driven servo calibration
  that outperforms manual methods with commodity hardware

---

## Experiment 2: Sesame Robot Gait Learning

### 2.1 Hypothesis

ECC can learn a stable bipedal gait from scratch in fewer than 100
attempts, where each attempt is a sequence of joint commands evaluated
for balance. The causal graph discovers which joint-angle combinations
produce stable center-of-mass trajectories, and HNSW stores successful
micro-movements for rapid composition.

### 2.2 Background

Bipedal gait generation is traditionally solved with either analytical
inverse kinematics (ZMP-based) or deep reinforcement learning (PPO, SAC)
requiring millions of simulation steps. ECC offers a middle path: causal
reasoning over a small number of real-world trials, storing discoveries
in an HNSW index for fast retrieval and composition.

### 2.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Sesame humanoid robot | 22 servos, 35cm tall | 1 | $450 | $450 |
| Raspberry Pi 5 (8 GB) | Compute platform | 1 | $80 | $80 |
| MPU-6050 IMU breakout | 6-axis accel + gyro, I2C | 1 | $4 | $4 |
| PCA9685 PWM driver (x2) | 32 channels total | 2 | $6 | $12 |
| Safety harness + gantry frame | Aluminum extrusion, pulleys | 1 | $60 | $60 |
| LiPo battery 7.4V 2200mAh | For servo power | 1 | $18 | $18 |
| Foam crash mat | 60x120 cm, 5cm thick | 1 | $15 | $15 |
| USB-C power supply (Pi) | 5V 5A | 1 | $12 | $12 |
| **Total** | | | | **$651** |

### 2.4 Software Requirements

| Component | Crate / Package | Purpose |
|-----------|----------------|---------|
| WeftOS kernel | `clawft-kernel` (ecc) | Causal graph + HNSW |
| IMU driver | `mpu6050` crate or `linux-embedded-hal` | Balance sensing |
| Servo control | `rppal` + `pwm-pca9685` | Joint actuation |
| Kinematics | Custom module | Forward kinematics for CoM |
| State machine | `clawft-claw-stage` | Gait phase management |
| Telemetry | `clawft-exochain` | Audit trail of attempts |
| Visualization | `rerun` (3D viewer) | Real-time joint replay |

### 2.5 Protocol

**Phase A -- Sensor fusion setup**

1. Mount IMU on robot torso (center of mass region).
2. Calibrate IMU: collect 1000 samples at rest, compute gyro bias and
   accelerometer scale factors.
3. Boot WeftOS kernel; run `run_calibration()` to verify tick cadence
   supports 50 Hz control loop (20ms tick).
4. Verify: read IMU at 50 Hz, confirm pitch/roll accuracy within 1 deg.

**Phase B -- Micro-movement discovery**

5. Define the 22-servo joint space. Group joints by limb:
   - Left leg: hip_yaw, hip_roll, hip_pitch, knee, ankle_pitch, ankle_roll (6)
   - Right leg: same (6)
   - Left arm: shoulder_pitch, shoulder_roll, elbow (3)
   - Right arm: same (3)
   - Torso: waist_yaw, waist_pitch (2)
   - Head: pan, tilt (2)
6. Start with all servos at neutral position (standing upright, held by
   harness).
7. For each micro-movement attempt `t` in `[0..500]`:
   a. Select a random subset of 2-4 leg joints.
   b. Apply small random perturbation (uniform +/-5 degrees).
   c. Hold for 200ms, record IMU (pitch, roll, yaw rates, accel).
   d. Compute stability score:
      ```
      stability = 1.0 / (1.0 + pitch_rate.abs() + roll_rate.abs())
      ```
   e. Create causal graph nodes:
      ```
      node_action = causal.add_node("action_{t}", {joints: deltas})
      node_result = causal.add_node("result_{t}", {stability, pitch, roll})
      causal.link(node_action, node_result, CausalEdgeType::Causes, stability)
      ```
   f. If stability > 0.7, insert the joint-delta vector into HNSW with
      metadata `{stable: true, phase: "micro"}`.
   g. Return servos to neutral before next attempt.
8. After 500 micro-movements, run `detect_communities()` on the causal
   graph to identify clusters of related successful movements.

**Phase C -- Step composition**

9. Define a "step" as a sequence of 4 phases: lift, swing, place, shift.
10. For each step attempt `s` in `[0..100]`:
    a. For each phase, query HNSW for the 5 nearest micro-movements
       matching the desired phase direction.
    b. Compose the top-scoring micro-movements into a 4-phase sequence.
    c. Execute the sequence (total duration ~800ms per step).
    d. Measure: distance moved (from IMU integration or ground marker),
       falls (harness tension sensor or pitch > 45 deg), smoothness
       (jerk integral from IMU).
    e. Score the step:
       ```
       step_score = distance * smoothness * (1.0 if no_fall else 0.1)
       ```
    f. Create causal nodes linking the 4 phase nodes to the step outcome.
    g. If step_score > threshold, store the composed step in HNSW as a
       "macro-movement".
11. Iterate: each successive step attempt uses the latest HNSW index,
    so the system converges on better compositions.

**Phase D -- Continuous walking**

12. Chain successful macro-steps into a walking sequence.
13. Release harness partially (allow lateral sway, catch falls).
14. Measure:
    - Distance walked before first fall.
    - Falls per 10-step sequence.
    - Gait smoothness (standard deviation of step duration).
    - Symmetry (left vs right step length ratio).

**Phase E -- Human baseline**

15. An experienced roboticist manually tunes gait parameters for the
    same Sesame robot using a traditional approach (analytical + trial
    and error). Record total time to achieve a stable 10-step walk.

### 2.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Attempts to first step | micro-movements + compositions until first forward step | < 200 total |
| Attempts to stable gait | total until 10 consecutive steps without fall | < 500 total |
| Falls per 10 steps | at steady state | < 1 |
| Gait smoothness | std_dev(step_duration) / mean(step_duration) | < 0.15 |
| Step symmetry | min(left_len, right_len) / max(left_len, right_len) | > 0.85 |
| Walking speed | meters per minute at steady state | > 0.5 m/min |
| Human baseline time | hours to achieve equivalent stability | > 8 hours |
| Causal graph size | nodes + edges at end of training | ~2,000 nodes, ~5,000 edges |
| HNSW index size | stored micro + macro movements | ~300-500 entries |

### 2.7 Expected Results

- The ECC system discovers stable micro-movements within the first 100
  attempts, composes a first step by attempt 150, and achieves a stable
  10-step gait by attempt 400.
- The causal graph community detection reveals 4-6 distinct movement
  clusters corresponding to the natural gait phases.
- Total ECC training time (including all 500+ attempts at 1s each) is
  under 15 minutes wall-clock, versus 8+ hours for human tuning.

### 2.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Hardware assembly + harness fabrication | 2 days | 1 |
| IMU calibration + software integration | 2 days | 1 |
| Phase B micro-movement collection | 1 day | 1 |
| Phase C step composition | 1 day | 1 |
| Phase D continuous walking | 1 day | 1 |
| Phase E human baseline | 2 days | 1 (different operator) |
| Analysis + visualization | 2 days | 1 |
| **Total** | **11 days** | |

### 2.9 Success Criteria

- PASS: Stable 10-step gait in < 600 attempts
- STRONG PASS: Stable 10-step gait in < 100 attempts (hypothesis target)
- FAIL: No stable step after 1000 attempts

### 2.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Servo burnout from repeated trials | Medium | High | Limit continuous run to 30min; cool-down periods |
| IMU drift during long sessions | Medium | Medium | Re-zero IMU every 50 attempts |
| Harness interferes with natural gait | High | Medium | Use overhead rail, minimize lateral constraint |
| Fall damages robot | Medium | High | Foam mat + harness catch; limit free-walk time |
| 50 Hz tick too slow for balance | Low | High | Verify with calibration; fall back to 100 Hz |

### 2.11 Publication Potential

- **Venue**: RSS (Robotics: Science and Systems), or pedestrian robotics
  workshop at IROS
- **Audience**: Legged locomotion researchers, embodied AI community
- **Angle**: Sample-efficient gait learning via causal reasoning vs
  model-free RL (100s of trials vs millions of timesteps)

---

## Experiment 3: Motion Mimicry from Video

### 3.1 Hypothesis

ECC can learn an animal walking pattern from video observation and
reproduce it on a bipedal robot with > 80% motion similarity in fewer
than 50 refinement iterations.

### 3.2 Background

Motion mimicry (imitation learning from video) typically requires a
large neural network trained on paired motion-capture data. ECC offers
an alternative: extract a skeleton from video via pose estimation, map
it to the robot's joint space using the causal graph to track
correspondence, and iteratively refine via HNSW-stored motion primitives.

### 3.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Sesame humanoid robot | (shared with Exp 2) | 1 | -- | -- |
| Raspberry Pi 5 (8 GB) | (shared with Exp 2) | 1 | -- | -- |
| Logitech C920 webcam | 1080p, 30fps | 1 | $50 | $50 |
| Monitor or tablet for video playback | >= 10 inch display | 1 | $0 | $0 |
| IMU (shared) | | 1 | -- | -- |
| **Total (incremental)** | | | | **$50** |

### 3.4 Software Requirements

| Component | Crate / Package | Purpose |
|-----------|----------------|---------|
| Pose estimation | MediaPipe Pose (Python) or `tflite-rs` | Extract joint angles from video |
| Skeleton mapper | Custom Rust module | Map source skeleton to robot DOF |
| WeftOS kernel | `clawft-kernel` (ecc) | Causal graph + HNSW |
| Video processing | `ffmpeg` CLI + frame extraction | Pre-process source video |
| Similarity metric | `nalgebra` | Cosine similarity on joint trajectories |

### 3.5 Protocol

**Phase A -- Source video preparation**

1. Select three source videos for reproducibility:
   - **Crab walk**: Sideways locomotion. Source: "Red crab migration
     Christmas Island" (BBC Earth, YouTube ID: `dUBKJDOF-cA`, timestamp
     0:42-0:52, 10 seconds).
   - **Cat stretch**: Full-body extension. Source: "Cat stretching in
     slow motion" (YouTube ID: `_sP2GJWPTKA`, timestamp 0:15-0:25).
   - **Bird head bob**: Rhythmic head movement during walk. Source:
     "Pigeon walking slow motion" (YouTube ID: `QYgBiMq8WQo`, timestamp
     0:08-0:18).
2. Extract frames at 15 fps using ffmpeg:
   ```
   ffmpeg -i source.mp4 -vf fps=15 -ss 00:00:42 -t 10 frames/%04d.png
   ```
3. Run MediaPipe Pose on each frame to extract 33 landmark positions.
4. Compute joint angles (shoulder, elbow, hip, knee, ankle) per frame.
5. Store the target trajectory as a time series of joint-angle vectors.

**Phase B -- Skeleton mapping**

6. Define the mapping between the 33-point MediaPipe skeleton and the
   Sesame robot's 22 servos. Not all source joints have robot
   equivalents (e.g., spine twist); map the closest DOF.
7. Create a CausalGraph mapping:
   ```
   For each source_joint, target_joint pair:
     node_src = causal.add_node("source_{joint}", {angles: [...]})
     node_tgt = causal.add_node("target_{joint}", {servo_id: id})
     causal.link(node_src, node_tgt, CausalEdgeType::Correlates, weight=mapping_confidence)
   ```
8. Insert the target trajectory frames into HNSW as reference embeddings.

**Phase C -- Iterative motion refinement**

9. For each refinement iteration `i` in `[0..50]`:
   a. Generate candidate joint trajectory by:
      - Querying HNSW for the 3 nearest stored movements to each target
        frame.
      - Blending the retrieved movements with the raw mapped trajectory
        (blend factor starts at 0.8 toward raw, shifts toward retrieved
        as iterations progress).
   b. Execute the candidate trajectory on the robot (with harness).
   c. Record actual joint positions via servo feedback (or camera).
   d. Compute frame-by-frame cosine similarity between target and actual
      joint-angle vectors:
      ```
      similarity = sum(target_i * actual_i) / (|target| * |actual|)
      ```
   e. Create causal nodes: candidate -> execution -> similarity.
   f. If similarity > previous best, store the execution in HNSW.
   g. Use `predict_changes()` to identify which joints contribute most
      to the error; focus refinement on those joints.

**Phase D -- Evaluation**

10. After 50 iterations, perform 5 clean executions of the best
    trajectory. Record video from two angles (front and side).
11. Compute final similarity metrics.
12. Have 3 human judges rate the motion similarity on a 1-5 Likert scale
    (blind: they see robot video and source video side by side).

### 3.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Cosine similarity (final) | frame-averaged cosine sim | > 0.80 |
| Iterations to 80% match | refinement steps to reach 0.80 threshold | < 30 |
| Human judge score | average Likert rating (1-5) | > 3.5 |
| Per-joint error | max angular error across all mapped joints | < 15 deg |
| Execution smoothness | jerk integral of actual trajectory | within 2x of target |
| Causal graph insights | joints flagged by predict_changes as error sources | correlates with human observation |

### 3.7 Expected Results

- Crab walk achieves highest similarity (simple lateral motion maps well
  to humanoid side-stepping).
- Cat stretch is hardest (requires spine flexibility the robot lacks);
  expected similarity ~0.65.
- Bird head bob achieves 0.85+ similarity (2-DOF head maps directly).
- The causal graph correctly identifies joint limitations (e.g., ankle
  roll range) as the primary error source.

### 3.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Video preparation + pose extraction | 1 day | 1 |
| Skeleton mapping implementation | 2 days | 1 |
| Iterative refinement runs (3 motions) | 3 days | 1 |
| Human judge evaluation | 1 day | 3 judges + 1 facilitator |
| Analysis | 1 day | 1 |
| **Total** | **8 days** | |

### 3.9 Success Criteria

- PASS: At least one motion achieves cosine similarity > 0.75
- STRONG PASS: Two or more motions achieve > 0.80
- FAIL: No motion exceeds 0.60

### 3.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| MediaPipe fails on animal video | Medium | High | Pre-filter with animal pose models (DeepLabCut) |
| Skeleton mapping ambiguity | High | Medium | Use manual mapping override for ambiguous joints |
| Robot lacks DOF for target motion | High | Medium | Report achievable subset; don't force impossible joints |
| Servo speed limits clip fast motions | Medium | Medium | Time-scale target trajectory to 0.5x speed |

### 3.11 Publication Potential

- **Venue**: Conference on Robot Learning (CoRL)
- **Audience**: Imitation learning, robot motion planning
- **Angle**: Zero-shot cross-species motion transfer via causal graphs
  (no paired training data required)

---

## Experiment 4: 3D Print Quality Learning

### 4.1 Hypothesis

ECC improves 3D print quality over successive prints by building a
causal model that maps printer parameters (temperature, flow rate,
speed) to quality outcomes, and suggests parameter adjustments that
reduce defects by at least 30% compared to default slicer profiles.

### 4.2 Background

FDM 3D printing quality depends on dozens of interacting parameters.
Experts spend hours tuning temperature towers, retraction tests, and
flow calibration cubes. ECC can automate this: print, photograph,
score, adjust, repeat -- with the causal graph tracking which parameter
changes caused which quality changes.

### 4.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Creality Ender-3 V3 SE | Direct drive, auto-level | 1 | $180 | $180 |
| MKS Gen L v2.1 board | 32-bit, Marlin-compatible | 1 | $25 | $25 |
| Raspberry Pi 5 (8 GB) | Compute + camera host | 1 | $80 | $80 |
| Pi Camera Module 3 | (shared or second unit) | 1 | $25 | $25 |
| Ring light | 10-inch, diffuse white | 1 | $15 | $15 |
| PLA filament (1 kg) | Generic white, 1.75mm | 2 | $18 | $36 |
| Digital calipers | 0.01mm resolution | 1 | $12 | $12 |
| **Total** | | | | **$373** |

### 4.4 Software Requirements

| Component | Crate / Package | Purpose |
|-----------|----------------|---------|
| WeftOS kernel | `clawft-kernel` (ecc) | Causal graph + HNSW |
| Printer control | OctoPrint API or `moonraker` | G-code submission, temp control |
| Slicer | PrusaSlicer CLI (`prusa-slicer --slice`) | Generate G-code from STL |
| Image analysis | `opencv` (Rust bindings) | Surface roughness scoring |
| Dimensional analysis | Custom module | Parse caliper measurements or camera-based measurement |
| G-code modifier | Custom Rust module | Inject parameter overrides |

### 4.5 Protocol

**Phase A -- Baseline prints (default profile)**

1. Slice the standard 20mm XYZ calibration cube with Cura "Standard
   Quality" profile for Ender-3 (0.2mm layer height, 200C nozzle, 60C
   bed, 50mm/s).
2. Print 10 identical cubes, numbering them 1-10.
3. For each cube:
   a. Photograph all 6 faces under ring light (consistent 30cm distance,
      same camera settings: ISO 100, f/5.6, 1/60s).
   b. Measure X, Y, Z dimensions with calipers (3 measurements each axis).
   c. Compute surface roughness score: convert each face photograph to
      grayscale, compute the standard deviation of pixel intensity in a
      central 10x10mm ROI. Higher std_dev = rougher surface.
   d. Score stringing: threshold the images for thin bright lines between
      faces; count pixels above threshold.
   e. Score layer adhesion: inspect bottom 3 layers for gaps or
      delamination (binary: pass/fail per cube).
4. Record all 10 cube scores as the baseline distribution.

**Phase B -- ECC-guided parameter adjustment**

5. Create initial causal graph nodes for the 4 primary parameters:
   ```
   node_temp = causal.add_node("nozzle_temp", {value: 200, range: [185, 220]})
   node_flow = causal.add_node("flow_rate", {value: 100, range: [90, 110]})
   node_speed = causal.add_node("print_speed", {value: 50, range: [30, 80]})
   node_retract = causal.add_node("retraction_dist", {value: 5.0, range: [2.0, 8.0]})
   ```
6. For each print `p` in `[11..20]`:
   a. ECC selects which parameter to adjust based on the causal graph:
      - Query `predict_changes()` from each parameter node.
      - Select the parameter with the highest predicted impact on quality.
      - Adjust by a small delta (e.g., +/-5C for temp, +/-5% for flow).
   b. Generate modified G-code via slicer with updated parameters.
   c. Print the cube.
   d. Score the cube (same method as Phase A).
   e. Create causal links:
      ```
      node_param_change = causal.add_node("change_{p}", {param, old, new})
      node_quality = causal.add_node("quality_{p}", {dim_error, roughness, stringing})
      causal.link(node_param_change, node_quality, CausalEdgeType::Causes, delta_quality)
      ```
   f. Insert the parameter vector + quality vector into HNSW.
   g. After each print, run `spectral_analysis()` to check if the graph
      has stabilized (eigenvalue gap indicates convergence).

**Phase C -- Control comparison**

7. Print 10 more cubes with Cura "Fine" profile (0.12mm layer, otherwise
   default). This represents the "best default" without manual tuning.
   Score identically.

### 4.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Dimensional accuracy | mean |measured - 20.00mm| across all axes | < 0.10 mm |
| Surface roughness (ECC) | avg pixel std_dev across 6 faces | 30% lower than baseline |
| Surface roughness (baseline) | same metric on prints 1-10 | reference value |
| Stringing score | pixel count above threshold | 50% reduction vs baseline |
| Layer adhesion | pass rate (no delamination) | 100% |
| Quality improvement rate | quality delta per print (slope) | positive, monotonic after print 13 |
| Parameter convergence | number of prints before ECC stops changing params | < 7 |
| Causal graph edges | total parameter-to-quality links | ~40-60 |

### 4.7 Expected Results

- Prints 11-13 show exploratory variation (some better, some worse).
- Prints 14-20 show monotonic improvement as ECC converges.
- Final ECC prints (18-20) outperform both the standard baseline and the
  fine-profile control by at least 20% on dimensional accuracy and 30%
  on surface roughness.
- The causal graph reveals temperature as the highest-impact parameter
  for roughness, and retraction distance as highest-impact for stringing.

### 4.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Printer setup + camera rig | 1 day | 1 |
| Phase A baseline prints (10 cubes) | 2 days | 1 |
| Phase B ECC-guided prints (10 cubes) | 3 days | 1 |
| Phase C control prints (10 cubes) | 2 days | 1 |
| Measurement + scoring (all 30 cubes) | 1 day | 1 |
| Analysis + visualization | 1 day | 1 |
| **Total** | **10 days** | |

### 4.9 Success Criteria

- PASS: ECC prints 18-20 beat baseline prints 1-10 by >= 20% on at
  least 2 of 3 quality metrics
- STRONG PASS: ECC prints beat both baseline AND fine-profile control
- FAIL: ECC prints show no improvement or regression vs baseline

### 4.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Filament batch variation confounds results | Medium | Medium | Use single spool for all 30 prints |
| Room temperature variation | Medium | Low | Log ambient temp; run in enclosed space |
| Camera exposure drift | Low | Medium | Manual exposure lock; include gray card in frame |
| Print failure (spaghetti, bed adhesion) | Low | High | Glue stick on bed; first-layer monitoring via OctoPrint |

### 4.11 Publication Potential

- **Venue**: Additive Manufacturing journal, or RAPID+TCT conference
- **Audience**: 3D printing community, process optimization engineers
- **Angle**: Autonomous closed-loop print quality optimization without
  ML training data

---

## Experiment 5: Game Character Gait Learning (Simulation)

### 5.1 Hypothesis

ECC can learn bipedal walking in a physics simulator with comparable
sample efficiency to state-of-the-art RL (PPO) while producing an
explainable causal trace of every decision.

### 5.2 Background

OpenAI Gym's Walker2d-v4 (MuJoCo) is the standard benchmark for
bipedal locomotion learning. PPO typically achieves stable walking in
1-3 million timesteps. ECC should match or exceed this sample efficiency
by exploiting causal structure rather than gradient descent.

### 5.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Development workstation | x86-64, 32GB RAM, NVIDIA GPU | 1 | $0 (existing) | $0 |
| Raspberry Pi 5 (optional) | For ECC-on-edge comparison | 1 | $80 | $80 |
| **Total** | | | | **$80** |

### 5.4 Software Requirements

| Component | Crate / Package | Purpose |
|-----------|----------------|---------|
| Physics simulator | MuJoCo 3.x (via `mujoco-rs` or Python bindings) | Rigid body dynamics |
| Alternative sim | Godot 4.3 + `godot-rust` (GDExtension) | If MuJoCo unavailable |
| WeftOS kernel | `clawft-kernel` (ecc) | Causal graph + HNSW |
| TCP bridge | `clawft-daemon` | Connect sim to kernel via RPC |
| RL baseline | Stable Baselines3 (Python, PPO) | Comparison agent |
| Gymnasium | `gymnasium[mujoco]` | Walker2d-v4 environment |
| Plotting | `plotters` or `matplotlib` | Learning curves |

### 5.5 Protocol

**Phase A -- Environment setup**

1. Install MuJoCo 3.x and Gymnasium with MuJoCo backend.
2. Verify Walker2d-v4 runs: random policy for 1000 steps, confirm
   observation/action spaces.
3. Walker2d observation space: 17-dimensional (joint positions + velocities).
4. Walker2d action space: 6-dimensional (joint torques).
5. Boot WeftOS kernel with ECC; connect via TCP (daemon RPC on port 8080).
6. Run `run_calibration()` to verify the kernel can process observations
   at simulator speed (~1000 steps/sec).

**Phase B -- ECC agent implementation**

7. Implement the ECC Walker agent:
   a. Each simulator step becomes a causal graph tick.
   b. Observation vector (17-dim) is inserted into HNSW with metadata
      `{step, reward, done}`.
   c. Action selection:
      - Query HNSW for the 10 nearest observations to the current state.
      - Retrieve the actions associated with those observations.
      - Weight actions by their reward: `action_i * reward_i / sum(rewards)`.
      - Add exploration noise: Gaussian with decaying sigma.
   d. After action execution:
      - Create causal nodes: `state -> action -> next_state`.
      - Edge weight = reward received.
      - If the agent falls (`done=True`), trace back the causal chain to
        the action that initiated the fall. Mark it with
        `CausalEdgeType::Inhibits`.
   e. Every 100 steps, run `spectral_analysis()` to detect whether the
      causal graph has formed stable attractors (eigenvalue gap > 0.1).

**Phase C -- Training runs**

8. Run ECC agent for 500,000 timesteps (or until convergence). Log:
   - Episode reward (cumulative distance walked).
   - Episode length.
   - Falls per episode.
   - HNSW index size.
   - Causal graph node/edge count.
   - `spectral_analysis()` results every 10,000 steps.
9. Repeat 5 times with different random seeds for statistical validity.

**Phase D -- PPO baseline**

10. Train PPO (Stable Baselines3, default hyperparameters for Walker2d)
    for 3,000,000 timesteps. Log same metrics.
11. Repeat 5 times with different seeds.

**Phase E -- Comparison**

12. Plot learning curves: episode reward vs timesteps (ECC vs PPO).
13. Compute sample efficiency: timesteps to reach reward threshold 1000
    (stable walking ~10m).
14. Compute explainability: for 10 random falls in each agent, can we
    trace the cause?
    - ECC: traverse reverse causal chain from fall event.
    - PPO: gradient-based attribution (SHAP or similar).
15. Measure: how many causal chain hops to reach the root cause of a
    fall? (ECC should be 3-5 hops; PPO attribution is not chain-based.)

### 5.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Timesteps to reward=1000 (ECC) | mean across 5 seeds | < 200,000 |
| Timesteps to reward=1000 (PPO) | mean across 5 seeds | ~1,500,000 (published) |
| Sample efficiency ratio | PPO_steps / ECC_steps | > 5x |
| Final reward (ECC) | mean episode reward at convergence | > 2000 |
| Final reward (PPO) | same | > 3000 (PPO often higher asymptotically) |
| Falls per episode (final) | at convergence | < 0.1 |
| Causal trace depth | hops from fall to root cause | 3-5 |
| HNSW index size at convergence | number of stored observations | < 50,000 |
| Graph nodes at convergence | causal graph size | < 100,000 |
| Wall-clock time (ECC) | total training time | < 2 hours |
| Wall-clock time (PPO) | total training time | ~1 hour (GPU-accelerated) |

### 5.7 Expected Results

- ECC reaches reward=1000 in 100,000-200,000 steps (7-15x more
  sample-efficient than PPO).
- PPO achieves higher asymptotic reward (gradient optimization is better
  for fine-tuning).
- ECC's causal traces provide human-readable explanations for every fall
  ("left knee over-extended at step 347, caused by torso lean at step
  342").
- PPO's SHAP attributions provide feature importance but not a causal
  chain.

### 5.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Environment setup + TCP bridge | 2 days | 1 |
| ECC agent implementation | 3 days | 1 |
| Training runs (ECC, 5 seeds) | 1 day | 1 (automated) |
| PPO baseline runs | 1 day | 1 (automated) |
| Analysis + plotting | 2 days | 1 |
| **Total** | **9 days** | |

### 5.9 Success Criteria

- PASS: ECC reaches reward=1000 in < 500,000 steps (3x better than PPO)
- STRONG PASS: ECC reaches reward=1000 in < 200,000 steps (7x better)
- FAIL: ECC does not reach reward=1000 within 3,000,000 steps

### 5.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| HNSW index grows too large for memory | Medium | High | Prune entries with reward < threshold every 50K steps |
| Causal graph traversal becomes slow | Medium | Medium | Monitor `calibration.compute_p95_us`; shard graph by episode |
| ECC never converges (exploration trap) | Medium | High | Add epsilon-greedy fallback; increase exploration noise |
| MuJoCo license/build issues | Low | High | Fall back to Godot physics (Phase A step 2 alternative) |

### 5.11 Publication Potential

- **Venue**: NeurIPS, ICML (main conference or workshop on
  explainable RL)
- **Audience**: RL researchers, embodied AI, explainable AI
- **Angle**: Causal-graph-based locomotion learning: competitive sample
  efficiency with full explainability

---

## Experiment 6: Sim-to-Real Transfer

### 6.1 Hypothesis

An ECC causal graph trained in simulation (Experiment 5) transfers to a
physical robot with fewer than 20 calibration movements, achieving
stable walking significantly faster than learning from scratch
(Experiment 2).

### 6.2 Background

Sim-to-real transfer is one of the hardest problems in robotics. Neural
network policies suffer from the "reality gap" -- sim dynamics differ
from real dynamics. ECC's causal graph is an explicit relational model,
not opaque weights, so it can be inspected and selectively recalibrated.
We hypothesize that only the edge weights (not the graph structure) need
updating.

### 6.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Sesame humanoid robot | (shared with Exp 2) | 1 | -- | -- |
| Raspberry Pi 5 (8 GB) | (shared) | 1 | -- | -- |
| IMU + servos | (shared with Exp 2) | -- | -- | -- |
| Safety harness | (shared) | 1 | -- | -- |
| **Total (incremental)** | | | | **$0** |

### 6.4 Software Requirements

Same as Experiments 2 and 5, plus:

| Component | Purpose |
|-----------|---------|
| `CausalGraph::save_to_file` / `load_from_file` equivalent | Export/import graph |
| `HnswService::save_to_file` / `load_from_file` | Export/import index |
| Edge-weight recalibration module | Adjust edge weights based on real observations |

### 6.5 Protocol

**Phase A -- Export from simulation**

1. After Experiment 5 converges, export the ECC causal graph:
   - Serialize all nodes and edges (JSON or bincode).
   - Export the HNSW index via `hnsw.save_to_file()`.
2. Record graph statistics: node count, edge count, community structure.

**Phase B -- Import to real robot**

3. Boot WeftOS on Pi 5 with Sesame robot connected.
4. Load the sim-trained causal graph and HNSW index.
5. Run `run_calibration()` to establish real-hardware tick cadence.
6. Map sim joint IDs to real servo IDs (the Walker2d model has 6 joints;
   the Sesame has 22 -- map only the 12 leg joints, freeze the rest).

**Phase C -- Calibration movements**

7. Execute 20 calibration movements:
   a. For each calibration step `c` in `[0..19]`:
      - Select an action from the sim-trained graph.
      - Execute on real robot (with harness).
      - Measure actual result (IMU stability, actual joint positions).
      - Compare real result to sim prediction.
      - Compute edge-weight correction factor:
        ```
        correction = real_stability / sim_predicted_stability
        ```
      - Update all causal edges involving the tested joints by
        multiplying their weight by the correction factor.
   b. Prioritize calibration movements that test the joints with the
      highest fan-out (most downstream causal edges).

**Phase D -- Walking test**

8. After 20 calibration movements, attempt continuous walking.
9. Measure: steps before first fall, gait quality metrics (same as
   Experiment 2).

**Phase E -- Comparison with from-scratch**

10. Compare all metrics against Experiment 2 results (from-scratch
    learning on the same robot).

### 6.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Calibration movements | number of test movements before walking attempt | exactly 20 |
| Steps before first fall (transfer) | after 20 calibrations | > 5 |
| Steps before first fall (from-scratch) | Experiment 2 result | comparison |
| Time to first stable step (transfer) | wall-clock from import to first step | < 5 min |
| Time to first stable step (from-scratch) | Experiment 2 result | > 30 min |
| Transfer speedup | scratch_time / transfer_time | > 6x |
| Edge weights changed | percentage of edges updated | < 30% |
| Graph structure changed | nodes/edges added or removed | 0 (hypothesis: structure transfers) |

### 6.7 Expected Results

- The sim-trained graph structure transfers intact: the same causal
  relationships hold in the real world (gravity, momentum, joint torque
  are universal).
- Only edge weights need updating: real servos are slower, have more
  friction, and different torque curves than simulated joints.
- After 20 calibration movements, the robot achieves 5+ consecutive
  steps -- a result that took 400+ attempts from scratch in Experiment 2.
- Total transfer time (import + 20 calibrations + first walk) is under
  10 minutes.

### 6.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Export from sim (depends on Exp 5) | 0.5 days | 1 |
| Import + joint mapping | 1 day | 1 |
| Calibration movements + walking test | 1 day | 1 |
| Comparison analysis | 0.5 days | 1 |
| **Total** | **3 days** | |

### 6.9 Success Criteria

- PASS: Stable step achieved within 20 calibration movements
- STRONG PASS: 10+ consecutive steps after transfer
- FAIL: Robot cannot walk after 50 calibration movements

### 6.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Sim dynamics too different from real | High | High | If transfer fails, do 50 additional calibration movements; report negative result honestly |
| Joint mapping errors | Medium | High | Verify mapping with manual single-joint tests first |
| HNSW index incompatible across platforms | Low | Medium | Use platform-independent serialization (JSON fallback) |
| Real servo speed limits invalidate sim timing | Medium | Medium | Scale all timing by measured servo speed ratio |

### 6.11 Publication Potential

- **Venue**: ICRA or CoRL (sim-to-real track)
- **Audience**: Sim-to-real researchers, domain adaptation community
- **Angle**: Explicit causal models as transferable representations --
  inspect and patch the reality gap instead of hoping it generalizes

---

## Experiment 7: Multi-Robot Skill Transfer

### 7.1 Hypothesis

Motor skills (specifically, "reaching" -- moving an end-effector to a
target position) learned by one robot can transfer to a kinematically
different robot via HNSW embedding space, requiring fewer than 30
adaptation movements versus learning from scratch.

### 7.2 Background

Robots in a fleet rarely have identical kinematic configurations. A
skill learned on Robot A (6-DOF arm) should be reusable on Robot B
(4-DOF arm) if the fundamental motor primitives (extend, retract,
rotate) are represented in a shared embedding space. HNSW provides this
shared space: motor primitives are stored as vectors that encode the
semantic meaning (direction, speed, force) rather than the raw joint
angles.

### 7.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Robot A: Sesame humanoid arm | 3-DOF (shoulder pitch/roll, elbow) | 1 | (shared) | $0 |
| Robot B: Custom 4-servo arm | 4-DOF (base rotate, shoulder, elbow, wrist) | 1 | $40 | $40 |
| Raspberry Pi 5 | (shared) | 1 | -- | $0 |
| PCA9685 | (shared) | 1 | -- | $0 |
| Target markers | 3D-printed cubes with ArUco, 5 positions | 5 | $1 | $5 |
| Camera | Pi Camera (shared) | 1 | -- | $0 |
| **Total (incremental)** | | | | **$45** |

### 7.4 Software Requirements

| Component | Purpose |
|-----------|---------|
| WeftOS kernel (ecc) | Causal graph + HNSW |
| Forward kinematics module (per robot) | Compute end-effector position from joint angles |
| Embedding encoder | Map (joint_deltas, end_effector_delta, speed) to 16-dim vector |
| Servo drivers | rppal + PCA9685 |

### 7.5 Protocol

**Phase A -- Robot A learns reaching**

1. Define 5 target positions in 3D space (within Robot A's workspace).
2. For each learning trial `t` in `[0..200]`:
   a. Select a random target from the 5 positions.
   b. Generate a random joint-angle trajectory (3 waypoints).
   c. Execute and measure final end-effector position (via camera +
      ArUco on the wrist).
   d. Compute reaching error: Euclidean distance from end-effector to
      target.
   e. Encode the movement as a 16-dim embedding:
      `[joint_deltas (3), end_effector_delta (3), speed (1), error (1),
       target_direction (3), padding (5)]`
   f. Insert into HNSW with metadata `{robot: "A", target, error}`.
   g. Create causal graph: `trajectory -> execution -> result`.
3. After 200 trials, Robot A can reach all 5 targets with < 5mm error.

**Phase B -- Export skill primitives**

4. Export the HNSW index and causal graph.
5. Filter: only export entries with error < 10mm (successful reaches).
6. Result: ~80-120 high-quality reaching primitives.

**Phase C -- Robot B imports and adapts**

7. Load the HNSW index into Robot B's WeftOS instance.
8. For each target position (5 targets):
   a. Query HNSW for the 5 nearest primitives matching the target
      direction.
   b. The retrieved primitives contain joint deltas for Robot A's 3 DOF.
      Map to Robot B's 4 DOF using the shared end-effector-delta
      components of the embedding.
   c. Execute the mapped trajectory on Robot B.
   d. Measure actual end-effector position.
   e. Compute error; create causal nodes linking the imported primitive
      to Robot B's execution.
   f. If error > threshold, adjust the mapping weights and re-insert
      the corrected primitive into HNSW.
9. Count total adaptation movements until all 5 targets are reached
   with < 10mm error.

**Phase D -- From-scratch baseline**

10. Clear HNSW and causal graph. Robot B learns reaching from scratch
    using the same protocol as Phase A. Count trials to < 10mm error
    on all 5 targets.

### 7.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Adaptation movements (transfer) | trials for Robot B to reach all 5 targets after import | < 30 |
| Learning trials (from scratch) | trials for Robot B to reach all 5 targets without transfer | ~150-200 |
| Transfer efficiency ratio | scratch_trials / transfer_trials | > 5x |
| Final reaching accuracy (transfer) | mean Euclidean error across 5 targets | < 10mm |
| Final reaching accuracy (scratch) | same, from-scratch | < 10mm |
| Shared embedding similarity | cosine sim between Robot A and Robot B primitives for same target | > 0.70 |
| Causal graph coupling | compute_coupling between imported and native subgraphs | > 0.5 |

### 7.7 Expected Results

- Robot B achieves < 10mm accuracy on all 5 targets within 20-30
  adaptation movements, versus 150-200 from scratch.
- The shared embedding space correctly captures the semantic similarity
  of reaching motions across different kinematic structures.
- The causal graph coupling score between imported and native primitives
  is high (> 0.5), confirming that the causal structure generalizes.

### 7.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Robot B assembly + FK calibration | 2 days | 1 |
| Phase A: Robot A training | 1 day | 1 |
| Phase B: Export | 0.5 days | 1 |
| Phase C: Transfer + adaptation | 1 day | 1 |
| Phase D: From-scratch baseline | 1 day | 1 |
| Analysis | 1 day | 1 |
| **Total** | **6.5 days** | |

### 7.9 Success Criteria

- PASS: Transfer efficiency ratio > 3x
- STRONG PASS: Transfer efficiency ratio > 5x with < 30 adaptations
- FAIL: Transfer efficiency ratio < 1.5x (transfer provides minimal benefit)

### 7.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Workspace mismatch (Robot B cannot reach Robot A's targets) | Medium | High | Pre-verify workspace overlap; use only reachable subset |
| Embedding space does not generalize across kinematics | Medium | High | Ablation: try pure end-effector embeddings (ignore joints) |
| Robot B has insufficient DOF for some motions | Low | Medium | Choose targets within Robot B's capability envelope |
| Camera calibration drift | Low | Low | Re-calibrate between Phase C and Phase D |

### 7.11 Publication Potential

- **Venue**: IROS (International Conference on Intelligent Robots and
  Systems)
- **Audience**: Multi-robot systems, transfer learning in robotics
- **Angle**: HNSW embedding spaces as universal skill representations
  across heterogeneous robot platforms

---

## Experiment 8: ECC Character Personality Emergence

### 8.1 Hypothesis

Different ECC initialization conditions combined with different
environmental experiences produce measurably different behavioral
personalities in game characters. Specifically: 10 identically
initialized ECC agents placed in the same environment with different
random encounter sequences will develop distinguishable behavioral
profiles after 1000 game ticks.

### 8.2 Background

Game NPCs typically exhibit personality through hand-authored behavior
trees or state machines. Emergent personality from experience would
make NPCs more varied and believable. ECC provides the substrate:
the causal graph structure grows differently based on which causal
relationships each agent discovers, and the HNSW index stores different
experiences.

### 8.3 Hardware

| Item | Specification | Qty | Unit Cost | Total |
|------|--------------|-----|-----------|-------|
| Development workstation | Any modern machine with 16+ GB RAM | 1 | $0 | $0 |
| **Total** | | | | **$0** |

### 8.4 Software Requirements

| Component | Crate / Package | Purpose |
|-----------|----------------|---------|
| WeftOS kernel (10 instances) | `clawft-kernel` (ecc) | One ECC per character |
| Game environment | Custom Rust grid world (40x40 cells) | Simulation arena |
| Alternative: Godot 4.3 | `godot-rust` GDExtension | 3D arena variant |
| Analysis tools | `petgraph`, `nalgebra` | Graph metrics, PCA |
| Visualization | `plotters` + `rerun` | Behavioral heatmaps, graph viz |

### 8.5 Protocol

**Phase A -- Environment design**

1. Create a 40x40 grid world with:
   - **Resources** (food, water): 20 locations, respawn after 50 ticks.
   - **Threats** (predators): 5 wandering agents, deterministic patrol.
   - **Other characters**: the 10 ECC agents + 10 "neutral" scripted NPCs.
   - **Shelter**: 8 safe zones where threats cannot enter.
   - **Exploration zones**: 4 areas with hidden bonuses (discovered by
     visiting).
2. Each ECC character has:
   - Health (0-100, decays 1/tick, restored by food).
   - Energy (0-100, decays 0.5/tick, restored by rest in shelter).
   - Inventory (up to 5 items).
   - Action space: move(N/S/E/W), gather, eat, rest, flee, interact.
3. Encounter sequence: each character receives a deterministic but unique
   sequence of events (seeded by character ID):
   - Character 0: early threat encounters (predators near spawn).
   - Character 1: early resource abundance (food near spawn).
   - Character 2: early social encounters (NPCs near spawn).
   - Characters 3-9: mixed, varying proportions.

**Phase B -- ECC agent implementation**

4. Each ECC agent's decision loop (per tick):
   a. Observe: 5x5 vision cone (cells in front of character).
   b. Encode observation as a 32-dim vector:
      `[health, energy, inventory_count, nearby_food, nearby_threat,
       nearby_shelter, nearby_npc, x_pos, y_pos, ..., padding]`
   c. Query HNSW for 5 nearest past observations.
   d. For each retrieved observation, look up the causal graph: what
      action was taken, and what was the outcome (health/energy delta)?
   e. Select action that maximizes expected health + energy:
      ```
      score(a) = sum(causal_weight * outcome for each retrieved
                     observation where action == a)
      ```
   f. Execute action, observe result.
   g. Create causal nodes: `observation -> action -> outcome`.
   h. Edge type: `Causes` if outcome was expected direction,
      `Contradicts` if outcome was opposite, `Inhibits` if action led to
      damage.
   i. Insert new observation into HNSW.

**Phase C -- Run simulation**

5. Run all 10 characters simultaneously for 1000 ticks.
6. Log per-tick: position, action, health, energy, causal graph size.
7. Snapshot causal graph and HNSW at ticks 250, 500, 750, 1000.

**Phase D -- Behavioral analysis**

8. For each character, compute behavioral profile:
   a. **Action distribution**: histogram of actions taken over 1000 ticks.
      Compute entropy: `H = -sum(p_a * log(p_a))`.
   b. **Spatial distribution**: heatmap of visited cells. Compute
      coverage (unique cells / total cells).
   c. **Risk profile**: ratio of time spent near threats vs shelter.
   d. **Social profile**: number of NPC interactions.
   e. **Exploration profile**: number of exploration zones discovered.
9. Classify each character into personality archetypes:
   - **Cautious**: low risk, high shelter time, avoids threats.
   - **Aggressive**: high risk, confronts threats, low shelter time.
   - **Curious**: high exploration, high coverage, moderate risk.
   - **Social**: high NPC interaction, moderate exploration.
   - **Survivalist**: optimal health/energy management, efficient paths.
10. Compute pairwise behavioral distance matrix (Jensen-Shannon
    divergence on action distributions).
11. Run PCA on the behavioral profile vectors; visualize the 10
    characters in 2D personality space.

**Phase E -- Graph structural analysis**

12. For each character's causal graph at tick 1000:
    a. Compute graph density (edges / possible edges).
    b. Run `detect_communities()` -- how many communities per character?
    c. Run `spectral_analysis()` -- eigenvalue spectrum as a fingerprint.
    d. Compute pairwise graph edit distance (approximate: compare edge
       type distributions).
13. Correlate graph structural metrics with behavioral profiles: do
    structurally different graphs produce behaviorally different
    characters?

**Phase F -- Human evaluation**

14. Record 30-second replay videos of each character's behavior.
15. Show 10 pairs (A, B) to 5 judges. For each pair, ask:
    - "Are these two characters behaving differently?" (yes/no)
    - "How would you describe each character's personality?" (free text)
16. Compute inter-rater agreement (Fleiss' kappa).

### 8.6 Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| Action distribution entropy (mean) | avg entropy across 10 characters | > 1.5 bits (diverse actions) |
| Action distribution divergence (mean pairwise JSD) | mean JSD across all 45 pairs | > 0.10 (distinguishable) |
| Spatial coverage variance | std_dev of coverage across characters | > 0.05 |
| Personality classification | number of distinct archetypes observed | >= 3 out of 5 |
| Graph community count (mean) | avg communities per character | > 3 |
| Graph spectral divergence | pairwise difference in eigenvalue spectra | measurable |
| Human judge agreement (kappa) | Fleiss' kappa on "different?" question | > 0.60 (substantial) |
| Human judge discrimination | % of pairs correctly identified as different | > 70% |
| Character survival rate | characters alive at tick 1000 | >= 8 out of 10 |
| HNSW entries per character | at tick 1000 | 200-500 |
| Causal graph nodes per character | at tick 1000 | 500-1500 |

### 8.7 Expected Results

- Characters with early threat exposure (Character 0) develop cautious
  behavior patterns: higher shelter usage, more flee actions, smaller
  explored area.
- Characters with early resource abundance (Character 1) develop
  broader exploration: they survived early, so they explored more, so
  their causal graph has more diverse experiences.
- At least 3 distinct personality archetypes emerge by tick 500.
- Human judges can distinguish characters with > 70% accuracy.
- The causal graph structural metrics (community count, spectral
  fingerprint) correlate with behavioral personality at r > 0.5.

### 8.8 Timeline

| Phase | Duration | Personnel |
|-------|---------|-----------|
| Grid world implementation | 3 days | 1 |
| ECC agent implementation | 2 days | 1 |
| Simulation runs + data collection | 1 day | 1 |
| Behavioral + structural analysis | 2 days | 1 |
| Human evaluation | 1 day | 5 judges + 1 facilitator |
| Final analysis + visualization | 1 day | 1 |
| **Total** | **10 days** | |

### 8.9 Success Criteria

- PASS: Mean pairwise JSD > 0.08 AND >= 3 archetypes AND human
  discrimination > 60%
- STRONG PASS: Mean pairwise JSD > 0.15 AND >= 4 archetypes AND human
  discrimination > 80%
- FAIL: Mean pairwise JSD < 0.05 (characters behave identically)

### 8.10 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Characters converge to identical optimal strategy | Medium | High | Ensure environment has multiple viable strategies (no single optimum) |
| Characters die before forming personalities | Medium | Medium | Add health floor (min 10) for first 200 ticks |
| HNSW retrieval latency slows simulation | Low | Medium | Use batch search; limit k to 5 |
| Grid world too simple for interesting behavior | Medium | Medium | Add Godot 3D variant as richer fallback |
| Human judges have no reference for "personality" | Low | Medium | Provide brief rubric with archetype descriptions |

### 8.11 Publication Potential

- **Venue**: AAAI Conference on Artificial Intelligence and Interactive
  Digital Entertainment (AIIDE), or Foundations of Digital Games (FDG)
- **Audience**: Game AI researchers, procedural generation, emergent
  narrative
- **Angle**: Emergent NPC personalities from causal experience graphs
  -- no hand-authored behavior trees required

---

## Cross-Experiment Dependencies

```
Experiment 1 (Servo Calibration)
    |
    v
Experiment 2 (Gait Learning) -------> Experiment 3 (Motion Mimicry)
    |                                      [shares robot hardware]
    v
Experiment 5 (Sim Gait) -----------> Experiment 6 (Sim-to-Real Transfer)
    |                                      [requires Exp 2 + Exp 5 results]
    |
    v
Experiment 7 (Multi-Robot Transfer)
    [shares robot from Exp 2, adds Robot B]

Experiment 4 (3D Print) ------------> independent
Experiment 8 (Personality) ---------> independent
```

**Critical path**: Exp 1 -> Exp 2 -> Exp 5 -> Exp 6 (total: 29 days).

**Parallelizable**: Exp 4 and Exp 8 can run concurrently with any other
experiment. Exp 3 can run concurrently with Exp 5 (after Exp 2 completes).

---

## Consolidated Bill of Materials

| Item | Experiments | Cost |
|------|------------|------|
| Raspberry Pi 5 (8 GB) | 1, 2, 3, 6, 7 | $80 |
| PCA9685 PWM driver (x2) | 1, 2, 7 | $12 |
| TowerPro SG90 servos (x4) | 1 | $12 |
| Sesame humanoid robot | 2, 3, 6, 7 | $450 |
| MPU-6050 IMU | 2, 3, 6 | $4 |
| Pi Camera Module 3 | 1, 4 | $25 |
| Logitech C920 webcam | 3 | $50 |
| Safety harness + gantry | 2, 6 | $60 |
| LiPo battery 7.4V 2200mAh | 2 | $18 |
| Foam crash mat | 2, 6 | $15 |
| USB-C power supply (Pi) | all | $12 |
| ArUco marker sheets | 1, 7 | $2 |
| Servo mounting brackets (3D) | 1 | $4 |
| 5V 3A power supply | 1 | $8 |
| Jumper wires + breadboard | all | $5 |
| Calibration weights | 1 | $12 |
| Creality Ender-3 V3 SE | 4 | $180 |
| MKS Gen L v2.1 | 4 | $25 |
| Ring light | 4 | $15 |
| PLA filament (2 kg) | 4 | $36 |
| Digital calipers | 4 | $12 |
| Custom 4-servo arm | 7 | $40 |
| Target markers (3D printed) | 7 | $5 |
| **Grand Total** | | **$1,102** |

Note: Development workstation assumed to be available (Experiments 5, 8).

---

## Consolidated Software Dependencies

| Crate / Package | Version | Experiments |
|----------------|---------|-------------|
| `clawft-kernel` (ecc feature) | workspace | all |
| `clawft-daemon` (TCP RPC) | workspace | 5, 6 |
| `clawft-exochain` | workspace | 1, 2 |
| `clawft-claw-stage` | workspace | 2 |
| `rppal` | >= 0.18 | 1, 2, 7 |
| `pwm-pca9685` | >= 0.4 | 1, 2, 7 |
| `opencv` (Rust bindings) | >= 0.92 | 1, 3, 4 |
| `nokhwa` | >= 0.14 | 1 (alt) |
| `mpu6050` | >= 0.2 | 2, 3, 6 |
| `plotters` | >= 0.3 | 1, 5, 8 |
| `rerun` | >= 0.18 | 2, 8 |
| `nalgebra` | >= 0.33 | 3, 8 |
| `petgraph` | >= 0.6 | 8 |
| `mujoco-rs` or Python bindings | 3.x | 5, 6 |
| `godot-rust` (GDExtension) | >= 0.2 | 5, 8 (alt) |
| Stable Baselines3 (Python) | >= 2.3 | 5 |
| Gymnasium (Python) | >= 1.0 | 5 |
| MediaPipe Pose (Python) | latest | 3 |
| PrusaSlicer CLI | >= 2.7 | 4 |
| OctoPrint | >= 1.10 | 4 |

---

## Master Timeline

```
Week 1:  Exp 1 (Servo Calibration)      | Exp 4 (3D Print - setup)
Week 2:  Exp 2 (Gait Learning - setup)  | Exp 4 (3D Print - runs)
Week 3:  Exp 2 (Gait Learning - runs)   | Exp 8 (Personality - build)
Week 4:  Exp 3 (Motion Mimicry)         | Exp 5 (Sim Gait)
Week 5:  Exp 6 (Sim-to-Real)            | Exp 8 (Personality - runs)
Week 6:  Exp 7 (Multi-Robot Transfer)   | Analysis + paper drafts
Week 7:  Buffer + re-runs               | Final analysis
```

Total calendar time: 7 weeks (with 2 parallel tracks).
Total person-days: ~63.5 across all experiments.

---

## Shared Infrastructure

### WeftOS Kernel Configuration for All Experiments

```toml
[ecc]
enabled = true
calibration_ticks = 30
tick_interval_ms = 20        # 50 Hz for real-time robotics
tick_budget_ratio = 0.3
vector_dimensions = 16       # compact for robotics (384 for NLP)

[hnsw]
m = 16                       # connections per node
ef_construction = 200        # build quality
ef_search = 50               # query quality
max_elements = 100_000       # upper bound for all experiments

[causal]
max_nodes = 200_000
max_edges = 500_000
enable_spectral = true
spectral_max_iterations = 100
```

### Data Collection Standard

All experiments log to ExoChain with the following event schema:

```json
{
  "experiment_id": "exp-{N}",
  "tick": 12345,
  "timestamp_us": 1712345678000000,
  "event_type": "observation|action|outcome|calibration",
  "payload": { ... },
  "blake3_hash": "abc123..."
}
```

Every event is BLAKE3-hashed and linked to the previous event in the
ExoChain, providing a tamper-evident audit trail for reproducibility.

### Reproducibility Checklist (All Experiments)

- [ ] Random seeds documented and fixed
- [ ] Hardware serial numbers / firmware versions recorded
- [ ] Room temperature logged at start and end
- [ ] WeftOS kernel version + git commit hash recorded
- [ ] EccCalibration result (p50, p95, headroom) logged at boot
- [ ] All raw data (camera frames, IMU logs, scores) stored with
      experiment ID prefix
- [ ] Causal graph and HNSW index snapshots saved at defined intervals
- [ ] Code used for each experiment tagged in git

---

## Risk Register (Cross-Experiment)

| ID | Risk | Experiments | Likelihood | Impact | Mitigation |
|----|------|-------------|-----------|--------|-----------|
| R1 | Pi 5 supply shortage | 1-3, 6-7 | Low | High | Order early; Jetson Nano as backup |
| R2 | Sesame robot delivery delay | 2, 3, 6, 7 | Medium | Critical | Order 8 weeks ahead; use simpler biped as fallback |
| R3 | WeftOS kernel regression breaks ECC | All | Low | Critical | Pin kernel to tested commit; CI gate |
| R4 | Researcher unavailable for 1+ weeks | All | Medium | High | Document all protocols thoroughly; any team member can run |
| R5 | MuJoCo licensing changes | 5, 6 | Low | Medium | Godot physics as alternative (already specified) |
| R6 | Negative results on key experiments | 2, 5, 6 | Medium | Medium | Negative results are publishable; document what we learn |

---

## Publication Strategy

| Experiment | Target Venue | Submission Deadline | Paper Type |
|-----------|-------------|-------------------|-----------|
| 1 (Servo) | ICRA Workshop | Jan 2027 | 4-page extended abstract |
| 2 (Gait) | RSS | Mar 2027 | 8-page full paper |
| 3 (Mimicry) | CoRL | Jun 2027 | 8-page full paper |
| 4 (3D Print) | Additive Mfg Journal | Rolling | Full journal article |
| 5 (Sim Gait) | NeurIPS Workshop | May 2027 | 4-page workshop paper |
| 6 (Sim-to-Real) | ICRA | Sep 2026 | 6-page conference paper |
| 7 (Multi-Robot) | IROS | Mar 2027 | 6-page conference paper |
| 8 (Personality) | AIIDE | May 2027 | 8-page full paper |

Combined narrative paper (all 8 experiments):
- **Venue**: Science Robotics or Nature Machine Intelligence
- **Title**: "Embodied Causal Cognition: From Servo Calibration to
  Emergent Personality via Causal Graphs and Approximate Nearest
  Neighbors"
- **Angle**: A unified cognitive architecture that spans physical
  calibration, locomotion, imitation, manufacturing, simulation, and
  game AI -- all using the same two data structures (DAG + HNSW).
