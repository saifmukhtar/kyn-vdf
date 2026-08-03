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

## Security & Formal Testing

- **Property Testing:** `tests/proptest_math.rs` validates group axioms (identity, associativity, inverse, and exponentiation) using `proptest`.
- **Differential Fuzzing:** `fuzz/fuzz_targets/fuzz_vdf.rs` continuously fuzzes deserialization and verification against malformed/arbitrary byte streams:
  ```bash
  cargo +nightly fuzz run fuzz_vdf
  ```
- **Negative Testing:** Strict validation ensures zero false positives on flipped bits, incorrect iteration counts, or non-reduced forms.

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
