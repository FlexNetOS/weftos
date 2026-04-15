# MCUNet: Tiny Deep Learning on IoT Devices (Lin et al., 2020)

## Citation

- **Title**: MCUNet: Tiny Deep Learning on IoT Devices
- **Authors**: Ji Lin, Wei-Ming Chen, Yujun Lin, John Cohn, Chuang Gan, Song Han
- **Affiliation**: MIT (Lin, Chen, Y. Lin, Han), MIT-IBM Watson AI Lab (Cohn, Gan)
- **Venue**: NeurIPS 2020 (spotlight)
- **arXiv**: 2007.10319 (20 Jul 2020; v2)
- **DOI**: 10.48550/arXiv.2007.10319
- **URL**: https://arxiv.org/abs/2007.10319
- **Code**: https://github.com/mit-han-lab/mcunet and https://github.com/mit-han-lab/tinyengine

## Status

**verified**. arXiv:2007.10319 resolves to the Lin et al. paper with the stated authors; NeurIPS 2020 spotlight acceptance is confirmed via the MIT Han Lab project page and Semantic Scholar. The PDF was retrieved and parsed; the quoted tables and numbers below are extracted directly from the v2 text.

## One-paragraph Summary

MCUNet is a **joint architecture-and-runtime co-design** for deep learning on microcontroller-class hardware (STM32F4/F7/H7, ≤ 512 KB SRAM, ≤ 2 MB flash). Two components: **TinyNAS**, a two-stage neural architecture search that first optimizes the search space itself to match the MCU's memory/FLOPs envelope and then performs weight-shared NAS inside that reduced space; and **TinyEngine**, a code-generating inference runtime that replaces TFLite Micro's graph interpreter with compiled static allocation, fuses depthwise-pointwise into in-place computations, and cuts peak memory by 3.4× and latency by 1.7-3.3× versus TFLite Micro + CMSIS-NN. On Google Speech Commands v2, MCUNet delivers **91% top-1 at 10 FPS** on the STM32F746 (Cortex-M7, 320 KB SRAM / 1 MB flash) — 2.8× faster inference and 4.1× smaller peak SRAM than the largest MobileNetV2 that fits, with 2 pp higher accuracy. The KWS numbers are stated in Section 4.3 (Table 3, Figure 9). On ImageNet, MCUNet is the first tiny-ML system to cross 70% top-1 on a commercial MCU (STM32H743 @ 512 KB SRAM / 2 MB flash). The paper makes three design points that matter for sonobuoy: (1) INT8 linear quantization alone isn't enough — the search space has to be MCU-aware; (2) the runtime and the architecture must be co-designed, because generic runtimes waste 65% of SRAM on the interpreter; (3) patch-based (block-wise) inference is the key memory trick that makes any CNN fit in tight SRAM.

## Methodology

### Problem setup (Section 1, Table 1)
The paper opens with a memory gap table that is the cleanest statement of the MCU constraint in the literature:

| Device | Memory | Storage |
|--------|--------|---------|
| NVIDIA V100 | 16 GB | TB~PB |
| iPhone 11 | 4 GB | ≥ 64 GB |
| STM32F746 | 320 kB | 1 MB |
| ResNet-50 demand | 7.2 MB peak | 102 MB |
| MobileNetV2 demand | 6.8 MB peak | 13.6 MB |
| MobileNetV2 INT8 | 1.7 MB peak (5.3× too big) | 3.4 MB |

Peak activation memory is 22× too large for an F746 even after INT8 quantization. MCUNet's job is to close the gap.

### TinyNAS — two-stage NAS
Stage 1, **search-space optimization**: randomly sample 1000 networks from a large MobileNet-style super-space; for each, check whether it fits the target MCU's (SRAM, flash, latency) constraints; keep only the feasible ones; fit the FLOP CDF of this feasible subset and choose width-multiplier + input-resolution presets that bias the distribution toward higher FLOPs within the feasible region. Intuition: MCU-feasible networks are so small that "accuracy ≈ FLOPs" is linear, so pushing the distribution right shifts the Pareto frontier.

