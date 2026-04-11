# Integration Architecture: WeftOS Gaming and Robotics Symposium

| Field | Value |
|-------|-------|
| Status | Draft |
| Date | 2026-04-04 |
| Authors | Integration Architecture Team |
| Depends on | ADR-021 (daemon-first RPC), ADR-047 (self-calibrating tick), ADR-049 (tiered kernel profiles) |
| Crates | clawft-kernel, clawft-types, clawft-core, clawft-edge-bench |

## Abstract

This document defines how WeftOS bridges the gap between cognitive reasoning
(the ECC causal graph, HNSW vector search, ExoChain governance) and physical
or simulated actuation. The core insight is that a robot arm, a game character,
and a CLI conversation are all instances of the same abstract loop:
**Perceive -- Think -- Act**. The ECC already implements the Think phase. This
architecture adds the Perceive (sensor) and Act (actuator) layers, connects
them through the causal graph, and defines the protocols, safety gates, and
performance budgets that make the whole system viable on hardware ranging from
an ESP32 sensor node to a multi-node server cluster.

---

## 1. The Perceive-Think-Act Framework

### 1.1 Formal Definition

The Perceive-Think-Act (PTA) loop is a three-phase control cycle that maps
directly onto the DEMOCRITUS cognitive tick already implemented in the ECC
subsystem of `clawft-kernel`.

```
PERCEIVE        THINK               ACT
---------       ---------------     ---------
Sensors    -->  CausalGraph    -->  Actuators
               + HNSW search
               + Governance
```

**PERCEIVE.** Sample all registered sensors. Each reading is converted into
a fixed-dimensional embedding vector and inserted into the HNSW store. A
corresponding `CausalNode` is created in the causal graph with edge type
`EvidenceFor` linking the new observation to the nearest existing nodes
found by vector search.

**THINK.** The DEMOCRITUS cycle runs:
1. Gather recent `CausalNode` entries (the "working set").
2. Perform HNSW similarity search against the working set to retrieve
   relevant historical nodes (motor memory, past observations).
3. Traverse causal edges (`Causes`, `Enables`, `Inhibits`) to build a
   local causal model of the current situation.
4. Propose an action by selecting a motion primitive or generating a new
   command vector.
5. Submit the proposed action through the governance gate for safety
   checking.

**ACT.** If the governance gate approves, the command is dispatched to
registered actuators. Actuator feedback is immediately re-ingested as a
new PERCEIVE sample, closing the loop.

### 1.2 How ECC Implements Each Phase

The existing ECC structures in `clawft-kernel/src/causal.rs` provide the
backbone:

- **CausalGraph** (concurrent DashMap-backed DAG) stores the reasoning
  state. Nodes represent observations, hypotheses, and action records.
  Edges encode causal relationships with weights and ExoChain sequence
  numbers for provenance.

- **CausalEdgeType** variants map to PTA phases:
  - `EvidenceFor` -- sensor readings supporting a hypothesis (PERCEIVE)
  - `Causes`, `Enables`, `Inhibits` -- reasoning links (THINK)
  - `TriggeredBy` -- action-to-cause provenance (ACT)
  - `Follows` -- temporal sequencing across all phases

- **HnswService** (`clawft-kernel/src/hnsw_service.rs`) provides the
  similarity search substrate. Its `search_batch` method is critical for
  the THINK phase: it acquires the mutex once and processes all working-set
  queries in a single lock hold, avoiding per-query overhead during the
  tight cognitive tick.

- **VectorBackend** (`clawft-kernel/src/vector_backend.rs`) abstracts the
  storage engine. The `insert_with_epoch` method provides optimistic
  concurrency control, ensuring that concurrent sensor ingestion and
  cognitive reasoning do not produce silent conflicts.

### 1.3 Loop Speed Tiers

Not all control loops need the same latency. The architecture defines four
tiers, each with its own tick rate, computational budget, and appropriate
use cases.

| Tier | Name | Period | Latency Budget | Use Case |
|------|------|--------|----------------|----------|
| T-reflex | Reflex | 0.5ms | 0.3ms | Emergency stop, collision avoidance, current limiting |
| T-servo | Servo | 5ms | 3ms | PID joint control, closed-loop position tracking |
| T-plan | Planning | 50ms | 30ms | Motion primitive selection, HNSW search, causal reasoning |
| T-strategy | Strategy | 500ms--5s | 200ms--3s | Goal planning, LLM inference, cross-domain transfer |

**T-reflex** runs outside the ECC entirely. It is a hard-real-time interrupt
handler on the microcontroller (ESP32 or Pi GPIO) that reads limit switches,
current sensors, and encoders and applies immediate corrective action. No
HNSW search, no causal reasoning. This is the spinal cord.

**T-servo** runs a simple PID loop on the host CPU (Pi 5) or offloaded to
a dedicated motor controller (MKS Gen L running Marlin). It consumes
target positions produced by T-plan and drives the physical actuator toward
them. Sensor feedback from encoders and current sensors is buffered for
T-plan ingestion.

**T-plan** is where the ECC DEMOCRITUS cycle operates. It reads buffered
sensor data, performs HNSW search, updates the causal graph, selects or
composes a motion primitive, and submits it to the governance gate. This
is the midbrain.

**T-plan** aligns with the self-calibrating tick from ADR-047. The kernel
measures its own tick latency and adjusts the planning period dynamically.
On a Pi 5 with 50K HNSW vectors, the measured p95 cognitive tick is under
30ms, leaving margin within the 50ms budget.

**T-strategy** handles high-level deliberation: "What should I do next?"
This tier may involve LLM inference (via cloud or local model), multi-step
causal chain analysis, and goal replanning. It communicates with T-plan by
updating the goal state in the causal graph, which T-plan reads on its
next tick.

### 1.4 The Human-as-Actuator Model

In a CLI conversation, the "actuator" is language output to a human, and
the "sensor" is text input from the human. This is the slowest PTA loop
(seconds to minutes per cycle), but it uses the same infrastructure:

- `ConversationActuator` sends a message (ACT)
- `CodebaseSensor` watches for file changes or user input (PERCEIVE)
- The ECC reasons about the conversation history using the causal graph
  and HNSW search against a knowledge base (THINK)

This model means that a WeftOS instance controlling a robot arm and a
WeftOS instance running a CLI conversation use the same kernel, the same
causal graph, and the same governance pipeline. The only difference is
which `Sensor` and `Actuator` implementations are registered.

---

## 2. Actuator Abstraction Layer

### 2.1 Core Trait

The `clawft-actuator` module defines a generic interface for anything that
receives commands and optionally returns feedback.

```rust
/// A device or interface that receives commands and produces feedback.
///
/// Implementations must be `Send + Sync` because the control loop
/// dispatches commands from async tasks.
pub trait Actuator: Send + Sync {
    /// The command type this actuator accepts.
    type Command: Send + Clone + 'static;

    /// The feedback type this actuator produces.
    type Feedback: Send + Clone + 'static;

    /// Send a command to the actuator.
    ///
    /// Returns immediately after the command is queued. Actual execution
    /// may be deferred to the next hardware cycle.
    fn send(&self, cmd: Self::Command) -> Result<(), ActuatorError>;

    /// Read the most recent feedback from the actuator.
    ///
    /// Non-blocking. Returns `None` if no feedback is available yet.
    fn read(&self) -> Result<Option<Self::Feedback>, ActuatorError>;

    /// Declare the actuator's capabilities and constraints.
    fn capabilities(&self) -> ActuatorCapabilities;

    /// Emergency stop. Immediately cease all motion and enter safe state.
    ///
    /// Implementations MUST make this non-blocking and infallible.
    fn emergency_stop(&self);

    /// Human-readable name for logging and diagnostics.
    fn name(&self) -> &str;
}

/// Static capabilities declaration for governance checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorCapabilities {
    /// Maximum force or torque the actuator can apply (Nm or N).
    pub max_force: f64,
    /// Maximum velocity (rad/s for rotary, m/s for linear).
    pub max_velocity: f64,
    /// Position range [min, max] (radians for rotary, meters for linear).
    pub position_range: (f64, f64),
    /// Whether the actuator supports continuous rotation.
    pub continuous: bool,
    /// Minimum command period (how often the actuator accepts new commands).
    pub min_command_period_us: u64,
    /// Whether the actuator has built-in safety limits (e.g., current cutoff).
    pub has_hardware_limits: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ActuatorError {
    #[error("actuator communication failed: {0}")]
    Communication(String),
    #[error("command rejected by hardware limits: {0}")]
    LimitExceeded(String),
    #[error("actuator in emergency stop state")]
    EmergencyStopped,
    #[error("actuator not initialized")]
    NotInitialized,
    #[error("{0}")]
    Other(String),
}
```

