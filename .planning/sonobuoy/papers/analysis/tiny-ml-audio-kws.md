# Streaming Keyword Spotting on Mobile Devices (Rybakov et al., 2020)

## Citation

- **Title**: Streaming keyword spotting on mobile devices
- **Authors**: Oleg Rybakov, Natasha Kononenko, Niranjan Subrahmanya, Mirkó Visontai, Stella Laurenzo
- **Affiliation**: Google Research (1), Google Speech (2)
- **Venue**: INTERSPEECH 2020 (preprint)
- **arXiv**: 2005.06720 (14 May 2020; v2 29 Jul 2020)
- **URL**: https://arxiv.org/abs/2005.06720
- **Code**: https://github.com/google-research/google-research/tree/master/kws_streaming

## Status

**verified**. arXiv:2005.06720 resolves to exactly this title, author list, and year. The PDF header confirms INTERSPEECH submission. Semantic Scholar entry (paper ID 70dc02df0c5a465e4f051f82f04a4448feab2788) independently matches. The paper was retrieved and parsed directly.

## One-paragraph Summary

Rybakov et al. deliver two contributions in one short paper. First, a Keras-layer wrapper ("Stream wrapper") that takes an ordinary non-streaming KWS model — DNN, CNN, DS-CNN, GRU, LSTM, CRNN, SVDF, TC-ResNet, MHAtt-RNN — and mechanically rewrites it into a streaming TFLite model that receives 20 ms audio frames and returns a class probability per tick, without any hand-rewriting by the developer. The conversion folds the MFCC front end inside the model graph (so deployment is one TFLite file), inserts ring-buffer "states" for convolutions and RNN layers, and handles both internal-state (model holds buffers) and external-state (caller feeds states back) variants. Second, the paper contributes a novel MHAtt-RNN architecture (4-head attention + GRU) that lowers classification error by ~10% relative over the then-SOTA TC-ResNet on Google Speech Commands v2, reaching 98.0% top-1 with only ~743K parameters. Benchmarked on a Pixel 4 phone with TFLite, streaming latency per 20 ms frame is sub-millisecond for most architectures (DNN 0.6 ms, SVDF 2.2 ms, DS-CNN ~3 ms), and quantized MFCC extraction alone costs ~1.8 ms of that. This paper is the canonical reference for the stream-conversion pattern any always-on, battery-limited acoustic detector should implement — exactly what a sonobuoy trigger stage needs.

## Methodology

### Task and front end
- **Dataset**: Google Speech Commands v1 and v2, 35-word vocabulary; trained on 12-label subset (10 command words + "silence" + "unknown").
- **Front end**: MFCC, 40 ms frames with 20 ms overlap (stride), so a 1-second clip becomes a 49-frame sequence. FFT or DFT selectable; FFT+INT8 quantization halves frontend latency.
- **Augmentation**: time-shift ±100 ms; signal resampling factor; SpecAugment for the TC-ResNet variant.
- **Optimizer**: Adam, cross-entropy loss. The paper is thin on training hyperparameters; those live in the repo.

### Stream wrapper mechanics
Converting a non-streaming model to streaming is reduced to four tiny steps:
1. Set input layer feature size = 1 frame (instead of the full 49).
2. Traverse the Keras graph; for every layer that depends on temporal context (Conv, DepthwiseConv, RNN), wrap it with the `Stream` class.
3. At inference time the `Stream` wrapper maintains a per-layer ring buffer of shape `(effective_filter_time, feature_dim)`. Each tick it appends the new frame, drops the oldest, and hands the buffer to the underlying cell — so the convolution computes on the new sample only.
4. Export to TFLite; optional INT8 quantization.

Critical design choices:
- **Stream wrapper is a training no-op** — the training graph equals the non-streaming graph, so training time is unchanged.
- **Internal vs external state**: internal state means TFLite holds the buffers; external state exposes them as additional inputs/outputs so the caller (C++ or Rust) owns them. External state is the deployment pattern you want on a microcontroller because it avoids TFLite allocator churn.
- **Limitations**: the wrapper does not support striding or pooling > 1 in the time dimension. Models using those (CNN+strd, DS-CNN+strd) cannot be auto-streamed; only the non-strided variants can.

### Architectures evaluated (Section 3, Table 1)
| Model | V1 acc. [%] | V2 acc. [%] | Streamable |
|-------|------------|------------|------------|
| DNN (pooling + FC stack) | 91.2 | 90.6 | yes |
| CNN (no stride) | — | — | yes |
| CNN+strd | 95.4 | 95.6 | no |
| DSCNN+strd | 97.0 | 97.1 | no |
| GRU | 96.6 | 97.2 | yes |
| LSTM | 96.9 | 97.5 | yes |
| CRNN | 97.0 | 97.5 | yes |
| SVDF | 96.3 | 96.9 | yes |
| TC-ResNet14 (365K) | — | 97.4 | no |
| **MHAtt-RNN** (GRU + 4-head attention) | **97.2** | **98.0** | yes |