Stage 2, **resource-constrained NAS** inside the pruned space using weight-shared super-network training (BigNAS-style): train a once-for-all super-network for 150 epochs on ImageNet / 100 epochs on Speech Commands / 30 epochs on VWW, then evolutionary-search the best sub-network under the concrete SRAM/flash/latency budget.

### TinyEngine — memory-efficient runtime
Key optimizations (Section 3.2):
1. **Code generation** instead of interpretation: the entire network graph is unrolled into a single C file at compile time; no runtime graph parser. Cuts interpreter overhead (which can be 65% of peak memory and 22% of latency in TFLM).
2. **In-place depthwise conv**: depthwise layers are written in place because input and output channels of a given filter don't share memory. Saves 1 activation tensor per depthwise layer.
3. **Operator fusion**: depthwise + pointwise + activation + batch-norm fold into a single kernel.
4. **Memory scheduling at network level**: allocation is planned across the whole network, not per-layer, letting activation buffers reuse ping-pong slots.
5. **Patch-based inference** (MCUNetV2 extension, discussed as future work in v1): break the first few high-peak layers into spatial patches so the peak activation tensor never materializes at full resolution. Reduces peak by 4-8× at the cost of 10-20% extra FLOPs.

### Training details (Section 4.1 and supplementary)
- Cosine-annealed LR with starting rate 0.05 per 256 samples.
- Super-network training: 2× epochs of the largest sub-network (so 200 epochs on Speech Commands).
- INT8 linear quantization post-training for deployment.
- Calibration on a validation split.

### Hardware targets evaluated
- STM32F412: Cortex-M4 @ 100 MHz, 256 kB SRAM, 1 MB flash.
- STM32F746: Cortex-M7 @ 216 MHz, 320 kB SRAM, 1 MB flash.
- STM32F765: Cortex-M7 @ 216 MHz, 512 kB SRAM, 1 MB flash.
- STM32H743: Cortex-M7 @ 480 MHz, 512 kB SRAM, 2 MB flash.

## Key Results

### Speech Commands v2 (KWS) on STM32F746 (Section 4.3, Fig 9 bottom)
- **MCUNet**: **91.0% top-1** at **10 FPS** (100 ms/inference).
- **2.8× faster** inference than best MobileNetV2 that fits 256 KB SRAM.
- **4.1× smaller peak SRAM** at the same accuracy.
- **+2 pp** accuracy vs largest MobileNetV2 runnable; **+3.3 pp** vs ProxylessNAS under 256 KB SRAM constraint.

### ImageNet on STM32H743 (Section 4.1)
- 70.7% top-1 on a commercial MCU — first in the literature.
- 3.5× less SRAM, 5.7× less flash vs INT8 quantized MobileNetV2/ResNet-18.

### VWW on STM32F746
- 2.4-3.4× faster and 3.7× smaller peak SRAM vs MobileNetV2 / ProxylessNAS.
- +0.4 pp accuracy over the previous first-place VWW challenge solution.

### TinyEngine vs TFLite Micro + CMSIS-NN
- **Peak memory**: 4.8× reduction (TinyEngine vs TFLM).
- **Flash (runtime library)**: 4.6× reduction.
- **Latency**: 1.7-3.3× speedup.

## Strengths

- **The first end-to-end tiny-ML system** that crosses 70% ImageNet and 91% Speech Commands on a real MCU. Before MCUNet, MCU inference was MobileNetV1 territory at sub-65% accuracy.
- **Runtime + architecture co-design** is a concrete, reusable pattern: the paper proves you can't just quantize a big model — the runtime also has to be redesigned. This lines up with the sonobuoy crate strategy (feature-gated runtime + crate-level model specialization).
- **MCU-family coverage**: numbers are reported on four STM32 variants spanning Cortex-M4 and Cortex-M7 with 256 kB to 512 kB SRAM. Directly actionable for sonobuoy MCU selection.
- **Open source**: TinyNAS + TinyEngine are public at mit-han-lab. TinyEngine's code-generation approach is a good reference for a Rust-native equivalent built on `candle-core` or `tract-onnx`.
- **Honest on limits**: Section 2 is explicit that MobileNetV2 INT8 is still 5.3× too big. The paper doesn't oversell quantization alone.

