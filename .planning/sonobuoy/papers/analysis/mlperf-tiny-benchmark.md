# MLPerf Tiny Benchmark (Banbury et al., 2021)

## Citation

- **Title**: MLPerf Tiny Benchmark
- **Authors**: Colby Banbury, Vijay Janapa Reddi, Peter Torelli, Jeremy Holleman, Nat Jeffries, Csaba Kiraly, Pietro Montino, David Kanter, Sebastian Ahmed, Danilo Pau, Urmish Thakker, Antonio Torrini, Peter Warden, Jay Cordaro, Giuseppe Di Guglielmo, Javier Duarte, Stephen Gibellini, Videet Parekh, Honson Tran, Nhan Tran, Niu Wenxu, Xu Xuesong (22 authors)
- **Affiliation**: Harvard (Banbury, Reddi, Warden), EEMBC (Torelli, Kanter), UNC Charlotte + Syntiant (Holleman), STMicroelectronics (Pau, Torrini), SambaNova (Thakker), Arm (Gibellini), UCSD (Duarte), Fermilab / Columbia (Di Guglielmo, N. Tran), and others.
- **Venue**: NeurIPS 2021 Datasets and Benchmarks Track; also a MLCommons working-group white paper.
- **arXiv**: 2106.07597 (14 Jun 2021; v4 24 Aug 2021)
- **DOI**: 10.48550/arXiv.2106.07597
- **URL**: https://arxiv.org/abs/2106.07597
- **Code**: https://github.com/mlcommons/tiny

## Status

**verified**. arXiv:2106.07597 resolves to this exact title and author list; v4 of the PDF was retrieved and parsed. Published at NeurIPS 2021 Datasets and Benchmarks Track (par.nsf.gov record 10300102). Independent confirmation via MLCommons blog post and Semantic Scholar.

## One-paragraph Summary

MLPerf Tiny defines the first industry-standard benchmark for machine-learning inference on microcontroller-class hardware (≤ 1 MB flash, ≤ 512 KB SRAM, ≤ 100 mW). It is the work of 50+ organizations inside MLCommons and provides four tasks — keyword spotting, visual wake words, image classification, anomaly detection — plus a reference implementation running on an STMicro NUCLEO-L4R5ZI (Cortex-M4F @ 120 MHz, 640 KB SRAM, 2 MB flash). The keyword-spotting benchmark uses a 52.5 KB INT8 DS-CNN on 49×10 MFCC features over Google Speech Commands v2, with a 90% top-1 quality target (the reference model hits 91.6% on the full test set). The benchmark methodology is the contribution: an EEMBC EnergyRunner harness measures wall-clock inference latency and energy-per-inference with 1 s warm-up, 10 s minimum runtime, and separate accuracy, performance, and energy submission modes. Closed-division submissions must use the reference models; open-division allows any model that hits the accuracy target. The v0.5 round (Jun 2021) reported the reference DS-CNN at ~1000 inferences/s and ~100 µJ/inference on the NUCLEO board. The paper is the single best published reference for realistic always-on audio-inference budgets on a sonobuoy-scale MCU.

## Methodology

### The four benchmarks (Table from Section 4, paper p. 5)
| Benchmark | Dataset | Input | Reference model | Quality target |
|-----------|---------|-------|-----------------|----------------|
| Keyword Spotting | Speech Commands v2 | 49×10 MFCC | **DS-CNN, 52.5 KB** | 90% top-1 |
| Visual Wake Words | VWW | 96×96 RGB | MobileNetV1 (325 KB) | 80% top-1 |
| Image Classification | CIFAR-10 | 32×32 | ResNet (96 KB) | 85% top-1 |
| Anomaly Detection | ToyADMOS | 5×128 mel | FC-AutoEncoder (270 KB) | 0.85 AUC |

### Reference hardware (Section 5)
- **Board**: STMicroelectronics NUCLEO-L4R5ZI.
- **MCU**: STM32L4R5ZIT6U, Arm Cortex-M4F, 120 MHz peak (80 MHz reported in some deployments), single-precision FPU.
- **Memory**: 640 KB SRAM, 2 MB flash.
- **Runtime**: TensorFlow Lite for Microcontrollers (TFLM), known-good snapshot pinned for stability; bare-metal MBED project; GCC-ARM toolchain.
- **Energy harness**: EEMBC EnergyRunner over a UART trigger with IoTConnect current probe; reports latency (ms), throughput (inf/s), and energy (µJ/inference).

### KWS task pipeline (Section 4.3)
1. **Audio**: 1-second clip at 16 kHz mono.
2. **Features**: 40-bin mel filterbank → DCT → 10-coefficient MFCC; window 40 ms, stride 20 ms → 49×10 per clip. Pre-selected features (fixed) so all submissions compare on the same input.
3. **Model**: DS-CNN — conv block (input) + 4 depthwise-separable conv blocks (64 filters) + avg-pool + FC classifier. INT8 post-training-quantized. 52.5 KB.
4. **Output**: 12 classes (standard Warden-subset: 10 commands + silence + unknown).
5. **Quality target**: 90% top-1 on 1000-utterance randomly-sampled subset (reference model hits 91.6% full set, 91.7% subset).

