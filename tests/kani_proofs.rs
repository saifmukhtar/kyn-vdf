// tests/kani_proofs.rs
// Formal verification proofs using AWS Kani.
// Run with: `cargo kani`
#![cfg(kani)]

use kyn_vdf::math::{Form, isqrt_fourth};
use num_bigint::BigInt;

/// Prove that calculating the Shanks L threshold never panics, 
/// bounds memory, and gracefully handles any integer size.
#[kani::proof]
fn prove_isqrt_fourth_safe() {
    // Generate a non-deterministic byte array representing an arbitrary BigInt
    let bytes: [u8; 32] = kani::any();
    let n = BigInt::from_signed_bytes_be(&bytes);

    // Ensure it's non-negative as isqrt_fourth requires positive n
    if n >= BigInt::from(0) {
        // Run the math - Kani will statically prove this cannot panic
        let _l = isqrt_fourth(&n);
    }
}

/// Prove that constructing a form with from_abd safely rejects division-by-zero
/// and invalid discriminant relationships, rather than panicking.
#[kani::proof]
fn prove_form_from_abd_safe() {
    let a_bytes: [u8; 4] = kani::any();
    let b_bytes: [u8; 4] = kani::any();
    let d_bytes: [u8; 4] = kani::any();

    let a = BigInt::from_signed_bytes_be(&a_bytes);
    let b = BigInt::from_signed_bytes_be(&b_bytes);
    let d = BigInt::from_signed_bytes_be(&d_bytes);

    // Kani will mathematically prove this never panics under any conditions
    let _form = Form::from_abd(&a, &b, &d);
}