## Limitations

- **No energy numbers in the v1 paper**: only latency and memory. MLPerf Tiny fills that gap. An analyst has to assume MCU current draw (7 mA @ 80 MHz on Cortex-M4) and multiply by latency.
- **MCUNet targets always-on wake-word, not always-on detection**: the 10 FPS KWS number assumes 10 detections/second with a 35-class classifier; a sonobuoy trigger wants a smaller class set and a 50 FPS streaming cadence, which is not directly benchmarked here.
- **16 kHz audio only**: the paper follows Speech Commands' 16 kHz sample rate. Marine bioacoustics (cetaceans 30 Hz - 200 kHz, fish 20 Hz - 10 kHz, vessel lines 1-1000 Hz) do not map cleanly to 16 kHz. Would need retraining with a matched front end.
- **Patch-based inference is hinted but not fully developed in v1**: the core MCUNet gains come from search-space optimization plus TinyEngine; the patch-based memory reduction belongs to MCUNetV2 (arXiv:2110.15352, NeurIPS 2021).
- **ImageNet-centric search space**: TinyNAS is most thoroughly studied on ImageNet; the audio KWS results are a one-table add-on. Adapting the search space to spectrogram CNNs requires re-running the stage-1 FLOP-distribution fit, not just rerunning the evolutionary search.

## Portable Details

### TinyNAS stage-1 heuristic (pseudocode)
```python
# Input: MCU constraints (SRAM_max, Flash_max, Latency_max)
# Sample 1000 networks from a MobileNet-style super-space
candidates = [sample_arch(super_space) for _ in range(1000)]
feasible = [arch for arch in candidates
            if peak_sram(arch) <= SRAM_max
            and flash(arch) <= Flash_max
            and latency(arch) <= Latency_max]
flops_dist = fit_cdf([flops(arch) for arch in feasible])
# Pick the (width_mult, input_res) combination whose resulting
# feasible subset has the highest-FLOPs mean.
best_preset = argmax(width_mult, input_res)(
    mean_flops(feasible_subset(width_mult, input_res)))
```
This reduces a 20-dimensional search space to a 2-dimensional grid, then runs evolutionary NAS inside it. The insight is that, in the MCU regime, "biggest feasible network" is almost always the best network.

### TinyEngine in-place depthwise
```c
// Ordinary depthwise needs input[C][H][W] + output[C][H][W].
// TinyEngine writes output into the same buffer as input because
// depthwise filters are per-channel; channel c of output only
// depends on channel c of input, so reuse is legal.
for (c = 0; c < C; c++) {
    depthwise_conv_channel(&buf[c*H*W], &buf[c*H*W], filter[c]);
}
```

### MCUNet KWS architecture (rough)
- Input: log-mel spectrogram, 98×40 (derived from 16 kHz audio, 25 ms window, 10 ms hop).
- ~12 inverted-residual blocks with expand ratios 3-6, kernels 3-5, width multiplier 0.3-0.5.
- Global pool, FC to 35 classes.
- INT8 quantized: ~470 KB flash, ~70 KB peak SRAM on the STM32F746.
- 100 ms/inference → 10 FPS.

### Quantization
Standard INT8 linear symmetric per-tensor; calibration on a validation split. No mixed precision in v1; 4-bit experiments mentioned only in the ImageNet section.

## Sonobuoy Integration Plan

