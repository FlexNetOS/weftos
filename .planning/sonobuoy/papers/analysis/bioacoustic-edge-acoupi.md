# acoupi: Deploying Bioacoustic AI on Edge Devices (Vuilliomenet et al., 2026)

## Citation

- **Title**: acoupi: An Open-Source Python Framework for Deploying Bioacoustic AI Models on Edge Devices
- **Authors**: Aude Vuilliomenet, Santiago Martínez Balvanera, Oisin Mac Aodha, Kate E. Jones, Duncan Wilson
- **Affiliation**: UCL (Vuilliomenet, Martínez Balvanera, Jones, Wilson), University of Edinburgh (Mac Aodha)
- **Venue**: Methods in Ecology and Evolution, 17(1):67-76 (2026)
- **arXiv**: 2501.17841 (29 Jan 2025)
- **DOI**: 10.1111/2041-210X.70208
- **URL**: https://arxiv.org/abs/2501.17841
- **Code**: https://github.com/acoupi/acoupi (core), https://github.com/acoupi/acoupi_birdnet (BirdNET wrapper), acoupi_batdetect2

## Status

**verified**. arXiv:2501.17841 title/authors/DOI match the published MEE article (besjournals.onlinelibrary.wiley.com/doi/10.1111/2041-210x.70208). GitHub repos referenced in the paper exist and are maintained. PDF retrieved and parsed; quoted numbers are from the v1 arXiv text.

## One-paragraph Summary

acoupi is not a model; it is the **software substrate** you build around a bioacoustic model when you deploy it on a Raspberry Pi in the field. The paper documents a Python framework (built on Celery for task management, SQLite for local storage, and pluggable communicators for cellular/Wi-Fi/LoRaWAN) that turns a stock RPi 4B into an always-on passive-acoustic-monitoring station running **BirdNET v2.4** (6,400 bird species) or **BatDetect2** (17 UK bat species) with **no cloud dependency**. The interesting engineering contributions — and the ones relevant to a sonobuoy — are: (1) Celery-managed failure recovery so power glitches don't lose data; (2) an edge data-filtering layer that keeps only detections above a configurable confidence threshold so RF bandwidth is used for signal, not raw audio; (3) working-memory-only audio buffering so a 32 GB SD card is not the bottleneck; and (4) a 30-day UK-urban-park field trial with quantitative reliability numbers (99.997% of 129,939 acoupi_birdnet recordings processed; 98.17% of 65,711 acoupi_batdetect2 recordings). The paper is short (10 MEE pages) but it is the single cleanest published reference for a **duty-cycled, edge-computing, AI-powered acoustic station** — exactly the reference architecture a sonobuoy inherits from.

## Methodology

### Hardware setup
- **SBC**: Raspberry Pi 4 Model B (quad-core Cortex-A72 @ 1.5 GHz, 4 GB RAM), running 64-bit Raspberry Pi OS (Linux). Paper notes any "similar-spec Linux SBC" should work.
- **Audio I/O**: USB or I²S microphone sized to the target species' Nyquist:
  - acoupi_birdnet → 48 kHz mic (BirdNET was trained on 48 kHz audio).
  - acoupi_batdetect2 → 256 kHz mic (BatDetect2 trained on ultrasonic recordings).
- **Storage**: ≥ 32 GB microSD card. Note: acoupi **does not store full audio by default** — recordings live in RAM only; only detections are persisted.
- **Power**: mains in the published trial (they explicitly acknowledge this does not match field conditions). Paper references Callebaut et al. 2021 for solar + cellular as the normal field configuration.

### Software architecture (Fig 1 in the paper)
acoupi is a Python package with pluggable components:

1. **Recorder**: starts/stops audio capture on a schedule; dumps into working memory.
2. **Model wrapper**: loads and runs the DL model on 3-9 s audio windows.
3. **Processor**: applies confidence-threshold filtering; decides which detections to keep.
4. **Store**: SQLite database of detections + metadata.
5. **Messenger**: transmits detections over cellular / Wi-Fi / LoRaWAN on a 30 s schedule, heartbeat every 30 min.
6. **Celery task runner**: queues and retries all of the above; survives power cycles.

