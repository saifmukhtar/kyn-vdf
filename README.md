<div align="center">

# `kyn-vdf`

**A pure Rust, WebAssembly-native Wesolowski Verifiable Delay Function (VDF) verifier over Imaginary Quadratic Class Groups.**

[![Formal Security Audit](https://github.com/saifmukhtar/kyn-vdf/actions/workflows/audit.yml/badge.svg)](https://github.com/saifmukhtar/kyn-vdf/actions/workflows/audit.yml)
[![CI Pipeline](https://github.com/saifmukhtar/kyn-vdf/actions/workflows/ci.yml/badge.svg)](https://github.com/saifmukhtar/kyn-vdf/actions/workflows/ci.yml)
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
- **Asymptotic Verification ($\mathcal{O}(\log T)$):** Verification takes constant logarithmic time (~130ms native, ~380ms in-browser WASM), even when proofs took minutes or hours to generate.
- **Chia-Compatible:** 100% test-vector compatible with Chia Network's reference 1024-bit class group VDF specification.
- **Fuzzing & Property Tested:** Hardened with `proptest` and `cargo-fuzz` against malformed inputs and boundary conditions.

> [!IMPORTANT]
> **Verifier-only by design.** `kyn-vdf` verifies Wesolowski proofs — it does not generate them. Proof generation requires $T$ sequential squarings (the delay itself) and is the job of a native VDF prover node, not a light client. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full design rationale.

---

## Benchmarks & Cross-Platform Performance

`kyn-vdf` is heavily optimized for zero-cost abstraction and cross-platform native execution. 

We have conducted extensive performance profiling across multiple environments to prove that `kyn-vdf` is ready for production light clients, mobile apps, and desktop validators. By eliminating heavy `libgmp` dependencies, you can now natively verify massive proofs across:
- **Desktop (x86_64):** Via pure CLI terminal or native Tauri Rust FFI.
- **Mobile (ARM64):** Inside native Android Termux or via Flutter Dart FFI.
- **WebAssembly (WASM):** Inside Chromium & Firefox browser sandboxes.

> [!TIP]
> **View the full performance report:** Read the comprehensive [BENCHMARKS.md](BENCHMARKS.md) to see how `kyn-vdf` achieves blazing fast **~130ms** verification times natively, and effectively zero FFI overhead when integrated into Flutter or Tauri apps!


---

## Installation

`kyn-vdf` is officially published and available on [crates.io](https://crates.io/crates/kyn-vdf).

Add it to your `Cargo.toml`:

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

`kyn-vdf` has zero native dependencies and compiles cleanly to WebAssembly.

### For Web Developers (JavaScript / TypeScript)

You can compile this crate into a native NPM package that exposes `verifyChiaVdf` to JavaScript:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build the JS/TS package (outputs to ./pkg)
wasm-pack build --target web
```

Then in your web app (e.g., React, Vue, Vite, or a browser extension):

```typescript
import init, { verifyChiaVdf } from './pkg/kyn_vdf.js';

async function run() {
  await init(); // Initialize the WASM module
  
  const challenge = new Uint8Array(32); // e.g. from network
  const proof = new Uint8Array(200);    // e.g. from network
  
  try {
    const isValid = verifyChiaVdf(challenge, proof, 500000n, 1024);
    console.log("Proof valid:", isValid);
  } catch (e) {
    console.error("Verification failed:", e);
  }
}
```

### Bare WASM (No JS Bindings)

If you are writing a smart contract (e.g. CosmWasm) or don't want the JS wrappers:

```bash
cargo build --target wasm32-unknown-unknown --release
```

---

## 🔒 Cryptographic Correctness & Formal Security

Cryptography is notoriously unforgiving of edge cases. To prove beyond any doubt that `kyn-vdf` is mathematically sound, we employ a rigorous 3-pillar validation strategy:

### 1. Differential Cross-Testing against Chia C++ (100% Match)
We do not rely on assumptions. The `kyn-vdf` repository includes an automated differential testing suite (`scripts/differential_test.py`) that strictly cross-validates our pure Rust engine against the official Chia Network C++ `chiavdf` engine. 

The test suite enforces **100% bit-for-bit mathematical parity** and ensures:
- **Discriminant Derivation Parity**: Seeds produce the exact same 1024-bit primes.
- **Genuine Proof Validation**: 100-iteration and 100,000-iteration test vectors pass seamlessly.
- **Adversarial Tamper-Resistance**: Any bit flips in the BQFC wire format ($y$ or $\pi$), coefficient tampering, iteration mismatches ($T \pm 1$), or cross-challenge seed swaps are immediately mathematically rejected.
- **Zero Malleability**: The BQFC deserializer strictly enforces canonical zero-padding for generator/identity flags, defending against proof malleability attacks that plague naive implementations.

### 2. Property-Based Testing (Axiomatic Proofs)
Using `proptest`, we continuously fuzz the underlying Class Group arithmetic against randomized inputs. This guarantees that Shanks' NUCOMP and NUDUPL reduction algorithms strictly satisfy all **Abelian Group Axioms**:
- Associativity: $(A \circ B) \circ C == A \circ (B \circ C)$
- Identity: $A \circ 1 == A$
- Inverses: $A \circ A^{-1} == 1$
- Fast Exponentiation Parity: Binary scalar multiplication yields the same canonical reduced form as sequential compositions.

### 3. Continuous Integration & Fuzzing
Every commit is vetted through strict GitHub Actions CI pipelines:
- `cargo clippy -- -D warnings` (Strict linting & zero unhandled panics)
- `cargo-fuzz` (LibFuzzer targeting the BQFC deserialization engine for OOM/panic resistance)
- **Automated Differential Cross-Validator** (Running the live C++ reference engine in CI)

`kyn-vdf` is proudly written with **`#![forbid(unsafe_code)]`** — bringing total memory safety to VDF verification.


---

## Acknowledgements

This crate is an independent, clean-room Rust implementation, but the mathematical protocols, BQFC serialization logic, and challenge generation algorithms were originally designed and pioneered by **Chia Network**. Full credit for the underlying VDF protocol specification belongs to the original Chia researchers and engineers.

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