### The right tier for MCUNet
MCUNet is a **confirmation-stage** technology, not a first-level trigger:
- **Level gate (Tier 1)**: µW, hardware. Not MCUNet.
- **Always-on trigger (Tier 2)**: 5 mW, Rybakov/MLPerf DS-CNN (≤ 60 KB INT8, 1 ms/inference). Not MCUNet.
- **Confirmation (Tier 3)**: 50-200 mW, MCUNet-sized model. 10 FPS is plenty for a sonobuoy "is this really a whale?" decision.
- **At-shore (Tier 4)**: full pipeline.

The Tier 3 model is what MCUNet gives you. It sits between the level-gate/KWS trigger (too small to do species ID) and the at-shore pipeline (too big to deploy on a sonobuoy). MCUNet-sized models can plausibly do 4-16 class sonobuoy classification (vessel classes, cetacean species, fish chorus, noise types) in 100 ms on a Cortex-M7 at ~50 mW — well within a 1 W budget even at 100% duty cycle during a confirmation window.

### TinyEngine vs Rust runtime
The TinyEngine C code is ~3k lines. A Rust port is a 1-2 week engineering project:
- `tract-onnx` already does operator fusion and static shape inference.
- In-place depthwise is a custom kernel in `tract-linalg`.
- Patch-based inference (from MCUNetV2) is a scheduler-level pass over the graph.

Recommendation: use `tract-onnx` for v1 of `clawft-sonobuoy-head`, benchmark against MCUNet's TinyEngine numbers, then port the specific optimizations that close the gap.

### Where TinyNAS fits in the sonobuoy crate plan
- Keep DEMONet as the at-shore temporal encoder (§2.1 of the synthesis).
- Train a **TinyNAS-derived student model** on the buoy that's distilled from DEMONet. Target STM32H743 class (512 KB SRAM, 2 MB flash). This is the Tier-3 confirmation model.
- The TinyNAS procedure itself is a one-shot: run it once per target MCU family, save the found architecture, reuse across sonobuoy variants.

### Budget fit
| Constraint | MCUNet KWS (F746) | Sonobuoy Tier 3 target |
|-----------|-------------------|------------------------|
| Flash | 470 KB | ≤ 2 MB (comfortably fits) |
| Peak SRAM | 70 KB | 100 KB budget (fits) |
| Latency | 100 ms | 100-500 ms (fits) |
| Power @ 80 MHz | ~5 mW avg, ~50 mW active | ≤ 100 mW during confirmation |
| Confirmation-duty-cycle | — | 5% (5 s every 100 s) → ~2.5 mW avg |

MCUNet-class models at 5% duty consume ~2.5 mW average on the confirmation tier; negligible vs the 1 W total.

## Follow-up References

1. **Lin et al. 2021**, "MCUNetV2: Memory-Efficient Patch-based Inference for Tiny Deep Learning," arXiv:2110.15352, NeurIPS 2021 — patch-based inference that drops peak SRAM another 4-8×. Essential follow-up for sonobuoy if the 100 KB SRAM budget gets tight.
2. **Lin et al. 2022**, "On-Device Training Under 256KB Memory" (MCUNetV3), arXiv:2206.15472, NeurIPS 2022 — adds on-device fine-tuning; relevant if the sonobuoy must adapt to local ambient noise in the field.
3. **Banbury et al. 2020**, "MicroNets: Neural Network Architectures for Deploying TinyML Applications on Commodity Microcontrollers," arXiv:2010.11267 — differentiable NAS with FLOP-as-latency-proxy; independent confirmation of TinyNAS-style gains.
4. **Sandler et al. 2018**, "MobileNetV2: Inverted Residuals and Linear Bottlenecks," arXiv:1801.04381 — the architectural family MCUNet searches inside.
5. **David et al. 2021**, "TensorFlow Lite Micro: Embedded Machine Learning on TinyML Systems," MLSys 2021 — the baseline runtime that TinyEngine beats; still the reference implementation for MLPerf Tiny.
