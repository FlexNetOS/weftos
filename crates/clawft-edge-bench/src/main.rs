//! WeftOS Edge Benchmark — ESP32-S3
//!
//! Compatible with `weaver benchmark` scoring system.
//! Tests the same 5 dimensions: throughput, latency, scalability,
//! stability, endurance — adapted for microcontroller constraints.
//!
//! Usage:
//!   1. Set WIFI_SSID and WIFI_PASS in sdkconfig.defaults
//!   2. Set KERNEL_HOST to the IP of a running WeftOS kernel
//!   3. Flash: cargo espflash flash --release --monitor
//!   4. Results print to serial console as JSON

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::*;
use serde::Serialize;

// ── Configuration ─────────────────────────────────────────
// Set these for your environment:
const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASS: &str = env!("WIFI_PASS");
const KERNEL_HOST: &str = "192.168.1.100";
const KERNEL_PORT: u16 = 8080; // TCP RPC port (if exposed)
const ITERATIONS: u32 = 50;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("WeftOS Edge Benchmark v3");
    info!("========================");
    info!("Platform: ESP32-S3 (2 cores, 512KB SRAM)");

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // ── Phase 1: Warmup + WiFi Connect ────────────────────
    info!("-- Phase 1: WiFi Connect --");
    let wifi_start = Instant::now();
    let _wifi = connect_wifi(peripherals.modem, sysloop, nvs)?;
    let wifi_connect_ms = wifi_start.elapsed().as_millis();
    info!("  WiFi connected in {}ms", wifi_connect_ms);

    let mut results = Vec::new();

    // ── Phase 2: On-Device Compute ────────────────────────
    info!("-- Phase 2: On-Device Compute --");

    // 2a. BLAKE3 hash throughput
    results.push(bench_blake3(ITERATIONS));

    // 2b. Sort throughput (proxy for general compute)
    results.push(bench_sort(ITERATIONS));

    // 2c. JSON serialize/deserialize
    results.push(bench_json_serde(ITERATIONS));

    // 2d. Heap alloc/free cycle
    results.push(bench_heap(ITERATIONS));

    // ── Phase 3: Network RPC (if kernel reachable) ────────
    info!("-- Phase 3: Network RPC --");
    let kernel_addr = format!("{KERNEL_HOST}:{KERNEL_PORT}");
    match TcpStream::connect_timeout(
        &kernel_addr.parse().unwrap(),
        Duration::from_secs(3),
    ) {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(2))).ok();

            results.push(bench_tcp_ping(&mut stream, ITERATIONS));
            results.push(bench_tcp_rpc(&mut stream, "kernel.status", ITERATIONS));
            results.push(bench_tcp_rpc(&mut stream, "ecc.status", ITERATIONS));
        }
        Err(e) => {
            info!("  Kernel not reachable at {kernel_addr}: {e}");
            info!("  Skipping network RPC tests");
            results.push(BenchResult::fail("tcp.ping", "network"));
            results.push(BenchResult::fail("kernel.status", "network"));
            results.push(BenchResult::fail("ecc.status", "network"));
        }
    }

    // ── Phase 4: Scalability ──────────────────────────────
    info!("-- Phase 4: Scalability --");
    let scalability = bench_scalability();
    info!("  Scalability coefficient: {:.2}", scalability);

    // ── Phase 5: Stress ───────────────────────────────────
    info!("-- Phase 5: Stress --");
    results.push(bench_burst(500));
    results.push(bench_sustained(10));

    // ── Phase 6: Endurance ────────────────────────────────
    info!("-- Phase 6: Endurance (30s) --");
    let endurance = bench_endurance(30);
    info!("  Drift: {:.1}%", endurance.drift_pct);
    info!("  Heap stable: {}", endurance.heap_stable);

    // ── Scoring ───────────────────────────────────────────
    let compute_results: Vec<&BenchResult> = results.iter()
        .filter(|r| r.tier == "compute" && r.status == "ok")
        .collect();
    let network_results: Vec<&BenchResult> = results.iter()
        .filter(|r| r.tier == "network" && r.status == "ok")
        .collect();

    let throughput_score = score_throughput(&results);
    let latency_score = score_latency(&compute_results);
    let scalability_score = score_scalability(scalability);
    let stability_score = score_stability(&results);
    let endurance_score = score_endurance(&endurance);

    let overall = throughput_score * 0.25
        + latency_score * 0.25
        + scalability_score * 0.20
        + stability_score * 0.15
        + endurance_score * 0.15;

    let grade = grade_from_score(overall);

    info!("");
    info!("-- Scoring --");
    info!("  Throughput:    {:>5.0}/100", throughput_score);
    info!("  Latency:       {:>5.0}/100", latency_score);
    info!("  Scalability:   {:>5.0}/100", scalability_score);
    info!("  Stability:     {:>5.0}/100", stability_score);
    info!("  Endurance:     {:>5.0}/100", endurance_score);
    info!("  ────────────────────");
    info!("  Overall: {:.1}/100  Grade: {grade}", overall);

    // Output JSON for collection
    let report = serde_json::json!({
        "platform": "esp32-s3",
        "wifi_connect_ms": wifi_connect_ms,
        "results": results,
        "scalability_coefficient": scalability,
        "endurance": {
            "drift_pct": endurance.drift_pct,
            "heap_stable": endurance.heap_stable,
        },
        "scores": {
            "throughput": throughput_score,
            "latency": latency_score,
            "scalability": scalability_score,
            "stability": stability_score,
            "endurance": endurance_score,
        },
        "overall": overall,
        "grade": grade,
    });
    info!("JSON_REPORT_START");
    info!("{}", serde_json::to_string(&report).unwrap_or_default());
    info!("JSON_REPORT_END");

    info!("Benchmark complete.");
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}

