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
- **CPU:** 11th Gen Intel Core i5-11400H @ 2.70GHz (6 Cores, 12 Threads)
- **RAM:** 16 GB DDR4
- **OS / Target:** Linux 6.x / `x86_64-unknown-linux-gnu`
- **Discriminant:** 1024-bit fundamental negative prime discriminant $D = -p$

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
