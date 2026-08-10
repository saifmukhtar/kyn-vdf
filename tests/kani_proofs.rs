// tests/kani_proofs.rs
// Formal verification proofs using AWS Kani.
// Run with: `cargo kani`
#![allow(unexpected_cfgs)]
#![cfg(kani)]

use kyn_vdf::math::{Form, isqrt_fourth};
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

/// Prove that constructing a form with from_abd on fixed, realistic inputs
/// returns the exact reduced form for valid inputs and `None` for invalid
/// ones, rather than panicking.
#[kani::proof]
fn prove_form_from_abd_safe() {
    let d = BigInt::from(-71);

    // Valid forms: (1, 1, 18) and (2, 1, 9) both satisfy b^2 - 4ac = D.
    let f1 = Form::from_abd(&BigInt::from(1), &BigInt::from(1), &d);
    kani::assert(f1.is_some(), "a=1, b=1, D=-71 is valid");
    if let Some(f) = f1 {
        kani::assert(f.a == BigInt::from(1), "a is 1");
        kani::assert(f.b == BigInt::from(1), "b is 1");
        kani::assert(f.c == BigInt::from(18), "c = (b^2 - D) / 4a = 18");
    }

    let f2 = Form::from_abd(&BigInt::from(2), &BigInt::from(1), &d);
    kani::assert(f2.is_some(), "a=2, b=1, D=-71 is valid");
    if let Some(f) = f2 {
        kani::assert(f.a == BigInt::from(2), "a is 2");
        kani::assert(f.c == BigInt::from(9), "c = (b^2 - D) / 4a = 9");
    }

    // Invalid: a = 0 is degenerate, and (b^2 - D) = 75 is not divisible by 8.
    let bad1 = Form::from_abd(&BigInt::from(0), &BigInt::from(1), &d);
    kani::assert(bad1.is_none(), "a = 0 is rejected");

    let bad2 = Form::from_abd(&BigInt::from(2), &BigInt::from(2), &d);
    kani::assert(bad2.is_none(), "non-divisible (b^2 - D) is rejected");
}