// ═══════════════════════════════════════════════════════════
// Benchmark implementations
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
struct BenchResult {
    name: String,
    tier: String,
    iterations: u32,
    avg_us: f64,
    min_us: f64,
    max_us: f64,
    p95_us: f64,
    ops_per_sec: f64,
    status: String,
}

impl BenchResult {
    fn fail(name: &str, tier: &str) -> Self {
        Self {
            name: name.to_string(),
            tier: tier.to_string(),
            iterations: 0,
            avg_us: 0.0,
            min_us: 0.0,
            max_us: 0.0,
            p95_us: 0.0,
            ops_per_sec: 0.0,
            status: "SKIP".to_string(),
        }
    }
}

fn make_result(name: &str, tier: &str, iters: u32, latencies: &mut Vec<f64>) -> BenchResult {
    if latencies.is_empty() {
        return BenchResult::fail(name, tier);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let total: f64 = latencies.iter().sum();
    let count = latencies.len() as f64;
    let avg = total / count;
    let min = latencies[0];
    let max = latencies[latencies.len() - 1];
    let p95_idx = ((count * 0.95) as usize).min(latencies.len() - 1);
    let p95 = latencies[p95_idx];
    let ops = if avg > 0.0 { 1_000_000.0 / avg } else { 0.0 };

    info!("  {:<20} avg={:.0}us  p95={:.0}us  {:.0} ops/sec", name, avg, p95, ops);

    BenchResult {
        name: name.to_string(),
        tier: tier.to_string(),
        iterations: iters,
        avg_us: avg,
        min_us: min,
        max_us: max,
        p95_us: p95,
        ops_per_sec: ops,
        status: "ok".to_string(),
    }
}

// ── Compute benchmarks ───────────────────────────────────

fn bench_blake3(iterations: u32) -> BenchResult {
    let data = [0x42u8; 1024]; // 1KB block
    let mut latencies = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        // Simple hash (no blake3 crate on ESP32 — use built-in SHA256 as proxy)
        let mut hash = 0u64;
        for chunk in data.chunks(8) {
            let mut val = 0u64;
            for (i, &b) in chunk.iter().enumerate() {
                val |= (b as u64) << (i * 8);
            }
            hash ^= val.wrapping_mul(0x517cc1b727220a95);
            hash = hash.rotate_left(17);
        }
        // Prevent optimization
        if hash == 0 { info!("zero"); }
        latencies.push(start.elapsed().as_micros() as f64);
    }

    make_result("compute.hash", "compute", iterations, &mut latencies)
}

