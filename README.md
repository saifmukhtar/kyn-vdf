<div align="center">

# `kyn-vdf`

**A pure Rust, WebAssembly-native Wesolowski Verifiable Delay Function (VDF) verifier over Imaginary Quadratic Class Groups.**

[![CI Pipeline](https://github.com/saifmukhtar/kyn-vdf/actions/workflows/ci.yml/badge.svg)](https://github.com/saifmukhtar/kyn-vdf/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/kyn-vdf.svg)](https://crates.io/crates/kyn-vdf)
[![Documentation](https://docs.rs/kyn-vdf/badge.svg)](https://docs.rs/kyn-vdf)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![WASM Ready](https://img.shields.io/badge/wasm-ready-brightgreen.svg)](#webassembly-support)
[![Safety](https://img.shields.io/badge/unsafe-0%25-green.svg)](#cryptographic-correctness--security-validation)

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
- **`no_std` Consensus Profile:** Supports bare-metal and consensus runtime environments with `default-features = false` (requires `alloc`).
- **WASM Native:** Compiles cleanly to WebAssembly for web browsers, browser extensions, and mobile light clients via the dedicated `crates/kyn-vdf-wasm` adapter crate.
- **Shanks' NUCOMP & NUDUPL:** Sub-quadratic composition and squaring of binary quadratic forms with partial Euclidean reduction.
- **Canonical 200-Byte Wire Format:** Strictly enforces exact 200-byte wire envelopes with zero proof malleability (rejects flag-aliasing, dirty sign bits, trailing garbage, and unflagged special forms).
- **Asymptotic Verification ($\mathcal{O}(\log T)$):** Verification runs in logarithmic time (~130ms native, ~380ms in-browser WASM) regardless of whether proof generation required thousands or millions of sequential squarings.
- **Chia-Compatible:** Bit-for-bit differential parity against Chia Network's C++ reference implementation (`chiavdf`) across 1024-bit discriminants and test vectors up to 1,000,000 iterations.
- **`#![forbid(unsafe_code)]`:** 100% safe Rust with zero undefined behavior.

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
kyn-vdf = "1.0"
```

For `no_std` consensus runtimes (requires `alloc`):
```toml
[dependencies]
kyn-vdf = { version = "1.0", default-features = false }
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

# Build the JS/TS package from the browser adapter crate.
# --out-name keeps the existing kyn_vdf.js / kyn_vdf.wasm filenames.
wasm-pack build crates/kyn-vdf-wasm \
  --target web \
  --release \
  --out-name kyn_vdf \
  --out-dir pkg
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

### Smart-contract / runtime consumers (no JS bindings)

Smart-contract and consensus runtimes should link the `rlib` without default
features. The consuming runtime or contract produces the final Wasm artifact;
this crate does not emit a standalone `.wasm` in that profile.

```toml
[dependencies]
kyn-vdf = { version = "1.0", default-features = false }
```

---

## 🔒 Cryptographic Correctness & Security Validation

Cryptography is notoriously unforgiving of edge cases. To prove that `kyn-vdf` is mathematically sound, robust against adversarial inputs, and free from proof malleability, we employ a 4-pillar validation strategy:

### 1. Differential Cross-Testing against Chia C++ (100% Bit-for-Bit Parity)
We do not rely on assumptions. The `kyn-vdf` repository includes automated differential test suites (`scripts/differential_test.py` and `scripts/live_differential_test.py`) that strictly cross-validate our pure Rust engine against the official Chia Network C++ `chiavdf` engine.

The differential test suite enforces:
- **Discriminant Derivation Parity**: Challenge seeds produce the exact same 1024-bit prime discriminants.
- **Deep Proof Verification**: Verification of official Chia 100-iteration, 100,000-iteration, and 1,000,000-iteration reference test vectors.
- **Adversarial Tamper-Resistance**: Any bit flips in the BQFC wire format ($y$ or $\pi$), coefficient tampering, iteration mismatches ($T \pm 1$), or cross-challenge seed swaps are immediately mathematically rejected.

### 2. Strict Canonical Wire Format & Zero Malleability
In binary quadratic form serialization (BQFC), naive implementations often suffer from proof malleability (where multiple distinct wire byte arrays deserialize to the same mathematical form). `kyn-vdf` strictly enforces a bijective encoding:
- **Exact 200-Byte Envelope**: Enforces `proof_bytes.len() == 200`. Proofs with truncated or trailing garbage bytes are rejected.
- **Flag-Aliasing Defense**: Special-form flag byte `bytes[0]` must strictly identify exactly one form (`BQFC_IS_1` or `BQFC_IS_GEN`). Mixed flags, dirty sign bits, and undefined upper bits (`0xF0`) are rejected.
- **Bijective Special Form Encoding**: Identity and generator forms encoded without flags through the slow decompression path are rejected.
- **Padding Validation**: All trailing bytes in sub-1024-bit wire buffers and special-form envelopes must strictly be zero.

### 3. Property-Based Testing (Axiomatic Invariants)
Using `proptest`, we continuously fuzz the underlying Class Group arithmetic against randomized inputs. This guarantees that Shanks' NUCOMP and NUDUPL reduction algorithms strictly satisfy all **Abelian Group Axioms**:
- Associativity: $(A \circ B) \circ C == A \circ (B \circ C)$
- Identity: $A \circ 1 == A$
- Inverses: $A \circ A^{-1} == 1$
- Fast Exponentiation Parity: Binary scalar multiplication yields the same canonical reduced form as sequential compositions.

### 4. Memory Safety & Checked Fallibility
- **`#![forbid(unsafe_code)]`**: Zero `unsafe` blocks in the entire codebase, eliminating undefined memory behavior natively.
- **Zero Panics on Malformed Input**: All public verifier APIs return explicit `Result<_, KynVdfError>` instead of panicking on division-by-zero, buffer bounds, integer overflow, or malformed inputs.
- **CI Linting**: Enforced with `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check`.


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