### 2.2 ServoActuator -- PCA9685 PWM Servo Control

Controls hobby servos via the PCA9685 16-channel PWM driver over I2C.
Deployed on Raspberry Pi 5 with up to 62 chained PCA9685 boards (992
servo channels).

```rust
pub struct ServoActuator {
    i2c_bus: I2cBus,
    address: u8,
    channel: u8,
    min_pulse_us: u16,   // typically 500
    max_pulse_us: u16,   // typically 2500
    angle_range: (f64, f64), // (0.0, 180.0) for standard, (-90.0, 90.0) for centered
    current_angle: AtomicF64,
}

pub struct ServoCommand {
    pub target_angle: f64,
    pub speed: Option<f64>,  // degrees per second, None = instant
}

pub struct ServoFeedback {
    pub current_angle: f64,      // last commanded (no encoder feedback on hobby servos)
    pub estimated_arrival: bool, // true when enough time has elapsed for movement
}
```

**I2C communication.** The PCA9685 operates at 50Hz PWM frequency. Each
channel maps to a 12-bit duty cycle register. The driver translates
angle -> pulse width -> duty cycle:

```
pulse_us = min_pulse + (angle - angle_min) / (angle_max - angle_min) * (max_pulse - min_pulse)
duty = pulse_us / 20000.0 * 4096.0
```

**Limitations.** Hobby servos provide no position feedback. The actuator
tracks the last commanded angle and estimates arrival based on the servo's
rated speed (typically 60 degrees in 0.12s at no load). Real feedback
requires external position sensors (see Section 3.5).

### 2.3 StepperActuator -- G-code Streaming to MKS Gen L

Controls stepper motors via G-code commands sent over serial UART to an
MKS Gen L board running Marlin firmware.

```rust
pub struct StepperActuator {
    serial: SerialPort,
    baud_rate: u32,  // 115200 typical
    axis: char,      // 'X', 'Y', 'Z', 'E'
    steps_per_mm: f64,
    max_feedrate_mm_min: f64,
    position: AtomicF64,
    buffer_depth: AtomicU32,  // Marlin command buffer occupancy
}

pub struct StepperCommand {
    /// Target position in millimeters (absolute).
    pub target_mm: f64,
    /// Feedrate in mm/min. Clamped to max_feedrate.
    pub feedrate: f64,
}

pub struct StepperFeedback {
    pub reported_position: f64,
    pub buffer_free: u32,
    pub endstop_triggered: bool,
}
```

**Flow control.** Marlin has a 16-command buffer. The actuator tracks
buffer occupancy by counting sent commands and received `ok` responses.
It blocks `send()` when the buffer is full to prevent command loss. The
`M114` command retrieves the current position for feedback.

**G-code mapping:**
- `G1 X{target} F{feedrate}` -- linear move
- `G28 X` -- home axis
- `M84` -- disable steppers (release)
- `M112` -- emergency stop

### 2.4 GameJointActuator -- TCP to Game Engine

Controls a bone/joint in a game engine character via TCP socket. This is
the primary actuator for the gaming domain.

```rust
pub struct GameJointActuator {
    connection: Arc<TcpConnection>,
    joint_id: String,
    character_id: String,
    angle_limits: (f64, f64),
}

pub struct GameJointCommand {
    pub target_angle: f64,
    pub velocity: f64,       // how fast to interpolate (degrees/sec)
    pub blend_weight: f32,   // 0.0-1.0 for animation blending
}

pub struct GameJointFeedback {
    pub current_angle: f64,
    pub applied_torque: f64,
    pub contact: bool,       // is this joint's limb in contact with something?
}
```

**Protocol.** See Section 5 for the full game engine bridge protocol.
The `GameJointActuator` serializes `GameJointCommand` into an
`ActionFrame` message and sends it on the shared TCP connection. Feedback
arrives as part of the next `PerceptionFrame` from the game engine.

### 2.5 ConversationActuator -- Language Output

The "slow actuator" for CLI and chat interfaces.

```rust
pub struct ConversationActuator {
    output_channel: mpsc::Sender<String>,
    pending_response: AtomicBool,
}

pub struct ConversationCommand {
    pub message: String,
    pub format: OutputFormat,  // Plain, Markdown, Json
}

pub struct ConversationFeedback {
    pub acknowledged: bool,
    pub user_response: Option<String>,
}
```

The conversation actuator does not participate in real-time control loops.
Its `min_command_period_us` is set to 100ms (10Hz), reflecting the rate
at which text can reasonably be streamed to a terminal.

### 2.6 HttpActuator -- REST API Calls

Controls IoT devices, cloud services, and external systems via HTTP.

```rust
pub struct HttpActuator {
    client: reqwest::Client,
    base_url: String,
    auth: Option<AuthConfig>,
    timeout: Duration,
}

pub struct HttpCommand {
    pub method: Method,
    pub path: String,
    pub body: Option<serde_json::Value>,
    pub headers: HashMap<String, String>,
}

pub struct HttpFeedback {
    pub status: u16,
    pub body: serde_json::Value,
    pub latency_ms: u64,
}
```

Use cases: turning on lights (Philips Hue), opening valves (industrial
IoT), triggering webhooks, calling cloud AI inference endpoints.

---

## 3. Sensor Abstraction Layer

### 3.1 Core Trait

```rust
/// A device or interface that produces readings.
///
/// Implementations must be `Send + Sync` for use in async control loops.
pub trait Sensor: Send + Sync {
    /// The reading type this sensor produces.
    type Reading: Send + Clone + Into<SensorEmbedding> + 'static;

    /// Take a single sample from the sensor.
    ///
    /// May block for up to one sample period. Returns an error if the
    /// sensor is disconnected or misconfigured.
    fn sample(&self) -> Result<Self::Reading, SensorError>;

    /// The sensor's native sample rate in Hz.
    fn sample_rate_hz(&self) -> f64;

    /// Dimensionality of the embedding produced by this sensor's readings.
    ///
    /// This determines the vector size inserted into the HNSW store.
    fn dimensions(&self) -> usize;

    /// Human-readable name for logging and diagnostics.
    fn name(&self) -> &str;

    /// Whether the sensor is currently connected and producing valid readings.
    fn is_healthy(&self) -> bool;
}

/// A fixed-dimensional embedding produced from a sensor reading.
///
/// All sensor readings are converted to this common representation
/// before insertion into the HNSW store.
#[derive(Debug, Clone)]
pub struct SensorEmbedding {
    pub vector: Vec<f32>,
    pub timestamp: u64,
    pub source: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum SensorError {
    #[error("sensor disconnected: {0}")]
    Disconnected(String),
    #[error("sensor read timeout after {0}ms")]
    Timeout(u64),
    #[error("sensor calibration required")]
    NeedsCalibration,
    #[error("{0}")]
    Other(String),
}
```

### 3.2 ImuSensor -- 6-DOF Accelerometer/Gyroscope

Reads a 6-axis IMU (e.g., MPU-6050, ICM-20948) over I2C or SPI.

```rust
pub struct ImuSensor {
    bus: I2cBus,
    address: u8,
    sample_rate: f64,      // typically 100-1000 Hz
    accel_range: AccelRange, // 2g, 4g, 8g, 16g
    gyro_range: GyroRange,   // 250, 500, 1000, 2000 dps
}

pub struct ImuReading {
    pub accel: [f32; 3],   // m/s^2 (X, Y, Z)
    pub gyro: [f32; 3],    // rad/s (roll, pitch, yaw rates)
    pub temperature: f32,  // Celsius
}
```

**Embedding strategy.** The 6-DOF reading is embedded as a 6-dimensional
vector `[ax, ay, az, gx, gy, gz]` after normalization to the [-1, 1]
range based on the configured sensor ranges. This preserves the relative
magnitudes while making similarity search meaningful: two readings with
similar acceleration and rotation profiles will be nearby in HNSW space.