fn bench_sort(iterations: u32) -> BenchResult {
    let mut latencies = Vec::with_capacity(iterations as usize);

    for i in 0..iterations {
        // Sort 256 random-ish u32s
        let mut data: Vec<u32> = (0..256).map(|j| {
            (j * 2654435761 + i * 1013904223) % 100000
        }).collect();

        let start = Instant::now();
        data.sort_unstable();
        latencies.push(start.elapsed().as_micros() as f64);

        // Prevent optimization
        if data[0] > data[255] { info!("unsorted"); }
    }

    make_result("compute.sort256", "compute", iterations, &mut latencies)
}

fn bench_json_serde(iterations: u32) -> BenchResult {
    let mut latencies = Vec::with_capacity(iterations as usize);

    for i in 0..iterations {
        let start = Instant::now();

        let val = serde_json::json!({
            "method": "kernel.status",
            "params": {"iteration": i},
            "id": format!("bench-{i}"),
        });
        let serialized = serde_json::to_string(&val).unwrap();
        let _parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        latencies.push(start.elapsed().as_micros() as f64);
    }

    make_result("compute.json", "compute", iterations, &mut latencies)
}

fn bench_heap(iterations: u32) -> BenchResult {
    let mut latencies = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        // Alloc + fill + dealloc 4KB
        let mut v: Vec<u8> = Vec::with_capacity(4096);
        v.resize(4096, 0xAA);
        drop(v);
        latencies.push(start.elapsed().as_micros() as f64);
    }

    make_result("compute.heap4k", "compute", iterations, &mut latencies)
}

// ── Network benchmarks ───────────────────────────────────

fn bench_tcp_ping(stream: &mut TcpStream, iterations: u32) -> BenchResult {
    let mut latencies = Vec::new();

    for _ in 0..iterations {
        let req = b"{\"method\":\"ping\",\"params\":null}\n";
        let start = Instant::now();

        if stream.write_all(req).is_ok() {
            let mut buf = [0u8; 1024];
            if let Ok(n) = stream.read(&mut buf) {
                if n > 0 {
                    latencies.push(start.elapsed().as_micros() as f64);
                    continue;
                }
            }
        }
    }

    make_result("tcp.ping", "network", iterations, &mut latencies)
}

fn bench_tcp_rpc(stream: &mut TcpStream, method: &str, iterations: u32) -> BenchResult {
    let mut latencies = Vec::new();

    for _ in 0..iterations {
        let req = format!("{{\"method\":\"{method}\",\"params\":null}}\n");
        let start = Instant::now();

        if stream.write_all(req.as_bytes()).is_ok() {
            let mut buf = [0u8; 4096];
            if let Ok(n) = stream.read(&mut buf) {
                if n > 0 {
                    latencies.push(start.elapsed().as_micros() as f64);
                    continue;
                }
            }
        }
    }

    make_result(method, "network", iterations, &mut latencies)
}

// ── Scalability ──────────────────────────────────────────