### Submission modes
- **Closed division**: must use the reference DS-CNN model weights; variations allowed only in compiler, runtime, quantization post-hoc.
- **Open division**: any model is allowed as long as it hits the 90% quality target on the 1000-utterance evaluation subset.
- **Measurement**: 1 s warm-up, ≥ 10 s minimum runtime; accuracy, latency, and energy reported in three separate runs.

### Quantization
INT8 uniform symmetric per-tensor quantization via TFLite's post-training quantization flow; calibration set is 100 randomly-selected training samples (indices pinned in `quant_cal_idxs.txt`). Activations INT8; biases INT32. No quantization-aware training in the reference; open-division submissions can use QAT.

## Key Results

### Reference DS-CNN on NUCLEO-L4R5ZI (Section 6)
- **Model size**: 52.5 KB flash (INT8).
- **Peak activation memory**: ~20 KB SRAM (est. from patch-based execution of DS-CNN blocks).
- **Accuracy**: 91.6% full test set, 91.7% on 1000-utterance subset. ≥ 90% target met by 1.6 pp.
- **Throughput**: ~1000 inferences/second at 120 MHz (≈ 1 ms/inference).
- **Energy**: ~100 µJ/inference on the reference board.
- **Footprint headroom**: 52.5 KB flash leaves 1.95 MB free; 20 KB SRAM leaves 620 KB free.

### Round 0.5 submissions (Jun 2021) highlights
- STMicroelectronics's own submission reached ~0.58 ms/inference (1725 inf/s) with their X-CUBE-AI runtime — ~1.7× speedup over the TFLM reference.
- Plumerai and Syntiant specialist-inference cores (reported in later rounds) reach < 100 µs/inference with < 10 µJ.

### KWS energy-budget implications
- At 1 inference/second duty cycle: 100 µJ/inf × 1 Hz = 100 µW of compute, plus microphone + MFCC front end (~few hundred µW) ≈ sub-milliwatt budget.
- At 50 inferences/second (streaming at 20 ms/frame): 100 µJ × 50 = 5 mW compute.
- Both are well inside a 1 W sonobuoy budget even with MCU wake/sleep overhead.

## Strengths

- **Actual MCU hardware**: unlike Rybakov (phone) or MCUNet (mixed), the reference measurements are on a Cortex-M4 at 120 MHz — directly comparable to the sonobuoy target MCU.
- **Energy is a first-class metric**: EEMBC EnergyRunner integration means energy-per-inference is reported in µJ, not inferred. This is the number that sizes the sonobuoy battery.
- **Public, reproducible, versioned**: MLPerf Tiny has run v0.5, v1.0, v1.1, v1.2, v1.3 at the time of writing. Each adds submissions; the reference models are frozen so the benchmark is truly longitudinal.
- **Four diverse tasks** mean you can pick and choose: the KWS task is the immediate analog for a sonobuoy trigger, but the anomaly-detection task (ToyADMOS on 5×128 mel) is a strong template for a marine-ambient-noise anomaly detector.
- **Closed + open divisions**: closed gives an apples-to-apples hardware comparison; open rewards algorithmic innovation. Both are useful benchmarks for a sonobuoy team.

## Limitations

- **Reference models are small and generic**: DS-CNN at 52.5 KB is pedagogically clean but not the Pareto frontier. Recent submissions (Plumerai, Syntiant) are 10× faster and lower energy. The right reading is "this is the floor, not the ceiling."
- **Input features are pre-selected**: MFCC 49×10 is fixed. A sonobuoy likely wants log-mel at a higher sample rate (32-96 kHz for marine-mammal calls), not 16 kHz MFCC. Submissions that change the front end are open-division only.
- **No continuous-streaming mode**: MLPerf Tiny measures single-clip inference, not the ring-buffer / stream pattern Rybakov describes. The real duty-cycle numbers for an always-on trigger need to be derived from the per-inference numbers plus streaming overhead.
- **No joint MCU + sensor power measurement**: EEMBC EnergyRunner captures the MCU's energy, not the microphone/ADC. For a hydrophone with 100-300 µA ADC draw, this is a material missing piece.
- **INT8 is the only quantization benchmarked in closed**: 4-bit / mixed-precision numbers are scattered across open-division submissions, hard to use as a planning input.

## Portable Details

### DS-CNN reference architecture (from `mlcommons/tiny/benchmark/training/keyword_spotting/`)
- Input: 49×10×1 MFCC (float32 for training, INT8 post-quantization for inference).
- Conv block: 10×4 conv, 64 filters, stride (2,2), batch-norm, ReLU.
- 4× depthwise-separable conv blocks: 3×3 depthwise + 1×1 pointwise, 64 filters, BN, ReLU.
- Global average pooling.
- FC: 12-class softmax.
- Total params: ~38K. INT8 model size: 52.5 KB (including metadata).
- Train: 36 epochs, Adam, cosine LR (from Keras reference). Google Speech Commands v2.

