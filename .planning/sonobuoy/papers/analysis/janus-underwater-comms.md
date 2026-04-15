# Potter, Alves, Green, Zappa, Nissen, McCoy 2014 — JANUS Underwater Communications Standard

## Citation

- **Authors**: John R. Potter, João Alves, Dale Green, Giovanni
  Zappa, Ivor Nissen, Kim McCoy
- **Title**: "The JANUS Underwater Communications Standard"
- **Venue**: Proc. 2014 IEEE Third Underwater Communications
  Conference and Workshop (UComms 2014), Sestri Levante, Italy,
  September 2014
- **DOI**: https://doi.org/10.1109/UComms.2014.7017134
- **Open PDF (NATO STO/CMRE open repository)**:
  https://repository.oceanbestpractices.org/bitstream/handle/11329/1304/14_Potter_Alves_Green_Zappa_Nissen_McCoy.pdf
- **NATO news (STANAG 4748 ratification)**:
  https://www.nato.int/cps/bu/natohq/news_143247.htm
- **IEEE Spectrum coverage**:
  https://spectrum.ieee.org/nato-develops-first-standardized-acoustic-signal-for-underwater-communications

## Status

**Verified.** The IEEE Xplore record is at DOI 10.1109/UComms.2014.7017134,
ADS bibcode 2014ucom.conf...19P, Semantic Scholar paper ID
76b4da4d1968ef341d129d38b76f0027439511e0. The open-access PDF at
oceanbestpractices.org is the canonical mirror. JANUS was ratified as
**NATO STANAG 4748** on 24 March 2017, making it the world's first
digital underwater-communications standard adopted by any
international body. The standard is publicly documented at
https://www.januswiki.com/ (CMRE-maintained) and has a reference
implementation in C (GPL) and Python (BSD).

## Historical context

Underwater acoustic communications prior to 2014 was a Balkanized
landscape: every modem vendor (Teledyne Benthos, LinkQuest, EvoLogics,
WHOI Micro-Modem, Sercel, Kongsberg) used a proprietary encoding,
carrier, and MAC. Two modems from different vendors could not talk to
each other even if they shared a channel. JANUS was CMRE's (NATO
Centre for Maritime Research and Experimentation, La Spezia) response:
**a deliberately simple, robust, freely-documented baseline protocol
that all underwater systems agree to understand**. The design
prioritizes ubiquity and robustness over data rate: every NATO
platform, regardless of vendor, should be able to broadcast an
identity + intent packet that any other NATO platform decodes. JANUS
sits as a "first-contact" layer under proprietary high-rate protocols.

For the sonobuoy ranging project, JANUS matters for four reasons:
(1) it gives us the **canonical waveform, frame structure, and MAC**
for our own inter-buoy ranging broadcasts; (2) it's designed for
*exactly* the short-packet broadcast pattern that OWTT ranging wants;
(3) it's freely implementable (no IP lock-in); and (4) NATO
ratification means third-party receivers (naval platforms, other
sonobuoys, research vessels) can decode our pings for free.

## Core content

### Physical layer

- **Carrier frequency**: 11.520 kHz (center), 4.160 kHz bandwidth
  (9.44-13.60 kHz). The band is chosen as a compromise between
  range (lower is better) and bandwidth (higher is better) and
  avoids the 38 kHz echosounder fundamental and the 12 kHz LBL
  classical band.
- **Modulation**: FH-BFSK — Frequency-Hopped Binary Frequency Shift
  Keying. Each bit is sent as a tone pair (one for 0, one for 1);
  the pair hops across 13 equally-spaced frequency slots spanning
  the band. This gives Doppler and fading robustness at the cost
  of spectral efficiency.
- **Symbol rate**: 160 symbols/s (one tone per 6.25 ms).
- **Bit rate after FEC**: 80 bps effective (1/2 convolutional
  encoder, constraint length 9).
- **Chip duration**: ~6.25 ms — this is the **matched-filter timing
  resolution**: ~1 ms for timing estimation using chirp preamble,
  meaning ~1.5 m range resolution.

### Frame structure