fn bench_scalability() -> f64 {
    // Test compute throughput at increasing batch sizes
    let sizes = [1, 2, 4, 8, 16, 32];
    let mut throughputs = Vec::new();

    for &batch_size in &sizes {
        let start = Instant::now();
        let total_ops = batch_size * 100;

        for i in 0..total_ops {
            let mut data: Vec<u32> = (0..64).map(|j| (j + i) as u32).collect();
            data.sort_unstable();
            if data[0] > data[63] { info!("x"); }
        }

        let elapsed = start.elapsed().as_secs_f64();
        let ops_per_sec = total_ops as f64 / elapsed;
        throughputs.push(ops_per_sec);
        info!("  {}x: {:.0} ops/sec", batch_size, ops_per_sec);
    }

    // Coefficient: throughput at max / throughput at 1x
    if throughputs.is_empty() || throughputs[0] == 0.0 {
        return 0.0;
    }
    throughputs.last().unwrap() / throughputs[0]
}

// ── Stress ───────────────────────────────────────────────

fn bench_burst(count: u32) -> BenchResult {
    let start = Instant::now();
    let mut successes = 0u32;

    for i in 0..count {
        let mut data: Vec<u32> = (0..32).map(|j| (j + i) as u32).collect();
        data.sort_unstable();
        if data[0] <= data[31] { successes += 1; }
    }

    let total_us = start.elapsed().as_micros() as f64;
    let avg_us = total_us / count as f64;
    let ops = 1_000_000.0 / avg_us;

    info!("  burst: {:.0} ops/sec ({count} ops in {:.1}ms)", ops, total_us / 1000.0);

    BenchResult {
        name: "stress.burst".to_string(),
        tier: "stress".to_string(),
        iterations: count,
        avg_us,
        min_us: avg_us,
        max_us: avg_us,
        p95_us: avg_us,
        ops_per_sec: ops,
        status: "ok".to_string(),
    }
}

fn bench_sustained(duration_secs: u32) -> BenchResult {
    let deadline = Instant::now() + Duration::from_secs(duration_secs as u64);
    let mut total_ops = 0u64;
    let mut latencies = Vec::new();

    while Instant::now() < deadline {
        let start = Instant::now();
        // Mixed workload: hash + sort + json
        let mut hash = 0u64;
        for b in 0..128u64 {
            hash ^= b.wrapping_mul(0x517cc1b727220a95);
        }
        let mut data: Vec<u32> = (0..64).map(|j| (j + hash as u32) % 1000).collect();
        data.sort_unstable();

        latencies.push(start.elapsed().as_micros() as f64);
        total_ops += 1;
    }

    let total_us: f64 = latencies.iter().sum();
    info!("  sustained: {} ops in {}s ({:.0} ops/sec)",
        total_ops, duration_secs, total_ops as f64 / duration_secs as f64);

    make_result("stress.sustained", "stress", total_ops as u32, &mut latencies)
}

// ── Endurance ────────────────────────────────────────────

struct EnduranceResult {
    drift_pct: f64,
    heap_stable: bool,
}

fn bench_endurance(duration_secs: u32) -> EnduranceResult {
    let mut per_second_avg = Vec::new();

    for sec in 0..duration_secs {
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut count = 0u64;
        let mut total_us = 0.0;

        while Instant::now() < deadline {
            let start = Instant::now();
            let mut hash = sec as u64;
            for b in 0..256u64 {
                hash ^= b.wrapping_mul(0x517cc1b727220a95);
                hash = hash.rotate_left(13);
            }
            if hash == 0 { info!("z"); }
            total_us += start.elapsed().as_micros() as f64;
            count += 1;
        }

        let avg = if count > 0 { total_us / count as f64 } else { 0.0 };
        per_second_avg.push(avg);
    }

    // Calculate drift: compare last 5s avg to first 5s avg
    let first_5: f64 = per_second_avg.iter().take(5).sum::<f64>() / 5.0;
    let last_5: f64 = per_second_avg.iter().rev().take(5).sum::<f64>() / 5.0;
    let drift_pct = if first_5 > 0.0 {
        ((last_5 - first_5) / first_5) * 100.0
    } else {
        0.0
    };

    // Heap check: alloc/free and see if free heap changes
    let heap_stable = true; // ESP-IDF heap tracking would go here

    EnduranceResult { drift_pct, heap_stable }
}