For the T-plan tier, IMU readings are typically downsampled from the
native rate (100-1000 Hz) to the planning rate (20 Hz). A sliding-window
average or median filter is applied during downsampling.

### 3.3 CameraSensor -- Frame Capture and Pose Estimation

```rust
pub struct CameraSensor {
    device: String,       // e.g., "/dev/video0"
    resolution: (u32, u32),
    fps: f64,
    pose_estimator: Option<PoseEstimator>,
}

pub struct CameraReading {
    /// Raw frame data (compressed JPEG to save memory).
    pub frame: Option<Vec<u8>>,
    /// Estimated pose keypoints (if pose estimator is configured).
    pub keypoints: Option<Vec<Keypoint>>,
    /// Detected objects with bounding boxes.
    pub detections: Vec<Detection>,
}

pub struct Keypoint {
    pub name: String,     // "left_shoulder", "right_elbow", etc.
    pub x: f32,
    pub y: f32,
    pub confidence: f32,
}
```

**Embedding strategy.** Camera readings are not embedded directly into
HNSW as raw pixels. Instead, the pose estimator extracts keypoint
positions which are concatenated into a vector:
`[lshoulder_x, lshoulder_y, rshoulder_x, ..., confidence_avg]`.
For a 17-keypoint model (COCO format), this produces a 35-dimensional
embedding (2 coordinates per keypoint + 1 aggregate confidence).

**Memory budget.** Raw frames are NOT stored in the HNSW store. Only the
derived keypoint embedding is stored. The raw frame is available for one
tick (held in a ring buffer of configurable depth) for debugging and
visualization, then discarded.

### 3.4 CurrentSensor -- Motor Current as Effort Proxy

```rust
pub struct CurrentSensor {
    adc_channel: AdcChannel,
    shunt_resistance: f64,  // ohms
    gain: f64,              // amplifier gain
    max_amps: f64,          // full-scale reading
}

pub struct CurrentReading {
    pub amps: f64,
    pub watts: f64,  // computed if voltage is known
}
```

Motor current is a proxy for effort/torque. A sudden spike in current
indicates the actuator has encountered resistance (e.g., collision,
stall). The T-reflex tier monitors current for emergency cutoff.

**Embedding.** Single-dimensional: `[normalized_current]`. Useful for
anomaly detection: the HNSW store accumulates a history of "normal"
current profiles during motion primitives, and a current reading far
from any stored vector triggers a causal `Contradicts` edge.

### 3.5 PositionSensor -- Servo Feedback / Encoder

```rust
pub struct PositionSensor {
    source: PositionSource,
    resolution: f64,  // degrees per count (for encoders)
}

pub enum PositionSource {
    /// Analog potentiometer feedback from servo (via ADC).
    AnalogFeedback { adc_channel: AdcChannel },
    /// Quadrature encoder on motor shaft.
    QuadratureEncoder { pin_a: GpioPin, pin_b: GpioPin },
    /// Absolute encoder (e.g., AS5048A) over SPI.
    AbsoluteEncoder { spi_bus: SpiBus, cs_pin: GpioPin },
}

pub struct PositionReading {
    pub angle_degrees: f64,
    pub velocity_dps: f64,  // degrees per second (computed from delta)
}
```

### 3.6 GameStateSensor -- Game Engine State over TCP

```rust
pub struct GameStateSensor {
    connection: Arc<TcpConnection>,
    character_id: String,
    last_frame: Mutex<Option<PerceptionFrame>>,
}

pub struct GameStateReading {
    pub joints: Vec<JointState>,
    pub contacts: Vec<ContactPoint>,
    pub world_position: [f64; 3],
    pub world_rotation: [f64; 4],  // quaternion
}
```

Receives `PerceptionFrame` messages from the game engine (see Section 5).
The reading is converted to an embedding by concatenating all joint angles
into a single vector, providing a compact representation of the
character's current pose.

### 3.7 CodebaseSensor -- File System Watcher

The "software sensor" for CLI/development workflows.

```rust
pub struct CodebaseSensor {
    watcher: RecommendedWatcher,
    root_path: PathBuf,
    event_buffer: Mutex<VecDeque<FileEvent>>,
}

pub struct CodebaseReading {
    pub events: Vec<FileEvent>,
    pub file_count: usize,
    pub recent_changes: Vec<String>,
}
```

This sensor does not produce fixed-dimensional embeddings directly.
Instead, file change events are converted to text embeddings using the
same embedding pipeline used for document ingestion in the knowledge
graph.

---

## 4. ECC Integration: Sensor to CausalGraph to Actuator

### 4.1 SensorNode

A `SensorNode` wraps a `Sensor`, handles embedding, and creates causal
graph entries.

```rust
pub struct SensorNode<S: Sensor> {
    sensor: S,
    node_prefix: String,
    hnsw: Arc<HnswService>,
    graph: Arc<CausalGraph>,
    sample_counter: AtomicU64,
    embedding_cache: Mutex<VecDeque<(u64, Vec<f32>)>>, // (node_id, embedding)
}

impl<S: Sensor> SensorNode<S> {
    /// Sample the sensor, embed the reading, and create a CausalNode.
    ///
    /// Returns the new node's ID in the causal graph.
    pub fn tick(&self) -> Result<NodeId, SensorError> {
        // 1. Sample
        let reading = self.sensor.sample()?;
        let embedding: SensorEmbedding = reading.into();

        // 2. Search for similar past readings
        let neighbors = self.hnsw.search(&embedding.vector, 5);

        // 3. Insert into HNSW
        let seq = self.sample_counter.fetch_add(1, Ordering::SeqCst);
        let key = format!("{}:{}", self.node_prefix, seq);
        self.hnsw.insert(
            key.clone(),
            embedding.vector.clone(),
            embedding.metadata.clone(),
        );

        // 4. Create CausalNode
        let node_id = self.graph.add_node(
            key,
            serde_json::json!({
                "source": embedding.source,
                "timestamp": embedding.timestamp,
                "sensor_type": std::any::type_name::<S>(),
            }),
        );

        // 5. Link to nearest neighbors with EvidenceFor edges
        for neighbor in &neighbors {
            // Parse the neighbor's node_id from its HNSW key
            if let Some(target_id) = parse_node_id(&neighbor.id) {
                self.graph.link(
                    node_id,
                    target_id,
                    CausalEdgeType::EvidenceFor,
                    neighbor.score, // similarity as weight
                    embedding.timestamp,
                    0, // chain_seq assigned by governance
                );
            }
        }

        // 6. Cache for ActuatorNode feedback matching
        let mut cache = self.embedding_cache.lock().unwrap();
        cache.push_back((node_id, embedding.vector));
        if cache.len() > 100 {
            cache.pop_front();
        }

        Ok(node_id)
    }
}
```

### 4.2 ActuatorNode

An `ActuatorNode` wraps an `Actuator`, receives commands from the causal
reasoning process, and records outcomes.

```rust
pub struct ActuatorNode<A: Actuator> {
    actuator: A,
    node_prefix: String,
    graph: Arc<CausalGraph>,
    governance: Arc<GovernanceGate>,
    last_command_node: AtomicU64,
}

impl<A: Actuator> ActuatorNode<A> {
    /// Execute a command after governance approval.
    ///
    /// Creates a CausalNode for the command, submits it to the governance
    /// gate, and dispatches to the actuator if approved.
    pub fn execute(
        &self,
        cmd: A::Command,
        cause_node: NodeId,
        effect_vector: EffectVector,
    ) -> Result<NodeId, ActuatorError> {
        // 1. Create command node in causal graph
        let cmd_node = self.graph.add_node(
            format!("{}:cmd", self.node_prefix),
            serde_json::json!({
                "actuator": self.actuator.name(),
                "effect_vector": effect_vector,
            }),
        );

        // 2. Link cause -> command
        self.graph.link(
            cause_node,
            cmd_node,
            CausalEdgeType::Causes,
            1.0,
            0, // timestamp set by graph
            0, // chain_seq
        );

        // 3. Governance check
        let capabilities = self.actuator.capabilities();
        let approval = self.governance.check(
            &effect_vector,
            &capabilities,
        );

        if !approval.approved {
            // Record rejection in causal graph
            self.graph.add_node(
                format!("{}:rejected", self.node_prefix),
                serde_json::json!({
                    "reason": approval.reason,
                    "effect_vector": effect_vector,
                }),
            );
            return Err(ActuatorError::LimitExceeded(
                approval.reason.unwrap_or_default(),
            ));
        }

        // 4. Dispatch to actuator
        self.actuator.send(cmd)?;
        self.last_command_node.store(cmd_node, Ordering::SeqCst);

        // 5. Record approval on ExoChain
        // (chain_seq will be assigned by the ExoChain append)

        Ok(cmd_node)
    }
}
```

