# Security & Cryptographic Audit

`kyn-vdf` is a production-ready, pure-Rust Verifiable Delay Function (VDF) verifier. It implements Shanks' sub-quadratic algorithms (NUCOMP/NUDUPL) for binary quadratic form arithmetic and verifies Chia-compatible Wesolowski VDF proofs.

## Production Readiness
As of version `1.0.0`, `kyn-vdf` is classified as **Production Ready**.

The architecture specifically mitigates class group edge-cases that plague experimental implementations:
- **No Unsafe Code**: The codebase uses `#![forbid(unsafe_code)]` to guarantee memory safety.
- **Panic-Free Bounds Checking**: Class group reductions strictly bound coefficient growth, ensuring the exponentiation math mathematically cannot panic or trigger OOMs.
- **Zero Malleability**: The BQFC deserializer strictly enforces canonical zero-padding for generator/identity flags, eliminating proof malleability attacks.

## Mathematical Validation
This library is not experimental. It undergoes continuous mathematical verification:
1. **Differential Cross-Testing**: An automated pipeline (via `scripts/differential_test.py`) strictly cross-validates this pure-Rust engine against the official Chia Network C++ reference implementation (`chiavdf`). It asserts **100% bit-for-bit mathematical parity** for Discriminant Derivation, proof squarings, and all boundary test vectors (including 100,000-iteration benchmarks).
2. **Abelian Group Axiom Fuzzing**: Uses `proptest` to fuzz Shanks' reduction algorithms against randomized class group elements, proving adherence to associativity, inverses, and identity axioms.
3. **Formal Verification (AWS Kani)**: Implements bit-precise formal model-checking using AWS Kani to mathematically prove the absence of integer overflows and division-by-zero panics under fully non-deterministic arbitrary inputs.
4. **Miri UB Analysis**: Continuous Integration runs the test suite through the Miri byte-level memory interpreter, proving the complete absence of Undefined Behavior.

## Reporting Vulnerabilities
If you discover a mathematical edge case, a bypass in the Wesolowski verifier, or a wire-format malleability issue, please open an issue in this repository. 