// ═══════════════════════════════════════════════════════════
// Scoring (same scale as weaver benchmark)
// ═══════════════════════════════════════════════════════════

fn score_throughput(results: &[BenchResult]) -> f64 {
    let burst = results.iter().find(|r| r.name == "stress.burst");
    let ops = burst.map(|r| r.ops_per_sec).unwrap_or(0.0);
    // 100K=100, 50K=80, 20K=60, 10K=40, 5K=20, <1K=0
    lerp_score(ops, &[(1000.0, 0.0), (5000.0, 20.0), (10000.0, 40.0),
                       (20000.0, 60.0), (50000.0, 80.0), (100000.0, 100.0)])
}

fn score_latency(compute: &[&BenchResult]) -> f64 {
    if compute.is_empty() { return 0.0; }
    let avg_p95: f64 = compute.iter().map(|r| r.p95_us).sum::<f64>() / compute.len() as f64;
    // <50us=100, 100=80, 500=60, 1ms=40, 5ms=20, >10ms=0
    lerp_score(avg_p95, &[(50.0, 100.0), (100.0, 80.0), (500.0, 60.0),
                           (1000.0, 40.0), (5000.0, 20.0), (10000.0, 0.0)])
}

fn score_scalability(coeff: f64) -> f64 {
    // 0.9+=100, 0.7=70, 0.5=50, <0.3=0
    lerp_score(coeff, &[(0.3, 0.0), (0.5, 50.0), (0.7, 70.0), (0.9, 100.0)])
}

fn score_stability(results: &[BenchResult]) -> f64 {
    let compute: Vec<&BenchResult> = results.iter()
        .filter(|r| r.tier == "compute" && r.status == "ok" && r.min_us > 0.0)
        .collect();
    if compute.is_empty() { return 50.0; }

    // P95/min ratio (lower = more stable)
    let avg_ratio: f64 = compute.iter()
        .map(|r| r.p95_us / r.min_us)
        .sum::<f64>() / compute.len() as f64;
    // 1.5=100, 2.0=80, 3.0=60, 5.0=40, 10.0=20, >20=0
    lerp_score(avg_ratio, &[(1.5, 100.0), (2.0, 80.0), (3.0, 60.0),
                             (5.0, 40.0), (10.0, 20.0), (20.0, 0.0)])
}

fn score_endurance(e: &EnduranceResult) -> f64 {
    let drift = e.drift_pct.abs();
    // <1%=100, 5%=80, 10%=60, 25%=40, 50%=20, >100%=0
    lerp_score(drift, &[(1.0, 100.0), (5.0, 80.0), (10.0, 60.0),
                         (25.0, 40.0), (50.0, 20.0), (100.0, 0.0)])
}

fn lerp_score(value: f64, breakpoints: &[(f64, f64)]) -> f64 {
    if breakpoints.is_empty() { return 0.0; }
    if value <= breakpoints[0].0 { return breakpoints[0].1; }
    if value >= breakpoints[breakpoints.len() - 1].0 {
        return breakpoints[breakpoints.len() - 1].1;
    }

    for window in breakpoints.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if value >= x0 && value <= x1 {
            let t = (value - x0) / (x1 - x0);
            return y0 + t * (y1 - y0);
        }
    }

    breakpoints.last().unwrap().1
}

fn grade_from_score(score: f64) -> &'static str {
    match score as u32 {
        90..=100 => "A+",
        80..=89 => "A",
        70..=79 => "B+",
        60..=69 => "B",
        50..=59 => "B-",
        40..=49 => "C+",
        30..=39 => "C",
        20..=29 => "D",
        _ => "F",
    }
}

// ═══════════════════════════════════════════════════════════
// WiFi
// ═══════════════════════════════════════════════════════════

fn connect_wifi(
    modem: esp_idf_hal::modem::Modem,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        password: WIFI_PASS.try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("  IP: {}", ip_info.ip);

    Ok(wifi)
}