### 4.3 ControlLoop

The `ControlLoop` orchestrates the full PTA cycle.

```rust
pub struct ControlLoop {
    sensors: Vec<Box<dyn AnySensorNode>>,
    actuators: Vec<Box<dyn AnyActuatorNode>>,
    planner: Box<dyn MotionPlanner>,
    graph: Arc<CausalGraph>,
    hnsw: Arc<HnswService>,
    governance: Arc<GovernanceGate>,
    tick_rate: Duration,
    tier: LoopTier,
}

pub enum LoopTier {
    Reflex,   // 0.5ms -- no ECC involvement
    Servo,    // 5ms   -- PID only
    Planning, // 50ms  -- full DEMOCRITUS
    Strategy, // 500ms+ -- LLM-assisted
}

impl ControlLoop {
    /// Run one complete PTA cycle.
    pub async fn tick(&mut self) -> Result<TickResult, ControlError> {
        let tick_start = Instant::now();

        // ── PERCEIVE ─────────────────────────────────────────
        let mut observations = Vec::new();
        for sensor in &self.sensors {
            match sensor.tick() {
                Ok(node_id) => observations.push(node_id),
                Err(e) => log::warn!("Sensor {} failed: {}", sensor.name(), e),
            }
        }

        // ── THINK ────────────────────────────────────────────
        // Build working set from recent observations
        let working_set: Vec<&[f32]> = self.collect_recent_embeddings(&observations);

        // Batch HNSW search for all observations
        let search_results = self.hnsw.search_batch(
            &working_set,
            10, // top_k per query
        );

        // Run planner to select action
        let action = self.planner.plan(
            &observations,
            &search_results,
            &self.graph,
        )?;

        // ── GOVERNANCE GATE ──────────────────────────────────
        // (handled inside ActuatorNode::execute)

        // ── ACT ──────────────────────────────────────────────
        for (actuator_idx, command) in action.commands.iter() {
            if let Some(actuator) = self.actuators.get(*actuator_idx) {
                actuator.execute_dyn(command, action.cause_node, action.effect_vector.clone())?;
            }
        }

        let tick_duration = tick_start.elapsed();
        Ok(TickResult {
            observations: observations.len(),
            actions: action.commands.len(),
            tick_duration,
            within_budget: tick_duration < self.tick_rate,
        })
    }
}
```

### 4.4 How Sensor Readings Become Causal Edges

The transformation from raw sensor data to causal structure follows a
three-step pipeline:

1. **Embedding.** The sensor reading is converted to a fixed-dimensional
   vector via the `Into<SensorEmbedding>` implementation. Different
   sensors produce different dimensionalities, but all go through the
   same HNSW insertion path.

2. **Similarity linking.** After HNSW insertion, the nearest neighbors
   are retrieved. Each neighbor represents a past observation. A
   `CausalEdgeType::EvidenceFor` edge is created from the new node to
   each neighbor, with the cosine similarity score as the edge weight.

3. **Anomaly detection.** If no neighbors are found within a configurable
   similarity threshold (default: 0.5), the new observation is flagged
   as anomalous. An `Inhibits` edge is created pointing to the current
   plan hypothesis, signaling that the observed state contradicts the
   expected outcome.

This pipeline means the causal graph naturally accumulates a history of
"what happened when" with similarity-based cross-references. The THINK
phase traverses these edges to find patterns: "every time the current
sensor spiked (node A), the joint position deviated (node B), which
caused a failed grasp (node C)."

### 4.5 How Causal Reasoning Produces Actuator Commands

The planner traverses the causal graph to produce commands:

1. **Goal identification.** The current goal is stored as a CausalNode
   with a special label prefix `goal:`. The planner finds the active
   goal node.

2. **Backward chaining.** From the goal, the planner follows reverse
   `Enables` and `Causes` edges to identify the preconditions that must
   be satisfied.

3. **State matching.** The planner compares current sensor observations
   (via HNSW search) against the preconditions. Unsatisfied preconditions
   become subgoals.

4. **Primitive selection.** For each subgoal, the planner searches HNSW
   for stored motion primitives whose expected initial state matches the
   current state. The best-matching primitive is selected.

5. **Command generation.** The selected primitive's command sequence is
   instantiated with the current joint positions and dispatched through
   the ActuatorNode.

### 4.6 The Governance Gate Between THINK and ACT

Every command passes through the governance gate before reaching an
actuator. This is the ECC's equivalent of the ExoChain governance layer.

```rust
pub struct GovernanceGate {
    rules: Vec<Box<dyn GovernanceRule>>,
    emergency_stop: AtomicBool,
}

pub struct GovernanceDecision {
    pub approved: bool,
    pub reason: Option<String>,
    pub modified_effect: Option<EffectVector>,
}

pub trait GovernanceRule: Send + Sync {
    fn check(
        &self,
        effect: &EffectVector,
        capabilities: &ActuatorCapabilities,
    ) -> GovernanceDecision;
}
```

Built-in rules:
- **JointLimitRule** -- rejects commands that exceed position range
- **VelocityLimitRule** -- caps velocity to safe maximums
- **CurrentLimitRule** -- rejects if predicted current exceeds threshold
- **CollisionRule** -- uses the causal graph to check if similar past
  commands resulted in collision events
- **EmergencyStopRule** -- always rejects when e-stop is active

---

## 5. Game Engine Bridge Protocol

### 5.1 Wire Format

The bridge uses a length-prefixed MessagePack encoding over TCP. JSON-lines
was considered but rejected due to:
- MessagePack is 30-50% smaller for typical game state messages
- Parsing is faster (no UTF-8 validation, no escape handling)
- Binary data (camera frames) can be included without base64 overhead

```
+----------+----------+----------------+
| len (4B) | type (1B)| payload (msgp) |
+----------+----------+----------------+
```

- `len`: 32-bit big-endian unsigned integer, total message size including
  the type byte and payload (excludes the 4 length bytes themselves).
- `type`: message type discriminator.
- `payload`: MessagePack-encoded body.

Message types:

| Type | Value | Direction | Description |
|------|-------|-----------|-------------|
| PerceptionFrame | 0x01 | Game -> WeftOS | Current state of all joints, contacts, camera |
| ActionFrame | 0x02 | WeftOS -> Game | Target angles and velocities for joints |
| SyncRequest | 0x03 | Either | Clock synchronization handshake |
| SyncResponse | 0x04 | Either | Clock synchronization response |
| CharacterRegister | 0x05 | Game -> WeftOS | Register a new character for control |
| CharacterRelease | 0x06 | Either | Release control of a character |
| Heartbeat | 0x07 | Either | Connection keepalive |
| Error | 0xFF | Either | Error notification |

