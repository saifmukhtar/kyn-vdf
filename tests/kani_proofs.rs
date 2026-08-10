// tests/kani_proofs.rs
// Formal verification proofs using AWS Kani.
// Run with: `cargo kani`
#![allow(unexpected_cfgs)]
#![cfg(kani)]

use kyn_vdf::math::isqrt_fourth;
use num_bigint::BigInt;

/// Prove that calculating the Shanks L threshold on fixed, realistic inputs
/// never panics and returns the correct floor fourth root.
#[kani::proof]
fn prove_isqrt_fourth_safe() {
    let cases: [(BigInt, BigInt); 4] = [
        (BigInt::from(0), BigInt::from(0)),
        (BigInt::from(1), BigInt::from(1)),
        (BigInt::from(16), BigInt::from(2)),
        (BigInt::from(10000), BigInt::from(10)),
    ];
    for (n, expected) in cases.into_iter() {
        let l = isqrt_fourth(&n);
        kani::assert(l == expected, "isqrt_fourth computes floor fourth root");
    }
}