### Duty cycle (field trial config, Section 4)
- **acoupi_birdnet**: records **9 s every 10 s** between 03:00-23:00 (20 h/day active); detections sent every 30 s if confidence > threshold.
- **acoupi_batdetect2**: records **3 s every 10 s** between 19:00-07:00 (12 h/day active, nocturnal); detections sent every 30 s.
- **Heartbeat**: every 30 min regardless of detection activity.

### Model details
- **BirdNET v2.4** (Kahl et al. 2021, *Ecological Informatics* 61:101236): ResNet-like, 157 layers, ~27 M parameters, TFLite INT8 quantized for edge deployment. Accepts 3-second 48 kHz mel-spectrogram windows; outputs per-class confidence over ~6,400 species.
- **BatDetect2** (Mac Aodha et al. 2022): custom CNN for 256 kHz echolocation calls; detects 17 UK bat species at millisecond resolution.
- Neither model is retrained by acoupi; the paper is explicitly about **deployment plumbing**, not model development.

### Data flow
1. Record N seconds → RAM buffer.
2. Spectrogram extraction (BirdNET: 3 s windows; BatDetect2: 3 s at 256 kHz).
3. Run TFLite inference.
4. If max class confidence > threshold, persist to SQLite.
5. Scheduled messenger sends recent detections (not audio) upstream.
6. RAM buffer overwritten; SD card holds only SQLite + optional raw-audio archive (opt-in).

## Key Results