### 5.2 PerceptionFrame

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerceptionFrame {
    /// Monotonic timestamp in microseconds (game clock).
    pub timestamp_us: u64,
    /// Character identifier.
    pub character_id: String,
    /// Current state of all controlled joints.
    pub joints: Vec<JointState>,
    /// Active contact points (collisions, ground contact).
    pub contacts: Vec<ContactPoint>,
    /// Optional camera frame (compressed JPEG, max 64KB).
    pub camera: Option<Vec<u8>>,
    /// Physics simulation step number.
    pub physics_step: u64,
    /// Time since last frame in seconds (for delta calculations).
    pub dt: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JointState {
    pub id: String,
    pub angle: f64,          // radians
    pub angular_velocity: f64, // rad/s
    pub torque: f64,         // Nm (applied torque)
    pub limits: (f64, f64),  // (min_angle, max_angle) radians
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContactPoint {
    pub body_part: String,      // "left_foot", "right_hand", etc.
    pub contact_normal: [f64; 3],
    pub contact_force: f64,     // Newtons
    pub other_object: String,   // what was contacted
}
```

### 5.3 ActionFrame

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionFrame {
    /// Timestamp matching the PerceptionFrame this action responds to.
    pub in_response_to: u64,
    /// Character identifier.
    pub character_id: String,
    /// Target joint commands.
    pub joints: Vec<JointCommand>,
    /// Metadata for debugging and logging.
    pub metadata: ActionMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JointCommand {
    pub id: String,
    pub target_angle: f64,    // radians
    pub max_velocity: f64,    // rad/s (how fast to get there)
    pub max_torque: f64,      // Nm (force limit)
    pub blend_weight: f32,    // 0.0-1.0 for animation blending
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionMetadata {
    /// Which motion primitive produced this action.
    pub primitive_id: Option<String>,
    /// ECC tick number that produced this action.
    pub ecc_tick: u64,
    /// Processing latency in microseconds.
    pub processing_us: u64,
    /// Governance approval status.
    pub governance_approved: bool,
}
```

### 5.4 Timing Model

**The mismatch problem.** Game engines typically run physics at 60Hz
(16.67ms per tick). The ECC planning tier runs at 20Hz (50ms per tick).
This means the game sends 3 perception frames for every 1 action frame
from WeftOS.

**Resolution: interpolation + prediction.**

1. The game engine sends PerceptionFrames every physics tick (16.67ms).
2. WeftOS buffers the 3 most recent frames.
3. On each ECC tick (50ms), WeftOS processes the most recent frame and
   produces an ActionFrame.
4. The game engine interpolates between the last two ActionFrames for
   ticks where no new action arrives. Linear interpolation is used for
   joint angles; velocity clamping is applied to prevent jerky motion.

```
Game ticks:    |  1  |  2  |  3  |  4  |  5  |  6  |
               |-----|-----|-----|-----|-----|-----|
Game sends:    P1    P2    P3    P4    P5    P6
               |           |           |
WeftOS recv:   P1          P3          P5
WeftOS sends:       A1          A2          A3
               |           |           |
Game applies:  A0   A0->A1 A1   A1->A2 A2   A2->A3
```

**Latency budget:**

| Phase | Budget | Typical |
|-------|--------|---------|
| Network (TCP loopback) | 1ms | 0.1ms |
| MessagePack deserialize | 2ms | 0.5ms |
| Sensor embedding + HNSW insert | 5ms | 2ms |
| HNSW batch search (10 queries x 10 results) | 10ms | 4ms |
| Causal graph traversal | 5ms | 2ms |
| Motion primitive selection | 5ms | 1ms |
| Governance check | 2ms | 0.5ms |
| MessagePack serialize + send | 2ms | 0.3ms |
| **Total** | **32ms** | **10.4ms** |
| **Margin** (within 50ms tick) | **18ms** | **39.6ms** |

### 5.5 Multiple Characters

Each character gets its own logical connection multiplexed over a single
TCP socket. The `character_id` field in every message identifies which
character the frame applies to. WeftOS maintains one `SensorNode` and
one set of `ActuatorNode` instances per character.

For high character counts (10+), the protocol supports batched frames:
a single message containing `PerceptionFrame` arrays for all characters,
reducing syscall overhead.

---

## 6. Hardware Communication Stack

### 6.1 Physical Topology

```
WeftOS Kernel (Raspberry Pi 5, 8GB)
  |
  +-- I2C Bus 1 (/dev/i2c-1)
  |     +-- PCA9685 @ 0x40 (channels 0-15: right arm servos)
  |     +-- PCA9685 @ 0x41 (channels 0-15: left arm servos)
  |     +-- PCA9685 @ 0x42 (channels 0-15: head/torso servos)
  |     +-- MPU-6050 @ 0x68 (main body IMU)
  |
  +-- Serial UART (/dev/ttyUSB0, 115200 baud)
  |     +-- MKS Gen L (Marlin firmware)
  |           +-- Stepper X: base rotation
  |           +-- Stepper Y: shoulder lift
  |           +-- Stepper Z: elbow
  |           +-- Stepper E: wrist
  |
  +-- SPI Bus 0 (/dev/spidev0.0)
  |     +-- ADS1115 ADC (4-channel, 16-bit)
  |           +-- CH0: current sensor (motor 1)
  |           +-- CH1: current sensor (motor 2)
  |           +-- CH2: servo position feedback
  |           +-- CH3: temperature sensor
  |
  +-- USB
  |     +-- Camera (v4l2, /dev/video0)
  |
  +-- GPIO
  |     +-- Pin 17: endstop X
  |     +-- Pin 27: endstop Y
  |     +-- Pin 22: endstop Z
  |     +-- Pin 5:  emergency stop button (pull-up, interrupt)
  |
  +-- WiFi (wlan0)
        +-- TCP :9000 -- Game engine bridge
        +-- TCP :9001 -- ESP32 sensor mesh gateway
        +-- TCP :8080 -- Kernel RPC (daemon-first, ADR-021)
```

### 6.2 Bus Driver Traits

```rust
/// Hardware bus abstraction.
///
/// Each bus type implements this trait to provide a uniform interface
/// for sensor and actuator drivers.
pub trait HardwareBus: Send + Sync {
    type Address;
    type Error: std::error::Error;

    fn read_register(&self, addr: Self::Address, reg: u8, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write_register(&self, addr: Self::Address, reg: u8, data: &[u8]) -> Result<(), Self::Error>;
    fn bus_name(&self) -> &str;
}

/// I2C bus driver.
pub struct I2cBus {
    dev: Mutex<File>,      // /dev/i2c-N
    bus_number: u8,
}

/// SPI bus driver.
pub struct SpiBus {
    dev: Mutex<File>,      // /dev/spidevN.M
    speed_hz: u32,
    mode: SpiMode,
}

/// Serial (UART) bus driver.
pub struct SerialBus {
    port: Mutex<SerialPort>,
    baud_rate: u32,
}

/// GPIO pin driver.
pub struct GpioPin {
    pin_number: u32,
    direction: GpioDirection,
    edge: Option<GpioEdge>,
}
```

### 6.3 ESP32 Sensor Mesh Gateway

ESP32 sensor nodes (T0 tier per ADR-049) connect to the Pi 5 over WiFi
using the compact binary protocol defined in the tiered kernel profiles
document. The Pi 5 runs a gateway service that:

1. Listens on TCP port 9001 for incoming T0 connections.
2. Decodes compact binary messages (4-byte header + CBOR payload + CRC32).
3. Translates each sensor report into a `SensorEmbedding`.
4. Feeds the embedding into the appropriate `SensorNode` for HNSW
   insertion and causal graph integration.

```rust
pub struct EspGateway {
    listener: TcpListener,
    sensors: HashMap<String, Box<dyn AnySensorNode>>,
    buffer: [u8; 1024],
}

impl EspGateway {
    pub async fn accept_loop(&self) {
        loop {
            let (stream, addr) = self.listener.accept().await.unwrap();
            // Each ESP32 gets its own task
            tokio::spawn(async move {
                handle_esp_connection(stream, addr).await;
            });
        }
    }
}
```

---

## 7. Motion Primitive Library

### 7.1 What Is a Motion Primitive?

A motion primitive is a reusable, parameterized unit of movement. It
consists of:

1. **Trajectory.** A time series of joint commands:
   `Vec<(Duration, Vec<JointCommand>)>`. Each entry specifies when each
   joint should be at what angle.

2. **Expected sensor profile.** A time series of expected sensor readings
   (stored as HNSW embeddings) that characterize "normal" execution.
   Significant deviation from the expected profile triggers re-planning.

3. **Preconditions.** HNSW embeddings representing the expected starting
   state. The planner selects a primitive only if the current sensor
   state is sufficiently similar to the precondition embeddings.

4. **Postconditions.** HNSW embeddings representing the expected ending
   state. Used by the planner to chain primitives: primitive B's
   preconditions should match primitive A's postconditions.

5. **Parameters.** Named values that can be adjusted at execution time:
   speed multiplier, amplitude scaling, target position offset.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MotionPrimitive {
    pub id: String,
    pub name: String,
    pub description: String,

    /// Time-indexed joint trajectory.
    pub trajectory: Vec<TrajectoryPoint>,

    /// Expected sensor embeddings during execution.
    pub expected_profile: Vec<Vec<f32>>,

    /// Embedding representing required starting state.
    pub precondition_embedding: Vec<f32>,

    /// Embedding representing expected ending state.
    pub postcondition_embedding: Vec<f32>,

    /// Named parameters with default values and ranges.
    pub parameters: HashMap<String, PrimitiveParam>,

    /// EffectVector bounds for governance pre-check.
    pub max_effect: EffectVector,

    /// Duration of the primitive at 1x speed.
    pub base_duration: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrajectoryPoint {
    pub time_offset: Duration,
    pub joints: Vec<JointCommand>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrimitiveParam {
    pub default: f64,
    pub min: f64,
    pub max: f64,
}
```

### 7.2 HNSW Storage Strategy

Each motion primitive is stored as multiple HNSW entries:

- **Precondition vector** (key: `primitive:{id}:pre`). Searched during
  planning to find primitives applicable to the current state.

- **Postcondition vector** (key: `primitive:{id}:post`). Searched to
  find primitives whose outcome matches a desired goal state.

- **Profile vectors** (keys: `primitive:{id}:profile:{step}`). Searched
  during execution to detect deviation from expected behavior.

The precondition and postcondition vectors use the same dimensionality
as sensor embeddings, enabling direct cosine similarity comparisons
between "where the robot is now" and "where this primitive starts."

**Memory budget.** A typical primitive has 20-50 trajectory points and
10-20 profile vectors. At 384 dimensions per vector and 4 bytes per
float, each primitive occupies:

```
(2 + 20) vectors * 384 dims * 4 bytes = 33,792 bytes (~33KB)
```

A library of 1,000 primitives requires ~33MB of HNSW storage. This fits
comfortably within the T2 node's 50K vector budget (22,000 vectors for
primitives, leaving 28,000 for sensor history).

### 7.3 Primitive Composition

Complex movements are built from simple primitives using three operators:

**Sequence.** Execute primitives one after another. The system verifies
that each primitive's postcondition matches the next primitive's
precondition.

```rust
pub fn sequence(primitives: &[&MotionPrimitive]) -> CompositePrimitive {
    // Verify precondition/postcondition chain
    for window in primitives.windows(2) {
        let similarity = cosine_similarity(
            &window[0].postcondition_embedding,
            &window[1].precondition_embedding,
        );
        assert!(similarity > 0.8, "Primitive chain broken: similarity {}", similarity);
    }
    CompositePrimitive::Sequence(primitives.to_vec())
}
```

**Blend.** Execute two primitives simultaneously, weighted by a blend
factor. Used for layering movements (e.g., walking + waving).

```rust
pub fn blend(
    base: &MotionPrimitive,
    overlay: &MotionPrimitive,
    weight: f32, // 0.0 = pure base, 1.0 = pure overlay
) -> CompositePrimitive {
    CompositePrimitive::Blend {
        base: base.clone(),
        overlay: overlay.clone(),
        weight,
    }
}
```

**Layer.** Execute primitives on disjoint joint sets. The lower body
follows one primitive (walking gait) while the upper body follows
another (arm gesture).

```rust
pub fn layer(
    lower: &MotionPrimitive,
    lower_joints: &[String],
    upper: &MotionPrimitive,
    upper_joints: &[String],
) -> CompositePrimitive {
    CompositePrimitive::Layer {
        layers: vec![
            (lower.clone(), lower_joints.to_vec()),
            (upper.clone(), upper_joints.to_vec()),
        ],
    }
}
```

### 7.4 Primitive Decomposition

The system learns new primitives by decomposing observed complex movements:

1. **Recording.** During teleoperation or demonstration, all sensor
   readings and actuator commands are captured as a raw trajectory.

2. **Segmentation.** The trajectory is segmented at points where the
   velocity profile crosses zero (pause points) or where the HNSW
   similarity to stored primitives drops below a threshold (novel
   movement detection).

3. **Embedding.** Each segment's start and end states are embedded as
   precondition and postcondition vectors.

4. **Deduplication.** If a new segment's precondition and postcondition
   are both similar (>0.9 cosine) to an existing primitive, the existing
   primitive is reused (possibly with updated parameters).

5. **Storage.** Novel segments are stored as new primitives in the HNSW
   store and registered in the primitive library.

### 7.5 Cross-Domain Transfer

The key insight enabling transfer between robotics and gaming is that
motion primitives are stored as abstract joint-angle trajectories with
semantic labels, not as hardware-specific commands.

A "reaching" primitive might be recorded on a physical robot:

```
precondition: arm at rest (shoulder 0, elbow 0, wrist 0)
trajectory: shoulder 45 -> 90, elbow 0 -> -45, wrist 0 -> 30
postcondition: arm extended (shoulder 90, elbow -45, wrist 30)
```

The same primitive can be applied to a game character by mapping joint
names:
- `robot:shoulder_pitch` -> `character:right_upper_arm.x`
- `robot:elbow_pitch` -> `character:right_forearm.x`
- `robot:wrist_pitch` -> `character:right_hand.x`

A `JointMapping` struct defines this correspondence:

```rust
pub struct JointMapping {
    pub mappings: HashMap<String, JointMapEntry>,
}

pub struct JointMapEntry {
    pub target_joint: String,
    pub scale: f64,        // angle scaling factor
    pub offset: f64,       // angle offset (coordinate system difference)
    pub invert: bool,      // reverse direction
}
```

The transfer is imperfect -- a physical robot has mass, inertia, and
compliance that a game character does not. The causal graph tracks
transfer outcomes: if a transferred primitive consistently fails in the
target domain, the system creates domain-specific variants through the
decomposition pipeline.

---

## 8. Safety and Governance for Physical Actuators

### 8.1 EffectVector Dimensions for Robotics

The EffectVector, already used in the ECC governance layer, is extended
with robotics-specific dimensions:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EffectVector {
    // ── Existing ECC dimensions ──────────────────────────
    pub knowledge_delta: f64,
    pub resource_cost: f64,
    pub reversibility: f64,

    // ── Robotics dimensions ──────────────────────────────
    /// Maximum joint torque commanded (Nm).
    pub joint_torque: f64,
    /// Maximum joint velocity commanded (rad/s).
    pub joint_velocity: f64,
    /// Maximum joint acceleration (rad/s^2).
    pub joint_acceleration: f64,
    /// Estimated collision risk (0.0 = none, 1.0 = certain).
    pub collision_risk: f64,
    /// Total electrical power draw (watts).
    pub power_draw: f64,
    /// Number of actuators affected.
    pub actuator_count: u32,
    /// Whether this command involves external contact (grasping, pushing).
    pub involves_contact: bool,
}
```

### 8.2 Hard Limits (Legislative Branch)

Hard limits are absolute constraints that cannot be overridden except by
firmware modification. They are checked before every actuator command.

```rust
pub struct HardLimits {
    /// Per-joint angle ranges (radians).
    pub joint_ranges: HashMap<String, (f64, f64)>,
    /// Maximum current per motor channel (amps).
    pub max_current: HashMap<String, f64>,
    /// Maximum temperature before thermal shutdown (Celsius).
    pub max_temperature: f64,
    /// Maximum total power draw (watts).
    pub max_total_power: f64,
    /// Minimum time between commands per actuator (microseconds).
    pub min_command_interval: HashMap<String, u64>,
}
```

Hard limits are "legislative" -- they define the constitution of what the
robot is physically allowed to do. They are set during robot assembly and
stored in a read-only configuration file. The kernel loads them at boot
and they cannot be modified at runtime.

### 8.3 Soft Governance (Executive Branch)

Soft governance rules are context-dependent policies that can be adjusted
at runtime. They run after hard limit checks.

```rust
pub struct SoftGovernance {
    /// Maximum velocity during normal operation (may be higher during
    /// emergency recovery).
    pub normal_max_velocity: f64,

    /// Maximum acceleration during normal operation.
    pub normal_max_acceleration: f64,

    /// Minimum distance to obstacles before slowing (meters).
    pub obstacle_proximity_threshold: f64,

    /// Whether the robot is in "learning mode" (reduced force limits).
    pub learning_mode: bool,

    /// Per-primitive force budget (can be learned from experience).
    pub primitive_force_budgets: HashMap<String, f64>,
}
```

Soft governance policies are the "executive" branch -- they interpret the
hard limits in context and may impose stricter constraints based on the
current situation (e.g., "we are near a human, reduce speed").

### 8.4 Emergency Stop

The emergency stop system operates outside the normal PTA loop at the
T-reflex tier:

1. **Hardware E-Stop.** A physical button connected to GPIO with
   interrupt-driven handling. When pressed, the GPIO ISR immediately
   sets all PCA9685 channels to their neutral positions and de-energizes
   steppers via Marlin `M112`. Latency: <1ms.

2. **Software E-Stop.** The `GovernanceGate` maintains an
   `AtomicBool` flag. When set, all `execute()` calls return
   `ActuatorError::EmergencyStopped` without reaching the actuator.
   Any running motion primitive is immediately cancelled.

3. **Impulse Queue E-Stop.** High-priority sensor events (current
   spike, temperature alarm, collision detected) bypass the normal
   PTA queue and directly trigger the software E-Stop. The event is
   recorded in the causal graph with a `TriggeredBy` edge linking
   the sensor reading to the E-Stop action.

**Recovery.** After E-Stop, the system enters a recovery state:
- All actuators are held at their current positions (no drift).
- The planner switches to T-strategy tier for deliberate recovery planning.
- A human operator must acknowledge the E-Stop before normal operation
  resumes (unless the E-Stop was triggered by a transient condition
  that has resolved, in which case auto-recovery is permitted after a
  configurable cooldown period).

### 8.5 The Three-Branch Model

Following the WeftOS governance pattern from the ExoChain:

| Branch | Role | Implementation |
|--------|------|----------------|
| **Legislative** | Define absolute limits | `HardLimits` config, loaded at boot, immutable at runtime |
| **Executive** | Apply contextual policies | `SoftGovernance` rules, adjustable at runtime, checked on every command |
| **Judicial** | Review past actions | Post-execution analysis: compare actual sensor readings against expected primitive profiles, update causal graph with outcome edges |

The judicial branch operates asynchronously after each completed motion
primitive. It compares the actual trajectory (recorded sensor data)
against the expected trajectory (primitive's profile vectors) and records
the outcome:

- **Success.** Postcondition reached within tolerance. A `Causes` edge
  is created from the primitive's command node to the postcondition
  observation, strengthening the primitive.
- **Partial.** Postcondition approximately reached but with deviation.
  The primitive's parameters are adjusted (e.g., speed reduced).
- **Failure.** Postcondition not reached. An `Inhibits` edge is created,
  weakening the primitive. If a primitive accumulates too many failure
  edges, it is marked as unreliable for the current context.

---

## 9. Crate Architecture

### 9.1 New Crates

```
clawft-actuator/
  Cargo.toml
  src/
    lib.rs              # Actuator trait, ActuatorCapabilities, ActuatorError
    servo.rs            # ServoActuator (PCA9685)
    stepper.rs          # StepperActuator (Marlin G-code)
    game_joint.rs       # GameJointActuator (TCP bridge)
    conversation.rs     # ConversationActuator (text output)
    http.rs             # HttpActuator (REST calls)

clawft-sensor/
  Cargo.toml
  src/
    lib.rs              # Sensor trait, SensorEmbedding, SensorError
    imu.rs              # ImuSensor (MPU-6050, ICM-20948)
    camera.rs           # CameraSensor (v4l2 + pose estimation)
    current.rs          # CurrentSensor (ADC-based)
    temperature.rs      # TemperatureSensor (thermistor ADC)
    position.rs         # PositionSensor (encoder, analog feedback)
    game_state.rs       # GameStateSensor (TCP bridge)
    codebase.rs         # CodebaseSensor (file watcher)

clawft-motion/
  Cargo.toml
  src/
    lib.rs              # MotionPrimitive, composition, decomposition
    primitive.rs        # Core primitive struct and trajectory types
    compose.rs          # Sequence, Blend, Layer operators
    decompose.rs        # Recording, segmentation, deduplication
    library.rs          # Primitive library with HNSW-backed storage
    transfer.rs         # Cross-domain joint mapping

clawft-sim-bridge/
  Cargo.toml
  src/
    lib.rs              # Protocol types, codec
    protocol.rs         # PerceptionFrame, ActionFrame, message types
    codec.rs            # MessagePack length-prefixed codec
    connection.rs       # TCP connection management, multiplexing
    timing.rs           # Clock sync, interpolation, frame buffering

clawft-robotics/
  Cargo.toml
  src/
    lib.rs              # High-level robot control API
    control_loop.rs     # ControlLoop, SensorNode, ActuatorNode
    planner.rs          # MotionPlanner trait and implementations
    governance.rs       # GovernanceGate, HardLimits, SoftGovernance
    safety.rs           # Emergency stop, current monitoring
    gait.rs             # Gait planner (walking patterns)
    reach.rs            # Reach planner (IK-based arm movement)
    joint_map.rs        # JointMapping for cross-domain transfer
```

### 9.2 Dependency Graph

```
clawft-types
  |
  +-- clawft-core
  |     |
  |     +-- clawft-kernel (existing: CausalGraph, HnswService, VectorBackend)
  |     |     |
  |     |     +-- clawft-robotics (new: uses kernel services)
  |     |           |
  |     |           +-- clawft-actuator (new: trait + impls)
  |     |           +-- clawft-sensor (new: trait + impls)
  |     |           +-- clawft-motion (new: primitives + HNSW storage)
  |     |           +-- clawft-sim-bridge (new: game engine protocol)
  |     |
  |     +-- clawft-edge-bench (existing: ESP32 benchmark)
  |
  +-- clawft-sensor (T0 no_std variant, different from the std version above)
```

Note: `clawft-sensor` appears twice. The T0 `no_std` variant
(for ESP32) is a separate crate with no dependency on `clawft-kernel`.
The `std` variant (in `clawft-robotics`) depends on `clawft-kernel` for
HNSW and causal graph integration. They share the `Sensor` trait
definition via a common `clawft-sensor-traits` crate that compiles under
both `std` and `no_std`.

### 9.3 Feature Gates

```toml
# clawft-robotics/Cargo.toml
[features]
default = ["servo", "stepper", "game-bridge"]
servo = ["dep:linux-embedded-hal"]
stepper = ["dep:serialport"]
game-bridge = ["clawft-sim-bridge"]
camera = ["dep:opencv"]
imu = ["dep:linux-embedded-hal"]
full = ["servo", "stepper", "game-bridge", "camera", "imu"]
```

On a development machine without I2C hardware, the `game-bridge` feature
alone provides the full game engine integration. On a Pi 5 with servos,
the `servo` and `imu` features add hardware drivers. The feature flags
align with ADR-049's tiered profile system.

---

## 10. Deployment Topology

### 10.1 Single Robot (Direct)

The simplest deployment: one Pi 5 directly controlling servos.

```
[Pi 5] -- I2C --> [PCA9685] --> [Servos]
                  [MPU-6050]
```

- Profile: T2 (Node)
- HNSW capacity: 50K vectors
- Control loop: T-plan at 20Hz, T-servo at 200Hz (PID on Pi CPU)
- Latency: <50ms perceive-to-act

### 10.2 Robot + Sensor Pack

Adds ESP32 sensor nodes for distributed sensing.

```
[ESP32 IMU] --WiFi--> [Pi 5] -- I2C --> [PCA9685] --> [Servos]
[ESP32 Current]         |
[ESP32 Temperature]     +--- Serial --> [MKS Gen L] --> [Steppers]
```

- Pi 5 profile: T2 (Node) with ESP gateway
- ESP32 profile: T0 (Sensor)
- Additional latency: 5-10ms WiFi round trip for sensor data
- Advantage: sensors can be placed at points where wires cannot reach

### 10.3 Simulation + Real (Parallel)

Run the same WeftOS kernel controlling both a game character and a
physical robot simultaneously.

```
[Game Engine] --TCP:9000--> [Pi 5] -- I2C --> [Servos]
                              |
                              +-- Two ControlLoops:
                                  1. GameSensors + GameActuators
                                  2. HwSensors + HwActuators
```

- Both control loops share the same CausalGraph and HNSW store.
- Motion primitives learned in simulation are directly available for
  hardware execution (and vice versa).
- The causal graph tracks which domain each observation came from,
  enabling the judicial branch to evaluate transfer effectiveness.

### 10.4 Fleet (Mesh)

Multiple robots coordinated by a central server.

```
                    [T3 Server (Fleet)]
                    /        |        \
            [Pi 5 A]   [Pi 5 B]   [Pi 5 C]
            Robot 1     Robot 2     Robot 3
```

- T3 server coordinates via WebSocket mesh (existing cluster feature).
- Each Pi 5 runs autonomously with local CausalGraph and HNSW.
- Motion primitives are shared across the fleet via the mesh: a primitive
  learned by Robot 1 is replicated to Robots 2 and 3.
- Fleet coordinator handles task allocation: "Robot 1 pick up object A,
  Robot 2 carry to location B."

### 10.5 Cloud-Assisted

For computationally expensive operations (LLM inference, vision model
inference, large-scale HNSW search).

```
[Pi 5 (local)] --WS--> [Cloud T3 Server]
     |                        |
     +-- Local T-plan         +-- T-strategy (LLM)
     +-- Local T-servo        +-- Vision inference
     +-- Hardware I/O         +-- Large HNSW (1M+ vectors)
```

- The Pi 5 handles real-time control (T-reflex through T-plan).
- T-strategy queries are forwarded to the cloud server.
- The cloud server has access to a much larger motor memory (millions
  of primitives from all robots in the fleet).
- Fallback: if the cloud is unreachable, the Pi 5 operates autonomously
  using its local 50K-vector HNSW store.

---

## 11. Performance Requirements

### 11.1 Control Loop Latency Budgets

| Tier | Period | Hard Deadline | Soft Target | Consequence of Miss |
|------|--------|---------------|-------------|---------------------|
| T-reflex | 0.5ms | 0.5ms | 0.3ms | Physical damage, injury |
| T-servo | 5ms | 5ms | 3ms | Jerky motion, reduced accuracy |
| T-plan | 50ms | 100ms | 30ms | Delayed response, visible lag |
| T-strategy | 5000ms | 30000ms | 1000ms | Slow decision making |

**T-reflex** runs as a kernel interrupt handler (or on the ESP32). Missing
this deadline means the emergency stop did not fire in time. This tier
must be validated through real-time testing on the target hardware.

**T-servo** can tolerate occasional misses (1 in 100) with graceful
degradation (the actuator holds its last commanded position). Sustained
misses (>10 consecutive) trigger a fallback to reduced speed operation.

**T-plan** is the most critical tier for user-perceived quality. The
50ms budget matches the DEMOCRITUS cognitive tick. The 100ms hard
deadline allows for occasional HNSW search spikes. Beyond 100ms, the
game engine's interpolation buffer runs empty and the character visibly
freezes.

### 11.2 Memory Budget for Motor Memory

| Component | Vectors | Dimensions | Bytes | Notes |
|-----------|---------|------------|-------|-------|
| Sensor history (60s) | 1,200 | 384 | 1.8 MB | 20Hz * 60s |
| Motion primitives (100) | 2,200 | 384 | 3.4 MB | 22 vectors/primitive |
| Motion primitives (1000) | 22,000 | 384 | 33.8 MB | Full library |
| Anomaly baseline | 5,000 | 384 | 7.7 MB | Normal operation profiles |
| Working memory | 1,000 | 384 | 1.5 MB | Recent causal nodes |
| **T2 Total (1000 primitives)** | **31,400** | -- | **48.2 MB** | Within 50K limit |

HNSW index overhead adds approximately 2x the raw vector storage for
the graph structure, bringing the total HNSW memory footprint to ~96MB
for a T2 node with 1,000 motion primitives. This is within the 1-4GB
memory budget defined in ADR-049.

### 11.3 Network Bandwidth for Game Engine Bridge

| Message Type | Size | Rate | Bandwidth |
|-------------|------|------|-----------|
| PerceptionFrame (20 joints, no camera) | ~800 bytes | 60 Hz | 48 KB/s |
| PerceptionFrame (20 joints + 64KB camera) | ~65 KB | 10 Hz | 650 KB/s |
| ActionFrame (20 joints) | ~500 bytes | 20 Hz | 10 KB/s |
| Heartbeat | 16 bytes | 1 Hz | 16 B/s |
| **Total (no camera)** | -- | -- | **~58 KB/s** |
| **Total (with camera)** | -- | -- | **~710 KB/s** |

These bandwidth numbers are well within WiFi capabilities (even 2.4GHz
WiFi provides 10+ MB/s real throughput). For TCP loopback (game engine
on the same machine or a nearby dev machine), bandwidth is effectively
unlimited.

### 11.4 Storage for ExoChain Movement History

Every actuator command and significant sensor event is recorded on the
ExoChain for provenance and audit. Storage estimates:

| Event Type | Size per Entry | Rate | Daily Storage |
|-----------|---------------|------|---------------|
| Actuator command | ~200 bytes | 20 Hz | 345 MB |
| Governance decision | ~100 bytes | 20 Hz | 172 MB |
| Sensor anomaly | ~300 bytes | ~1 Hz | 26 MB |
| Primitive completion | ~500 bytes | ~0.1 Hz | 4 MB |
| E-Stop event | ~1 KB | rare | negligible |
| **Total** | -- | -- | **~547 MB/day** |

With ExoChain checkpointing (ADR: `checkpoint_interval: 1000`), the
in-memory chain holds at most 1,000 recent events (~200KB). Older events
are persisted to disk. A 128GB SD card on the Pi 5 holds approximately
234 days of continuous operation history.

**Compaction.** The ExoChain does not compact (it is append-only for
auditability). However, the HNSW store's `soft_delete` and `compact`
methods (from `VectorBackend`) allow periodic cleanup of stale sensor
embeddings. A recommended policy: keep 1 hour of full sensor history,
then downsample to 1 reading per second for the previous 24 hours, and
1 reading per minute for older data.

---

## Appendix A: Relationship to Existing ADRs

| ADR | Relationship |
|-----|-------------|
| ADR-021 (daemon-first RPC) | Game engine bridge and ESP gateway use the same TCP RPC infrastructure |
| ADR-047 (self-calibrating tick) | T-plan tick rate is auto-adjusted based on measured HNSW search latency |
| ADR-049 (tiered kernel profiles) | Sensor and actuator crates align with T0-T4 deployment tiers |

## Appendix B: Open Questions

1. **Embedding dimensionality.** Should all sensors use the same
   dimensionality (384, matching the kernel default) via padding/projection,
   or should the HNSW store support mixed dimensionalities? Mixed dims
   would require separate HNSW instances per sensor type.

2. **Real-time guarantees.** The T-reflex tier needs hard real-time
   guarantees that Linux cannot provide. Options: (a) run T-reflex on
   the ESP32 and use the Pi for T-servo and above, (b) use a real-time
   Linux kernel (PREEMPT_RT), (c) use a dedicated PRU (if available on
   the platform).

3. **Primitive format standardization.** Should motion primitives use an
   existing format (e.g., URDF for robot description, MoveIt motion
   plans) or a WeftOS-native format? A native format is simpler but
   reduces interoperability.

4. **Camera embedding model.** The pose estimation approach works for
   humanoid characters but not for abstract game entities. A more general
   approach might use a pre-trained vision encoder (e.g., CLIP) to
   produce generic scene embeddings.

5. **Multi-robot coordination protocol.** The fleet topology assumes a
   central coordinator. A fully decentralized approach using the existing
   mesh consensus might be more resilient but adds complexity.

---

## Appendix C: Glossary

| Term | Definition |
|------|-----------|
| DEMOCRITUS | The ECC's cognitive tick cycle: observe, reason, decide, act |
| ECC | Embedded Cognitive Core -- the kernel's causal reasoning subsystem |
| ExoChain | Append-only event log with dual-signature governance |
| HNSW | Hierarchical Navigable Small World -- approximate nearest neighbor search |
| PTA | Perceive-Think-Act control loop |
| T-reflex | 0.5ms control tier for emergency responses |
| T-servo | 5ms control tier for PID joint control |
| T-plan | 50ms control tier for motion planning with ECC |
| T-strategy | 500ms+ control tier for high-level deliberation |
| Motion primitive | A reusable, parameterized unit of movement |
| EffectVector | Multi-dimensional impact descriptor for governance checking |
| Governance gate | Safety checkpoint between THINK and ACT phases |