### Quantization procedure
```python
converter = tf.lite.TFLiteConverter.from_keras_model(model)
converter.optimizations = [tf.lite.Optimize.DEFAULT]
converter.representative_dataset = calibration_data_gen   # 100 samples from quant_cal_idxs.txt
converter.target_spec.supported_ops = [tf.lite.OpsSet.TFLITE_BUILTINS_INT8]
converter.inference_input_type = tf.int8
converter.inference_output_type = tf.int8
quantized = converter.convert()
```

### Energy-measurement invariants
- **1 s warm-up** ensures caches, clock gating, and DVFS are in steady state.
- **≥ 10 s runtime** averages over thermal and supply-noise variation.
- **Three separate runs**: accuracy-only, performance-only, energy-only. Accuracy run is on the 1000-utterance subset and must match the 90% target; performance run is latency; energy run uses EnergyRunner.

### NUCLEO-L4R5ZI relevant specs
- Cortex-M4F @ 120 MHz (80 MHz for EnergyRunner measurements).
- 640 KB SRAM (512 KB main SRAM + 128 KB SRAM2).
- 2 MB flash.
- Active current @ 80 MHz: ~7 mA (56 mW @ 8 V input, or ~23 mW @ 3.3 V).
- Deep-sleep current: < 10 µA.
- This is the closest published analog to the sonobuoy MCU tier.

## Sonobuoy Integration Plan

### MLPerf-Tiny as a sizing model
Treat MLPerf Tiny as the validated floor for on-buoy inference. The 52.5 KB DS-CNN KWS model at ~100 µJ/inference and ~1 ms latency is **well inside** a 1 W battery budget:

- **Always-on continuous triggering** (every 20 ms frame): 50 inf/s × 100 µJ = 5 mW compute. At 1 W total budget, this is 0.5% of the budget. The rest goes to microphone, RF, and sleep overhead.
- **Event-triggered confirmation** (1 inf/s): 100 µW compute. Negligible.

### On-buoy vs at-shore split (informed by MLPerf)
| Tier | Location | Budget | Model | Source |
|------|----------|--------|-------|--------|
| 1 | On-buoy | ~100 µW | RMS level gate | hardware |
| 2 | On-buoy | ~5 mW | DS-CNN KWS-style trigger, 4-8 classes | MLPerf Tiny reference (this paper) |
| 3 | On-buoy | ~50 mW | Larger DS-CNN or MCUNet, 16-32 classes | MLPerf open-division + MCUNet |
| 4 | At shore | ~200 W | Full DEMONet / K-STEMIT / Perch | Round 1 |

### Which MLPerf task to reuse on-buoy
- **KWS (DS-CNN)** → adapt to a 4-class sonobuoy trigger (vessel / cetacean / transient / noise). Retrain with target sample rate (32-48 kHz, not 16 kHz), keep the architecture, regenerate INT8 with the same calibration flow.
- **Anomaly Detection (FC-AutoEncoder)** → adapt to marine-ambient-noise anomaly detection. 5×128 log-mel on 1-second windows; reconstruction-loss threshold triggers a confirmation-tier inference.
- **Visual Wake Words** → not applicable unless the sonobuoy adds a camera (generally not).
- **Image Classification (CIFAR-10 ResNet 96 KB)** → template for per-spectrogram-image species classification in the confirmation stage.

### Budget fit
- Flash: 52.5 KB (trigger) + 270 KB (anomaly) + 96 KB (confirmation classifier) = **419 KB**. Fits comfortably in 10 MB budget.
- SRAM: 20 KB peak (trigger) + maybe 80 KB (anomaly + confirmation) = **100 KB peak**, right at the budget. Patch-based inference (MCUNet) brings this down.
- Energy: 5 mW (trigger streaming) + 50 mW (occasional confirmation) ≪ 1 W. Plenty of headroom for microphone + RF.

## Follow-up References

1. **Zhang et al. 2018**, "Hello Edge: Keyword Spotting on Microcontrollers," arXiv:1711.07128 — origin of the DS-CNN that MLPerf Tiny uses as its KWS reference.
2. **Warden 2018**, "Speech Commands: A Dataset for Limited-Vocabulary Speech Recognition," arXiv:1804.03209 — dataset for the KWS benchmark.
3. **Koizumi et al. 2019**, "ToyADMOS: A dataset of miniature-machine operating sounds for anomalous sound detection," WASPAA 2019 — dataset for the anomaly-detection benchmark, directly relevant to marine-ambient anomaly.
4. **EEMBC IoTConnect / EnergyRunner** (https://www.eembc.org/energyrunner/) — the energy-measurement harness that MLPerf Tiny standardizes on.
5. **Prakash et al. 2023**, "MLPerf Tiny v1.0 results," MLCommons whitepaper — longitudinal results across submission rounds, tracking algorithmic and hardware improvements.
