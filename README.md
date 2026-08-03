<div align="center">

# `kyn-vdf`

**A pure Rust, WebAssembly-native Wesolowski Verifiable Delay Function (VDF) verifier over Imaginary Quadratic Class Groups.**

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io/crates/kyn-vdf)
[![Documentation](https://docs.rs/kyn-vdf/badge.svg)](https://docs.rs/kyn-vdf)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
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

---

## Hardware Benchmarks

The following empirical benchmarks demonstrate the fundamental property of Wesolowski VDFs: **Proving time scales linearly $\mathcal{O}(T)$, while Verification time remains flat and logarithmic $\mathcal{O}(\log T)$**.

### Test Machine Specifications
- **CPU:** 11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz (6 Cores, 12 Threads)
- **RAM:** 16 GB DDR4
- **OS / Target:** Linux 6.x / `x86_64-unknown-linux-gnu`
- **Discriminant Size:** 1024-bit fundamental negative prime discriminant $D = -p$ ($p \equiv 7 \pmod 8$)

### Empirical Results

| Iterations ($T$) | Prove Time (C++ Reference) | `kyn-vdf` Verify Time (Pure Rust) | Speedup Factor | Pure Rust Valid? | Tamper Rejection? |
|:---:|:---:|:---:|:---:|:---:|:---:|
| **100** | $6.57\text{ ms}$ | $42.26\text{ ms}$ | $0.15\times$ | ✅ **PASS** | ✅ **REJECTED** |
| **1,000** | $17.05\text{ ms}$ | $118.30\text{ ms}$ | $0.14\times$ | ✅ **PASS** | ✅ **REJECTED** |
| **10,000** | $84.24\text{ ms}$ | $108.70\text{ ms}$ | $0.77\times$ | ✅ **PASS** | ✅ **REJECTED** |
| **50,000** | $372.96\text{ ms}$ | $109.85\text{ ms}$ | $3.39\times$ | ✅ **PASS** | ✅ **REJECTED** |
| **100,000** | $733.85\text{ ms}$ | $108.20\text{ ms}$ | $6.78\times$ | ✅ **PASS** | ✅ **REJECTED** |
| **250,000** | $1.81\text{ s}$ | $108.00\text{ ms}$ | $16.76\times$ | ✅ **PASS** | ✅ **REJECTED** |
| **500,000** | $3.54\text{ s}$ | **$109.71\text{ ms}$** | **$32.26\times$** | ✅ **PASS** | ✅ **REJECTED** |

> **Note:** At $500,000$ iterations, verification in pure Rust is **$32\times$ faster** than proof generation, validating $100\%$ of authentic proofs and rejecting $100\%$ of malformed/tampered inputs.

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
use num_bigint::BigInt;

fn main() {
    let seed = b"kinetic-class-group-seed";
    let discriminant = create_discriminant(seed, 1024);

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
 
Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).