### MHAtt-RNN novelty
Replaces the LSTM in the classic attention-RNN with a GRU and a 4-head attention layer. The attention heads pool over the 49-frame MFCC sequence with learned queries, feeding a fully-connected classifier. The 10%-relative error reduction comes almost entirely from the multi-head variant over the single-head prior work.

## Key Results

### Streaming latency on Pixel 4 (Table 2, partial reproduction)
| Model | Non-stream latency (ms) | Stream latency (ms) | Params (K) |
|-------|-------------------------|---------------------|------------|
| DNN | 4 | 0.6 | 447 |
| CNN | — | ~1.0 | — |
| SVDF | — | ~2.2 | — |
| DSCNN | — | ~3.0 | — |
| GRU | — | ~1.5 | — |
| CRNN | — | ~2.5 | — |
| MHAtt-RNN | — | ~3–4 | ~743 |

- **Streaming frame budget**: 20 ms of audio in ≤ 3 ms of compute leaves ≥ 85% headroom; on a 120 MHz Cortex-M4 this drops by roughly 20× but streaming SVDF or CRNN still fits.
- **MFCC extraction alone**: ~3.7 ms without quantization, ~1.8 ms with FFT + INT8.
- **Accuracy**: 98.0% on Speech Commands v2 (MHAtt-RNN, non-streaming). Statefully-trained RNN streaming variants lose up to 2 pp accuracy if states are not reset between utterances.

## Strengths

- **Zero-effort streaming conversion**: a single Keras import turns any non-streaming model into a streamable TFLite graph. This is the right abstraction for an acoustic-trigger pipeline because the research/training code and the deployment code are literally the same model object.
- **MFCC inside the model graph**: feature extraction ships with the TFLite file, so C++/Rust deployment only needs TFLite Micro + a ring buffer, not a separate DSP library.
- **Architecture-agnostic**: works for DNN, CNN, RNN, attention, hybrid. Lets the sonobuoy designer swap triggers without rewriting the runtime.
- **Open-source** (Apache 2.0): production-quality reference code at the Google Research repo.
- **Empirical separation of training vs inference graph**: the paper is explicit that streaming is a graph-rewrite, not a retraining. Invaluable for anyone who already has a pretrained model.

## Limitations

- **Phone-grade numbers, not microcontroller**: Pixel 4 has ~8 GB RAM and a 2.7 GHz CPU. The latency numbers do not transfer directly to an MCU; expect 10-30× slowdown and 100-1000× less memory. This paper must be combined with MCUNet-style (MCU-specific NAS) or MLPerf-Tiny (actual MCU measurements) to get a deployable sonobuoy trigger.
- **No energy measurements**: latency is reported but not mJ/inference. For a 1 W battery budget this is the number that actually matters; you must add your own energy harness.
- **Striding/pooling > 1 in time is not supported** by the Stream wrapper. The highest-accuracy DSCNN+strd and CNN+strd variants are effectively off the table for streaming conversion.
- **Quantization is mentioned but not characterized**: INT8 TFLite conversion is listed but no accuracy-drop numbers are given.
- **Always-on microphone assumed**: there is no discussion of duty-cycling the feature extractor. A real low-power deployment gates the whole pipeline by a cheap VAD / level detector upstream, which this paper does not address.

## Portable Details

### Stream-wrapper invariants
For each wrapped layer with effective temporal receptive field `R`:
```
state_buffer: float32[R-1, feature_dim]      # past R-1 samples
on tick(new_frame):
    buffer = concat([state_buffer, new_frame])   # shape [R, feature_dim]
    output = layer.call(buffer)                  # same math as training
    state_buffer = buffer[1:, :]                 # drop oldest
```
This is ~30 lines of Rust per layer type. Port of the `Stream` wrapper to `candle-core` or `tract-onnx` is straightforward and is the right first step for a Rust sonobuoy trigger.

### MFCC configuration
- Frame length: 40 ms → 640 samples at 16 kHz.
- Stride: 20 ms → 320 samples.
- Output sequence length for 1 s clip: 49 frames.
- Mel bins: 40 (paper) but see discussion of 10-bin variant in Section 4.
- FFT: 512-point; INT8 post-training quantization halves runtime.

