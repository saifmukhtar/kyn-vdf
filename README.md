<div align="center">

# `kyn-vdf`

**A pure Rust, WebAssembly-native Wesolowski Verifiable Delay Function (VDF) verifier over Imaginary Quadratic Class Groups.**

[![Crates.io](https://img.shields.io/crates/v/kyn-vdf.svg)](https://crates.io/crates/kyn-vdf)
[![Documentation](https://docs.rs/kyn-vdf/badge.svg)](https://docs.rs/kyn-vdf)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![WASM Ready](https://img.shields.io/badge/wasm-ready-brightgreen.svg)](#webassembly-support)
[![Safety](https://img.shields.io/badge/unsafe-0%25-green.svg)](#security--formal-correctness)

</div>

---

## Overview

Verifiable Delay Functions (VDFs) require a prover to spend non-parallelizable sequential time evaluating a mathematical function, while allowing anyone to verify the output in exponentially faster time $\mathcal{O}(\log T)$.

Until now, the open-source ecosystem lacked a **pure Rust, zero-FFI implementation** capable of verifying Chia-compatible Wesolowski VDF proofs. Reference implementations relied on C++ binaries and `libgmp`, preventing execution in:
- **WebAssembly (WASM) & In-Browser Light Clients**
- **Native Mobile Apps (iOS & Android)** without complex C++ NDK cross-compilation
- **Smart Contracts** (CosmWasm, Substrate, Solana, NEAR)
- **Embedded & Pure Rust Environments**

`kyn-vdf` provides an independent, clean-room, 100% pure Rust implementation of binary quadratic class group arithmetic and Wesolowski verification, running seamlessly in any environment.

---

## Key Features

- **Pure Rust / Zero FFI:** Built on safe, arbitrary-precision arithmetic (`num-bigint`). Zero C/C++, GMP, or OS dependencies.
- **WASM Native:** Compiles out-of-the-box to `wasm32-unknown-unknown` for web browsers and mobile wallets.
- **Shanks' NUCOMP & NUDUPL:** Implements sub-quadratic binary quadratic form composition and squaring with partial Euclidean reduction.
- **Asymptotic Verification ($\mathcal{O}(\log T)$):** Verification takes constant logarithmic time ($\sim 100\text{ ms}$), even when proofs took minutes or hours to generate.
- **Chia-Compatible:** 100% test-vector compatible with Chia Network's reference 1024-bit class group VDF specification.
- **Fuzzing & Property Tested:** Hardened with `proptest` and `cargo-fuzz` against malformed inputs and boundary conditions.

> **Verifier-only by design.** `kyn-vdf` verifies Wesolowski proofs — it does not generate them. Proof generation requires $T$ sequential squarings (the delay itself) and is the job of a native VDF prover node, not a light client. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full design rationale.

---

## Benchmarks

All numbers below are **real measurements** from `cargo bench` and the chiavdf C++ reference prover running on the same machine. Reproduce them yourself:

```bash
# Pure Rust verification
cargo bench

# C++ prove times (requires kinetic-vdf with chiavdf)
cargo run --release --bin prove_timing -p kinetic-vdf
```

### Test Machine

| Field | Value |
|---|---|
| **CPU** | 11th Gen Intel Core i5-11400H |
| **Base / Boost Clock** | 2.70 GHz base / 4.10 GHz boost (observed during bench) |
| **Cores / Threads** | 6 cores, 12 threads |
| **L3 Cache** | 12 MB |
| **RAM** | 16 GB DDR4 (8.6 GB available during bench) |
| **OS** | Fedora 44, Linux kernel 7.1.3-200.fc44.x86_64 |
| **Target triple** | `x86_64-unknown-linux-gnu` |
| **Rust profile** | `--release` (optimized, no debug info) |
| **Discriminant** | 1024-bit fundamental negative prime $D = -p,\; p \equiv 7 \pmod 8$ |

### Class Group Arithmetic (per operation)

| Operation | Time |
|:---|:---:|
| NUDUPL squaring (1024-bit) | **~5.08 µs** |
| NUCOMP composition (1024-bit) | **~4.81 µs** |

### Prove vs. Verify — Full Comparison

| Iterations ($T$) | chiavdf Prove (C++) | chiavdf Verify (C++) | `kyn-vdf` Verify (Pure Rust) | vs. Prove |
|:---:|:---:|:---:|:---:|:---:|
| **100** | 13.77 ms | 14.80 ms | 12.70 ms | ~1× |
| **1,000** | 24.62 ms | 19.32 ms | 86.66 ms | 0.28× |
| **10,000** | 90.23 ms | 17.33 ms | 85.00 ms | 1.06× |
| **100,000** | 742.12 ms | 17.91 ms | 93.29 ms | **7.96×** |
| **500,000** | 3,577.65 ms | 17.85 ms | 82.04 ms | **43.6×** |

**Reading the table honestly:**

- The C++ chiavdf verifier (~14–19 ms) is faster than `kyn-vdf` (~82–93 ms) — it uses libgmp, an optimized C++ big-integer library. That's the cost of zero FFI and WASM compatibility.
- Despite being ~5× slower than C++ verify, `kyn-vdf` still verifies **43× faster** than C++ can *prove* at T=500,000.
- Once $T \geq 1{,}000$, `kyn-vdf` verification is **flat at ~82–93 ms** regardless of how large $T$ grows — confirming the $\mathcal{O}(\log T)$ guarantee.
- At T=100, pure Rust is actually faster than C++ verify (12.70 ms vs 14.80 ms) because $2^{100} < B$, making $r$ a smaller number.

### Extended Benchmark — $O(\log T)$ at Scale (T ≥ 1M)

Run with `cargo run --release --example extended_bench`:

| Iterations ($T$) | Prove time (pure Rust equiv.) | `kyn-vdf` Verify | Speedup vs. Prove |
|:---:|:---:|:---:|:---:|
| **1,000,000** | 259,632 ms (4.3 min) | **88.72 ms** | **2,926×** |
| **2,000,000** | 503,684 ms (8.4 min) | **90.36 ms** | **5,573×** |
| **5,000,000** | 1,288,749 ms (21.5 min) | **102.25 ms** | **12,603×** |

> **Verification is flat.** From T=500,000 to T=5,000,000 (10× more iterations), verify
> time increased by only ~20 ms (82 ms → 102 ms). Prove time scaled linearly by 360×.
> This is the $\mathcal{O}(\log T)$ guarantee demonstrated at real scale.

> **Note on "Prove time (pure Rust equiv.)":** The extended bench pre-computes
> $y = x^{2^T}$ in pure Rust using `num-bigint`. This is equivalent in work to proving,
> but slower than the C++ chiavdf prover which uses libgmp and hardware-optimized
> arithmetic. Real prove times with chiavdf C++ would be ~6–7× faster.


---

## Installation

Add `kyn-vdf` to your `Cargo.toml`:

```toml
[dependencies]
kyn-vdf = "0.1"
```

---

## Quick Start

### 1. One-Step Verification
Verify a 1024-bit Chia-compatible proof in one function call:

```rust
use kyn_vdf::verify_chia_vdf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let challenge = [0x42u8; 32];
    let proof_bytes: Vec<u8> = get_proof_from_network(); // 200 bytes: y || π
    let iterations = 500_000u64;

    let is_valid = verify_chia_vdf(&challenge, &proof_bytes, iterations, 1024)?;
    if is_valid {
        println!("✅ VDF Proof mathematically verified!");
    } else {
        println!("❌ Invalid proof rejected.");
    }

    Ok(())
}
```

### 2. Low-Level Class Group Arithmetic
Directly manipulate binary quadratic forms $(a, b, c)$ with Shanks' NUCOMP/NUDUPL:

```rust
use kyn_vdf::math::Form;
use kyn_vdf::chia::create_discriminant;

fn main() {
    let seed = b"kyn-vdf-seed";
    let discriminant = create_discriminant(seed, 1024).expect("valid seed");

    // Canonical generator element (2, 1, c)
    let g = Form::generator(&discriminant).expect("valid generator");

    // Fast squaring via Shanks' NUDUPL + Gauss reduction
    let g2 = g.square(&discriminant);

    // Fast composition via Shanks' NUCOMP
    let g3 = g.compose(&g2, &discriminant);

    assert!(g3.is_reduced());
}
```

---

## WebAssembly Support

`kyn-vdf` has zero native dependencies and compiles cleanly to WebAssembly:

```bash
cargo build --target wasm32-unknown-unknown --release
```

---

## Security & Formal Testing

- **Property Testing:** `tests/proptest_math.rs` validates group axioms (identity, associativity, inverse, and exponentiation) using `proptest`.
- **Differential Fuzzing:** `fuzz/fuzz_targets/fuzz_vdf.rs` continuously fuzzes deserialization and verification against malformed/arbitrary byte streams:
  ```bash
  cargo +nightly fuzz run fuzz_vdf
  ```
- **Negative Testing:** Strict validation ensures zero false positives on flipped bits, incorrect iteration counts, or non-reduced forms.

---

## Academic References

1. **Benjamin Wesolowski (2018):** *"Efficient Verifiable Delay Functions"*. [ePrint 2018/623](https://eprint.iacr.org/2018/623).
2. **Daniel Shanks (1989):** *"On Gauss and Composition I, II"*. Algorithmic NUCOMP and NUDUPL for quadratic forms.
3. **Henri Cohen (1993):** *"A Course in Computational Algebraic Number Theory"*, Springer-Verlag GTM 138 (Algorithms for Binary Quadratic Forms).
4. **Chia Network:** Reference C++ implementation (`chiavdf`).

---

## License

Dual-licensed under your choice of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

This is the standard dual-licensing approach used by the majority of the Rust ecosystem.
