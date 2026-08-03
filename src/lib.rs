//! # kyn-vdf
//!
//! A pure Rust, WebAssembly-compatible implementation of Wesolowski Verifiable Delay Function (VDF)
//! verification over Imaginary Quadratic Class Groups.
//!
//! ## Key Features
//! - **Pure Rust / Zero FFI**: No C/C++ compiler, `libgmp`, or OS dependencies required.
//! - **WebAssembly Native**: Compiles directly to `wasm32-unknown-unknown` for in-browser and mobile light client verification.
//! - **Shanks' NUCOMP / NUDUPL**: Sub-quadratic binary quadratic form composition and squaring with partial Euclidean reduction.
//! - **Chia-Compatible**: 100% test-vector compatible with the Chia Network reference VDF specification.
//! - **Fast & Constant Verification**: Asymptotically $\mathcal{O}(\log T)$ verification time regardless of iteration count.
//!
//! ## Quick Start
//! ```rust
//! use kyn_vdf::verify_chia_vdf;
//!
//! # fn example(challenge: &[u8], proof_bytes: &[u8]) -> Result<(), kyn_vdf::KynVdfError> {
//! let is_valid = verify_chia_vdf(challenge, proof_bytes, 100_000, 1024)?;
//! assert!(is_valid);
//! # Ok(())
//! # }
//! ```

pub mod chia;
pub mod error;
pub mod math;

pub use chia::{
    create_discriminant, deserialize_form, get_b, hash_prime, is_probable_prime, serialize_form,
    verify_wesolowski, CompressedForm,
};
pub use error::KynVdfError;
pub use math::{isqrt_fourth, xgcd_partial, Form};

/// Verifies a Chia-compatible Wesolowski VDF proof from raw byte slices.
///
/// # Parameters
/// * `challenge_seed` - The challenge byte slice (typically 32 bytes).
/// * `proof_bytes` - The serialized proof bytes (either 200 bytes containing `y || π`, or 100 bytes of `π`).
/// * `iterations` - Number of sequential squarings evaluated.
/// * `discriminant_size_bits` - Size of the generated class group discriminant in bits (e.g. 1024).
///
/// # Returns
/// Returns `Ok(true)` if the proof is mathematically valid, `Ok(false)` if the proof is rejected,
/// or `Err(KynVdfError)` if inputs are malformed.
pub fn verify_chia_vdf(
    challenge_seed: &[u8],
    proof_bytes: &[u8],
    iterations: u64,
    discriminant_size_bits: usize,
) -> Result<bool, KynVdfError> {
    if iterations == 0 {
        return Err(KynVdfError::InvalidIterations(0));
    }
    if proof_bytes.len() < 200 {
        return Err(KynVdfError::InvalidProofLength {
            expected: 200,
            actual: proof_bytes.len(),
        });
    }

    let d = create_discriminant(challenge_seed, discriminant_size_bits);
    let x = Form::generator(&d).ok_or(KynVdfError::InvalidDiscriminantIdentity)?;

    let y_form = deserialize_form(&d, &proof_bytes[0..100])?;
    let proof_form = deserialize_form(&d, &proof_bytes[100..200])?;

    verify_wesolowski(&d, &x, &y_form, &proof_form, iterations)
}

/// Pure Rust VDF Verifier struct.
#[derive(Debug, Clone, Default)]
pub struct KynVdfVerifier {
    discriminant_size_bits: usize,
}

impl KynVdfVerifier {
    /// Creates a new `KynVdfVerifier` with the default 1024-bit discriminant.
    pub fn new() -> Self {
        Self {
            discriminant_size_bits: 1024,
        }
    }

    /// Creates a new `KynVdfVerifier` with custom discriminant bit-size.
    pub fn with_bits(discriminant_size_bits: usize) -> Self {
        Self {
            discriminant_size_bits,
        }
    }

    /// Verifies a Wesolowski proof.
    pub fn verify(&self, challenge: &[u8], proof_bytes: &[u8], iterations: u64) -> Result<bool, KynVdfError> {
        verify_chia_vdf(challenge, proof_bytes, iterations, self.discriminant_size_bits)
    }
}