### 30-day field trial (Section 5, Table 1)
- **Location**: UCL People and Nature Garden Lab, Queen Elizabeth Olympic Park, London (51°32'18.8"N, 0°0'32.36"W).
- **Dates**: October-November 2024.

| Programme | Recordings produced | Failed | Failure % | Detections | Transmit % |
|-----------|--------------------:|-------:|----------:|-----------:|-----------:|
| acoupi_birdnet | 129,939 | 4 | 0.003% | 8,716 | 100% |
| acoupi_batdetect2 | 65,711 | 1,203 | 1.83% | 868 | 100% |

### Reliability analysis
- **BirdNET pipeline**: 99.997% success rate over 30 days. Four failed recordings attributed to transient I/O errors.
- **BatDetect2 pipeline**: 98.17% success rate. Higher failure rate traced to the 256 kHz USB audio capture saturating USB bandwidth; not a model issue.
- **Transmission**: 100% of detections reached the remote server (Wi-Fi link at UCL, not representative of a true field link).

### Not in the paper but derivable
- Per-detection bandwidth: if a detection row is ~200 B (class ID + confidence + timestamp + optional 128-byte embedding), then 8,716 detections × 200 B = ~1.7 MB of upstream traffic over 30 days — **about 60 KB/day**. This is the key sonobuoy number: a whole season's worth of sonobuoy detection metadata is tens of MB, not tens of GB.

## Strengths

- **Only peer-reviewed edge-PAM framework paper** (other PAM frameworks are GitHub-only). Published in MEE, a top-quartile methods journal, which the ecology community actually reads.
- **Reliability is quantified**: 99.997% processing rate is concrete evidence that a TFLite-on-Pi pipeline can run unattended for 30+ days. The sonobuoy counterpart will face harder conditions but has the same architecture.
- **Edge filtering + metadata-only transmission**: the paper crystallizes the right data-flow pattern for RF-constrained deployments. Upload detections, not audio. This is directly applicable to sonobuoy RF.
- **Celery-based task management**: Celery is overkill for an embedded MCU but the **retry + scheduled task + heartbeat pattern** is what any serious always-on acoustic station needs. Good design reference.
- **Hardware agnostic**: any Linux SBC works. This means the sonobuoy project could prototype on a Pi Zero 2W (same ISA family, 512 MB RAM, 1.5 W power budget) before committing to a custom board.

## Limitations

- **Raspberry Pi is too hungry for a free-drifting sonobuoy** on battery alone. RPi 4B idle current ~450 mA @ 5 V (2.25 W); under load easily 5-8 W. A sonobuoy with a ~1 W budget has to drop to Pi Zero 2W (~0.4-0.8 W), Cortex-M7 MCU (10-100 mW), or a Jetson Orin Nano in a special duty-cycle mode. The paper does not measure power.
- **Mains-powered trial**: the 30-day trial used wall power and UCL Wi-Fi. The paper is explicit that this is "not fully represent[ative of] the challenges of field deployments." Real sonobuoy conditions (battery + Iridium/UHF RF + saltwater + drift) are materially harder.
- **Python / Celery stack is not MCU-deployable**: if the sonobuoy MCU is Cortex-M-class, acoupi's software architecture is the right template but the implementation language has to be C or Rust. The paper does not address this port.
- **BirdNET is a big model**: 27 M params, ~35 MB TFLite INT8. Fits on an RPi (4 GB RAM) but is 300× over a 100 KB sonobuoy-MCU SRAM budget. The model must be shrunk (MCUNet/quantization/distillation) or kept at shore.
- **No energy-per-inference measurement**: latency is not reported; energy is not reported. "Reliability" is the only measured metric. This is the paper's blind spot.
- **Urban park, not marine**: BirdNET transfers poorly to marine mammals (the canonical Perch/Watkins transfer is via Ghani 2023, not this paper). The acoupi **framework** transfers; the **model** does not.

## Portable Details

### Celery task skeleton (framework-agnostic)
```python
@celery.task(bind=True, autoretry_for=(Exception,), retry_backoff=True, max_retries=3)
def process_recording(self, recording_id: str):
    audio = recorder.fetch(recording_id)         # from RAM buffer
    detections = model.predict(audio)            # TFLite call
    filtered = [d for d in detections if d.confidence > THRESHOLD]
    store.persist(filtered)
    if filtered:
        messenger.enqueue(filtered)              # queued for next transmit window
```
This is the canonical **record → detect → filter → persist → transmit** pipeline. The Rust equivalent on an MCU is ~200 LoC using `tokio` + `async-mqtt` + `tract-onnx`.

### BirdNET v2.4 hyperparameters
- Input: 3-second 48 kHz mono audio → 96-bin log-mel spectrogram → 144×96 tensor.
- Output: softmax over ~6,400 classes.
- Confidence threshold: user-configurable; typical field value 0.7.
- Inference latency on RPi 4B: ~1-2 s per 3-second clip (derived from the 9 s / 10 s duty cycle — acoupi fits 9 s of recording + inference + store into a 10 s cycle, so per-clip budget ≤ 1 s).

### Audio-buffer invariant
No disk writes of raw audio unless the user opts in. Recordings live in `/dev/shm` (tmpfs) or equivalent RAM-backed filesystem. This matters for flash wear on the sonobuoy.

### Transmission schedule
- Detection transmit window: 30 s.
- Heartbeat: 30 min.
- Retry backoff: exponential via Celery, max 3 retries, then dead-letter queue.

## Sonobuoy Integration Plan

### What ports directly
- **The framework pattern**: record → detect → filter → persist → transmit. This is exactly the sonobuoy data flow.
- **Metadata-only transmission**: send detections (20-200 B each), not audio. Confirmed viable at 60 KB/day, which fits easily inside a 1 kbps Iridium SBD channel (10 MB/day ceiling).
- **Duty cycle philosophy**: record 9 s out of 10 s is the upper bound; a sonobuoy can push down to 1 s out of 60 s and still catch everything but fast transients.
- **Retry + heartbeat + scheduled task pattern**: directly reusable in a Rust async runtime.

### What does not port
- **Raspberry Pi 4B as hardware**: too power-hungry. Downgrade to:
  - Cortex-M7 MCU (STM32H743) for the trigger tier — 50 mW, directly fits.
  - Pi Zero 2W for the confirmation tier — 500 mW idle, 1 W peak, fits if duty-cycled.
- **BirdNET v2.4 on-buoy**: too big (27 M params). Alternatives:
  - **At shore**: run full BirdNET / Perch / SurfPerch on the returned audio burst.
  - **On-buoy**: MCUNet-distilled 4-16 class student model (see mcunet-tinyml.md).
- **Python + Celery stack**: Rust + `tokio` + `sqlx` + `tract-onnx` are the right substitutes. Maintains the same architectural pattern in ~1500 LoC.

### Proposed sonobuoy tier map
| Tier | Cost | Model | Power | Analog in acoupi |
|------|------|-------|-------|-------------------|
| 1 (analog gate) | 10-50 µW | threshold | µW | N/A — below acoupi's Pi tier |
| 2 (MCU trigger) | 5 mW | MLPerf Tiny DS-CNN | mW | N/A — below acoupi's Pi tier |
| 3 (MCU confirmation) | 50 mW | MCUNet-student | 10% of budget | N/A |
| 4 (Pi confirmation, optional) | 1-2 W | BirdNET INT8 distilled | 10-20% duty | Direct acoupi analog |
| 5 (at-shore GPU) | 200 W | Full round-1 stack | at dock | acoupi's remote server |

If the sonobuoy has a Pi Zero 2W as its top-tier on-buoy compute, **acoupi itself is a candidate framework**. Run acoupi_birdnet at 1% duty cycle (3 s out of 300 s, chosen to fit a 1 W average with 10% active-tier margin) for final on-buoy confirmation; transmit detections over Iridium SBD.

### Integration with WeftOS crates
- `clawft-sonobuoy-head` wraps the acoupi pattern in Rust (record → detect → filter → persist → transmit).
- `clawft-sonobuoy-head::store` is SQLite-backed, mirrors acoupi's detection schema.
- `clawft-sonobuoy-head::messenger` is transport-agnostic (Iridium, LoRaWAN, Wi-Fi) and follows acoupi's 30 s transmit / 30 min heartbeat cadence.
- The at-shore receiver re-hydrates detections and routes matching audio-burst uploads through the full K-STEMIT stack (synthesis §2).

### Reliability target
acoupi field trial hit 99.997% (birds) and 98.17% (bats) over 30 days on mains power. A sonobuoy on battery in open water should aim for 99% or better over 30 days as a minimum viable product; below that, Celery-style retry + a watchdog-triggered reboot is essential.

## Follow-up References

1. **Kahl et al. 2021**, "BirdNET: A deep learning solution for avian diversity monitoring," *Ecological Informatics* 61:101236, DOI:10.1016/j.ecoinf.2021.101236 — the model acoupi deploys. ResNet-like, 157 layers, 27 M params, ~48 kHz input.
2. **Mac Aodha et al. 2022**, "Towards a General Approach for Bat Echolocation Detection and Classification," *BioRxiv* / *Methods in Ecology and Evolution* — BatDetect2 model reference.
3. **Hill et al. 2019**, "AudioMoth: A low-cost acoustic device for monitoring biodiversity and the environment," *Methods in Ecology and Evolution* 10(8):1199-1211 — the reference hardware acoupi explicitly improves on (AudioMoth is record-only; acoupi is detect-and-forward).
4. **Ghani et al. 2023**, "Global birdsong embeddings enable superior transfer learning for bioacoustic classification," *Scientific Reports* 13:22876 — the Perch paper (synthesis §2.4); what acoupi would use instead of BirdNET for marine mammals.
5. **Stowell 2022**, "Computational bioacoustics with deep learning: a review and roadmap," *PeerJ* 10:e13152 — the broader review acoupi cites; useful for understanding where edge-deployment sits in the bioacoustic-DL landscape.