Every JANUS packet begins with a **32-chirp preamble** (the "chirp
wake-up") for detection and synchronization, followed by a 64-bit
**base frame** containing:

    | version (4) | class_user_id (8) | application_type (6) |
    | tx/rx flag (1) | forwarding_cap (1) | mobility flag (1) |
    | schedule flag (1) | payload_size (7) | cargo_size (10) |
    | payload_data (variable) | crc (8) |

Then a variable-length **cargo** of up to 4088 bits at the sender's
discretion. The base frame is MUST-decode; cargo is optional.

### MAC / scheduling

JANUS is broadcast-only (no handshake). Channel access is either
(a) **slotted ALOHA** (random backoff), (b) **time-triggered TDMA**
with an externally-provided clock, or (c) **schedule-advertised**
(one buoy acts as clock master and announces slot assignments).
The standard does not mandate any one — it specifies the frame
structure so different MACs can interoperate.

### Accuracy numbers (Potter-Alves 2014 + 2017 STANAG trials)

- **Detection range**: 2-5 km typical, up to 20 km in deep water
  with high TX power.
- **SNR threshold**: 0 dB post-processing (matched filter +
  convolutional decode gives ~6-8 dB processing gain).
- **Raw BER**: <10⁻⁴ at typical SNR (~5 dB input).
- **Timing precision** (when used for OWTT ranging): ~1 ms RMS
  from the chirp preamble matched filter → **1.5 m range RMS**
  equivalent.
- **Preamble-only ranging**: ~100 µs with high-SNR chirp
  (bandwidth-limited) → 15 cm range resolution — similar to WHOI
  Micro-Modem performance.
- **Packet duration**: 400 ms for a 64-byte cargo.

### The ranging use case (JANUS §6.2)

The standard explicitly documents using the chirp preamble as a
ranging reference. Two broadcasts exchanged between a pair yields a
**two-way timing pair**; combining gives an RTT. One-way ranging
works when both sides share a synchronous clock (JANUS advertises a
clock-bias field in the base frame for this case). The `schedule
flag` is used to announce a buoy's transmission slot so other buoys
listen in the right window.

## Portable details — JANUS as sonobuoy ranging waveform

### Why JANUS is the right physical layer

For inter-buoy ranging at 100 m - 5 km spacing, JANUS's ~12 kHz /
~4 kHz band and ~1 ms timing resolution match the engineering
requirements well:
- **Range resolution**: 1.5 m (full frame) or 15 cm (preamble-only)
  — well below the ranging error target.
- **Power**: ~10-20 W acoustic TX, ~0.1 J per packet — fits a
  sonobuoy power budget.
- **Interoperability**: anyone decoding JANUS can decode our
  broadcasts (good or bad depending on operational posture).
- **Hardware**: commodity. Every modern underwater acoustic modem
  supports JANUS in firmware.
- **Free implementation**: C (GPL) and Python (BSD) reference
  stacks at januswiki.com.

### Frame content for ranging broadcast

```rust
#[derive(Debug, Clone)]
pub struct JanusRangingFrame {
    // JANUS base frame (64 bits)
    pub version: u8,              // 4 bits, = 0x3
    pub class_user_id: u8,        // 8 bits, CMRE-allocated
    pub application_type: u8,     // 6 bits, = "sonobuoy-ranging"
    pub tx_rx_flag: bool,
    pub forwarding_cap: bool,
    pub mobility_flag: bool,      // = true for drifting buoy
    pub schedule_flag: bool,
    pub payload_size: u8,         // 7 bits
    pub cargo_size: u16,          // 10 bits

    // Ranging cargo (256 bits)
    pub node_id: u16,
    pub tx_time_us: u64,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub gps_time_us: u64,
    pub gps_sigma_m: f32,
    pub clock_bias_est_us: f32,
    pub clock_drift_est: f32,
    pub battery_mv: u16,
    pub mode: u8,                 // tactical/silent/pam
}
```

Frame size: 64 + 256 + CRC = 328 bits, ~4.1 s on-air at 80 bps.
Too slow. Use the JANUS base frame only (64 bits at 80 bps = 800 ms)
for timing, piggyback extended metadata on an out-of-band LoRa
channel — this is exactly what Otero-2023 does.

### Packet time = 1 s at 80 bps is a problem for TDMA

For N=8 buoys, TDMA slot would be ~1-2 s, giving epoch ~8-16 s.
Mitigation: use **preamble-only ranging** — each buoy transmits a
chirp preamble (32 chirps × 6.25 ms = 200 ms) at its scheduled
slot, encoding the node ID in the preamble's frequency pattern.
This drops the ping duration to ~250 ms and allows N=8 buoys at
~2 s epoch (0.5 Hz per-buoy update).

### Alternative: WHOI Micro-Modem "mini-packet" compatibility

Micro-Modem Rate-0 uses a 13-tone FH-FSK at 9-14 kHz with 512 ms
LFM chirp preamble. A clawft JANUS-compatible frame can also be
decoded by µModem firmware in hybrid mode, giving free
interoperability with WHOI-instrumented vessels.

## Integration with the sonobuoy stack

JANUS is the **recommended waveform** for the inter-buoy ranging
broadcasts — it is the right frequency, the right timing precision,
the right MAC flexibility, and crucially it is a NATO standard so
the pings are interoperable with naval platforms. In the K-STEMIT-
extended architecture JANUS slots in at the physical layer below the
`clawft-sonobuoy-ranging` crate: the ranging module builds
`JanusRangingFrame`s (or preamble-only mini-frames) and hands them
to an acoustic-modem abstraction whose backend can be a real µModem,
a JANUS reference implementation, or a simulator. The matched-filter
timing precision (~1 ms full frame, ~100 µs preamble) sets the
ranging error floor before SSP uncertainty, consistent with the
meter-scale accuracy target. The JANUS `schedule_flag` is used to
announce TDMA slot ownership, which becomes the ranging subsystem's
coordination protocol — implemented as a WeftOS `mesh_*` gossip
message and updated by the Raft leader (per ADR-062 compliance
pattern). The NATO STANAG 4748 ratification gives the project
regulatory cover for deploying active acoustic transmitters in
coastal waters.

## Strengths

1. **NATO standard** — formally ratified 24 March 2017 as STANAG
   4748. Deploying JANUS gives institutional cover for operating
   active transmitters in international waters.
2. **Free and open** — reference implementations in C (GPL) and
   Python (BSD), full specification at januswiki.com. No IP
   licensing cost.
3. **Interoperable** — any NATO platform decodes JANUS. Sonobuoy
   broadcasts become legible to naval assets, research vessels,
   and other sonobuoy fields.
4. **Designed for robustness, not speed** — FH-BFSK + 1/2 FEC holds
   packets together at SNRs where proprietary high-rate modulations
   fail. Matches the drifting-buoy operating environment.
5. **Chirp preamble ranging built-in** — the 32-chirp wake-up
   doubles as a ranging reference; no additional waveform needed.

## Limitations

1. **Low data rate** — 80 bps after FEC is 50-500× slower than
   proprietary modems (Teledyne 15.4 kbps). A full
   JanusRangingFrame takes ~4 s on-air; preamble-only ranging is
   faster but loses payload.
2. **No built-in security** — JANUS broadcasts are plaintext;
   spoofing and replay attacks are trivial. The WeftOS rvf-crypto
   layer must sign-and-authenticate.
3. **Stealth incompatible** — active transmission at 190 dB re 1 µPa
   is loud and geographically locatable. Tactical silent-mode
   operation requires a different scheme.
4. **4 kHz band congestion** — the 9-14 kHz band also hosts LBL
   transponders (Hunt-1974 class) and some high-power military
   sonars. Operational deconfliction is required.
5. **FH-BFSK is not bandwidth-efficient** — JANUS uses ~4 kHz of
   channel for ~80 bps of throughput; LFM chirp with PSK achieves
   ~10-100× the spectral efficiency but loses the FH anti-fading
   benefit.

## Follow-up references

1. **NATO STANAG 4748 (Edition 1)** (2017). *Digital Underwater
   Signalling Standard for Network Node Discovery &
   Interoperability*. Formal NATO Standardization Document. The
   authoritative version of JANUS.
2. **januswiki.com** (CMRE, ongoing). Full specification, reference
   implementations, test vectors, example applications.
3. **Freitag, L., Grund, M., Singh, S., Partan, J., Koski, P.,
   Ball, K.** (2005). "The WHOI Micro-Modem: an Acoustic
   Communications and Navigation System for Multiple Platforms."
   MTS/IEEE OCEANS 2005. WHOI's equivalent — not NATO-standard but
   the de-facto research-lab choice.
4. **Preisig, J.** (2007). "Acoustic propagation considerations for
   underwater acoustic communications network design." *ACM
   SIGMOBILE Mobile Computing and Communications Review* 11(4):2.
   Foundation paper on why underwater networking is hard.
5. **Alves, J., Ribas, L., Zappa, G., Carreras, M.** (2016).
   "JANUS-based services for Operationally Relevant Underwater
   Applications." Proc. OCEANS 2016. Operational use cases of
   JANUS — direct predecessor to a sonobuoy-ranging application.
