# kyn-vdf: Performance Benchmarks

The `kyn-vdf` crate is a pure Rust, zero-dependency implementation of Verifiable Delay Functions (VDF) utilizing Wesolowski proofs. 

> [!IMPORTANT]  
> **Verifier Only:** `kyn-vdf` is strictly engineered as a lightweight, client-side **verifier**. It evaluates pre-computed proofs and rapidly validates them. It does **not** generate proofs, as proof generation requires specialized, heavy infrastructure.

The following benchmarks demonstrate the extreme efficiency, stability, and scaling characteristics of `kyn-vdf`. By eliminating heavy C++ and `libgmp` dependencies, this library unlocks cryptographic verification across a multitude of environments—from in-browser WebAssembly modules to natively compiled UI frameworks (Tauri/Flutter) targeting bare-metal processors.

---

## Methodology & Environment

To establish a highly rigorous "reality check" representing real-world blockchain consensus protocols, we evaluated a **100,000-iteration test vector** ($T=100,000$). At this massive iteration count, the exponent wraps modulo the 264-bit Fiat-Shamir prime, forcing the verifier to execute the maximum theoretical mathematical work (~264 sequential squarings).

Every environment executed **1,000 continuous verification rounds** in an asynchronous tight loop. This exhaustive testing ensures that our average execution times are highly accurate, while simultaneously verifying the absence of memory leaks and thermal throttling under sustained load.

> [!NOTE]  
> **Strict Release Profiles:** All native and Foreign Function Interface (FFI) benchmarks were strictly compiled using the Rust `--release` profile. The immense speeds showcased below are the direct result of Rust's zero-cost abstractions and aggressive LLVM optimizations (`opt-level = 3`).

### Hardware Specifications

**Desktop Workstation:**
- **CPU:** 11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz
- **RAM:** 16GB
- **OS:** Linux (x86_64)

**Mobile Device:**
- **Device:** Nothing A059P (Board: `volcano`)
- **CPU:** Qualcomm SM7635 (Snapdragon) Octa-Core
  - 1x Prime Core @ 2.49 GHz
  - 3x Performance Cores @ 2.40 GHz
  - 4x Efficiency Cores @ 1.80 GHz
- **RAM:** 8GB
- **OS:** Android (ARM64)

---

## Executive Summary

The table below highlights the average time required to fully verify a massive 100,000-iteration proof. 

| Platform | Execution Environment | Technology | Average Time ($T=100k$) |
| :--- | :--- | :--- | :--- |
| **Desktop (Intel i5)** | CLI Terminal | Pure Native (x86_64) | **128.61 ms** 🏆 |
| **Desktop (Intel i5)** | Tauri App | Native FFI (Rust) | **132.28 ms** |
| **Desktop (Intel i5)** | Chromium | WebAssembly Sandbox | **381.56 ms** |
| **Desktop (Intel i5)** | Firefox | WebAssembly Sandbox | **442.30 ms** |
| **Mobile (Snapdragon)**| Termux / ADB | Pure Native (ARM64) | **296.34 ms** 🚀 |
| **Mobile (Snapdragon)**| Flutter App | Native FFI (Dart) | **302.73 ms** |
| **Mobile (Snapdragon)**| Chrome Android | WebAssembly Sandbox | **568.76 ms** |

---

## Analysis & Architectural Takeaways

### 1. The Dominance of Native Compilation
The benchmarks vividly illustrate the staggering performance gap between the sandboxed WebAssembly execution and bare-metal native compilation. While the WebAssembly implementation provides incredibly accessible, sub-second verification natively inside any web browser, leveraging `kyn-vdf` as a native binary yields a dramatic **~3x speed multiplier on desktop** (128ms vs 381ms) and nearly a **2x multiplier on mobile** (296ms vs 568ms). 

Through pure Rust optimizations, a modern smartphone can verify massive cryptographic proofs significantly faster than a desktop workstation constrained by a browser sandbox.

### 2. Frictionless FFI Integrations
When deploying cross-platform applications, the communication bridge between the UI framework and the core cryptography library is often a major bottleneck. However, the data proves that `kyn-vdf` incurs virtually zero penalty:
- **Tauri (Desktop):** ~4 ms overhead (132.28 ms vs 128.61 ms)
- **Flutter (Mobile):** ~6 ms overhead (302.73 ms vs 296.34 ms)

This validates that integrating `kyn-vdf` into Tauri desktop wrappers or Flutter mobile applications is the definitively optimal deployment strategy. You achieve raw, uncompromised cryptographic speed without sacrificing the rapid development cycles of modern UI frameworks.

### 3. Flat $\mathcal{O}(\log T)$ Scaling
Because Wesolowski VDF verification scales logarithmically, the maximum size of the exponent calculations caps out mathematically once $T \ge 264$. Therefore, whether a prover performs 100,000 iterations or 10,000,000 iterations, the verification time will remain entirely flat at the speeds recorded in these benchmarks. A user opening a mobile wallet will experience a virtually instant (~300ms) verification regardless of the underlying cryptographic difficulty.

---

## 📊 The Raw Data

*The raw terminal outputs are preserved below to guarantee the authenticity of the benchmarked averages.*

### 1. Native Execution (x86_64 & ARM64)
Executed directly on the processor via terminal, completely bypassing UI framework and browser overhead.

**Desktop Native (Intel i5):**
```text
Result: ✅ VALID
Total time: 128.61 seconds
Average time per verification: 128.61 ms
(Measured over 1000 runs natively)
```

**Mobile Native (Snapdragon via Termux):**
```text
Result: ✅ VALID
Total time: 296.34 seconds
Average time per verification: 296.34 ms
(Measured over 1000 runs natively)
```

### 2. Framework Integrations (Tauri & Flutter)
Testing real-world cross-platform overhead using Foreign Function Interfaces.

**Tauri App (Desktop):**
```text
Result: ✅ VALID
Total time: 132.28 seconds
Average time per verification: 132.28 ms
(Measured over 1000 runs natively)
```

**Flutter App (Mobile):**
```text
Result: ✅ VALID
Total time: 302.73 seconds
Average time per verification: 302.73 ms
(Measured over 1000 runs natively)
```

### 3. WebAssembly Sandboxing
Compiled via `wasm-pack build --target web --release`.

**Chromium (Desktop):**
```text
Result: ✅ VALID
Total time: 381.56 seconds
Average time per verification: 381.56 ms
(Measured over 1000 runs)
```

**Firefox (Desktop):**
```text
Result: ✅ VALID
Total time: 442.30 seconds
Average time per verification: 442.30 ms
(Measured over 1000 runs)
```

**Mobile Chrome (Android):**
```text
Result: ✅ VALID
Total time: 568.76 seconds
Average time per verification: 568.76 ms
(Measured over 1000 runs)
```