### MHAtt-RNN hyperparameters (from code)
- GRU: 128 hidden units, 2 layers.
- Attention: 4 heads, `d_model = 64`, `d_k = d_v = 16`, query is a learned vector broadcast over time.
- Total: ~743K parameters, ~365 KB INT8.

### Speech-Commands v2 default split
Train: 85,511 utterances. Val: 10,102. Test: 4,890. 35 words. The 12-label subset ("yes","no","up","down","left","right","on","off","stop","go", plus "silence", "unknown") is the standard KWS benchmark.

## Sonobuoy Integration Plan

### On-buoy (1 W, 10 MB flash, 100 KB SRAM)
A stream-converted **SVDF** or **GRU** model (≤ 400 KB INT8, ~1.5-2 ms/frame on Pixel 4 → ~40 ms on Cortex-M4 @ 120 MHz, which is still < 50% of a 100 ms frame budget) is a realistic trigger. The Rybakov pattern maps onto the sonobuoy pipeline as follows:

1. **Always-on stage (µW-scale)**: analog hydrophone → band-pass filter → RMS-level gate running in hardware / Cortex-M0+ sleep loop. ~10-50 µW. No ML.
2. **Trigger stage (on-buoy, 10-50 mW)**: when the level gate fires, wake a Cortex-M4 or Cortex-M33 running a Rybakov-style streaming SVDF or GRU with custom classes for sonobuoy targets ("vessel-present", "cetacean-call", "noise", "silence"). ~3 ms/frame budget × 50 fps = 150 ms of compute per second = 15% duty cycle inside this stage. **This is exactly the Rybakov design point.**
3. **Confirmation stage (on-buoy, 100-300 mW)**: on positive trigger, buffer 5-10 seconds of raw audio and run a bigger model (e.g., MCUNet-sized DS-CNN) for a cleaner decision. ~100 ms/inference, 1 inference per confirmed detection.
4. **Shore-side heavy inference (GPU)**: send the 5-10 s clip via RF burst; at-shore runs the full round-1 pipeline (DEMONet → K-STEMIT fusion → species ID via Perch/BEATs).

### Split recommendation
- **On-buoy (Rybakov-style)**: stream wrapper; MFCC inside the graph; SVDF/GRU trigger; INT8 quantization; ring-buffered state.
- **At shore (round-1 stack)**: DEMONet MoE, K-STEMIT spatial branch, Helmholtz-PINN physics prior, Perch/SurfPerch species ID, HNSW retrieval.
- **Why the split works**: the trigger model has only 4-8 output classes ("is this interesting?"); the full pipeline classifies 6,400+ bird-equivalent species. One fits in 365 KB; the other is 80 MB+ of ONNX graph that will never fit on a sonobuoy.

### Fit-check against the budget
| Constraint | Rybakov MHAtt-RNN | Rybakov SVDF | Rybakov DNN |
|-----------|-------------------|--------------|-------------|
| Flash (INT8) | ~365 KB | ~200 KB | ~450 KB |
| Peak SRAM (streaming) | ~80 KB | ~30 KB | ~40 KB |
| Ops/frame (Cortex-M4 @ 120 MHz estimate) | ~36 MOPS, ~60 ms | ~20 MOPS, ~33 ms | ~4 MOPS, ~7 ms |
| Fits 100 KB SRAM budget? | yes | yes | yes |
| Fits 10 MB flash budget? | yes | yes | yes |
| Fits 1 W continuous (assuming 30% duty)? | marginal | comfortable | comfortable |

**SVDF or small GRU is the recommended sonobuoy trigger.** MHAtt-RNN is good for the confirmation stage.

## Follow-up References

1. **Zhang et al. 2018**, "Hello Edge: Keyword Spotting on Microcontrollers," arXiv:1711.07128 — the canonical DS-CNN KWS reference that Rybakov benchmarks against; includes INT8 numbers for Cortex-M7.
2. **Warden 2018**, "Speech Commands: A Dataset for Limited-Vocabulary Speech Recognition," arXiv:1804.03209 — the dataset used throughout.
3. **Banbury et al. 2021**, "MLPerf Tiny Benchmark," arXiv:2106.07597 — picks up where Rybakov leaves off and actually measures KWS on a Cortex-M4 at both latency and energy.
4. **Lin et al. 2020**, "MCUNet: Tiny Deep Learning on IoT Devices," arXiv:2007.10319 — hardware-aware NAS that reaches 91% on GSC under 256 KB SRAM.
5. **Choi et al. 2019**, "Temporal Convolution for Real-Time Keyword Spotting on Mobile Devices," arXiv:1904.03814 — the TC-ResNet paper MHAtt-RNN beats on v2.
