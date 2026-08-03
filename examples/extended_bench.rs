//! Extended verification timing script for large iteration counts.
//!
//! Measures pure-Rust kyn-vdf verification time at T = 1M, 2M, 5M to confirm the
//! O(log T) flat-verification property holds at 7-figure iteration counts.
//!
//! NOTE: pre-computing y = x^(2^T) is the slow part (equivalent to proving).
//!       Only the `verify_wesolowski` call itself is measured.
//!
//! Run with:
//!   cargo run --release --example extended_bench

use kyn_vdf::chia::{create_discriminant, get_b, verify_wesolowski};
use kyn_vdf::math::Form;
use num_bigint::BigUint;
use std::time::Instant;

fn get_cpu_info() -> String {
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if line.starts_with("model name") {
                if let Some(name) = line.split(':').nth(1) {
                    return name.trim().to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║        kyn-vdf  ·  Extended Verification Timing (T ≥ 1M)        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("CPU : {}", get_cpu_info());
    println!("Rust: {}", std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".into()));
    println!("Disc: 1024-bit fundamental negative prime discriminant");
    println!();

    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let x = Form::generator(&d).expect("valid generator");

    let iter_counts: &[u64] = &[1_000_000, 2_000_000, 5_000_000];

    println!("  Precomputing proofs and outputs (this is the slow/proving step)…");
    println!();

    println!("┌─────────────────┬──────────────────────┬─────────────────────┬──────────────┐");
    println!("│ Iterations (T)  │ Precompute (proving) │ Verify (pure Rust)  │ Valid?       │");
    println!("├─────────────────┼──────────────────────┼─────────────────────┼──────────────┤");

    for &iters in iter_counts {
        // --- Pre-compute y = x^(2^T)  and  proof = x^floor(2^T / B) ---
        // This is the slow part — equivalent to running the VDF prover.
        let t_precompute = Instant::now();

        let exp = BigUint::from(2u32).pow(iters as u32);
        let y = x.pow(&exp, &d);

        let b_val = get_b(&d, &x, &y).expect("get_b failed");
        let r = BigUint::from(2u32).modpow(&BigUint::from(iters), &b_val);
        let proof_exp = &exp / &b_val;
        let proof = x.pow(&proof_exp, &d);

        let precompute_ms = t_precompute.elapsed().as_secs_f64() * 1000.0;
        let _ = r;

        // --- Measure pure-Rust verification only ---
        let t_verify = Instant::now();
        let valid = verify_wesolowski(&d, &x, &y, &proof, iters).expect("verify failed");
        let verify_ms = t_verify.elapsed().as_secs_f64() * 1000.0;

        println!(
            "│ {:>15} │ {:>18.2} ms │ {:>17.2} ms │ {}        │",
            format!("{}", iters),
            precompute_ms,
            verify_ms,
            if valid { "✅ PASS" } else { "❌ FAIL" }
        );
    }

    println!("└─────────────────┴──────────────────────┴─────────────────────┴──────────────┘");
    println!();
    println!("O(log T) confirmed if verify times are flat across all rows above.");
}
