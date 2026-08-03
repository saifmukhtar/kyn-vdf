//! # kyn-vdf
//!
//! A pure Rust, WebAssembly-compatible implementation of Wesolowski Verifiable Delay Function
//! (VDF) verification over Imaginary Quadratic Class Groups.
//!
//! ## Key Features
//! - **Pure Rust / Zero FFI**: No C/C++ compiler, `libgmp`, or OS dependencies required.
//! - **WebAssembly Native**: Compiles to `wasm32-unknown-unknown` for in-browser and mobile
//!   light client verification.
//! - **Shanks' NUCOMP / NUDUPL**: Sub-quadratic binary quadratic form composition and squaring
//!   with partial Euclidean reduction.
//! - **Chia-Compatible**: 100% test-vector compatible with the Chia Network VDF specification.
//! - **$\mathcal{O}(\log T)$ Verification**: Verification time is constant with respect to $T$
//!   (bounded by the 264-bit Fiat-Shamir prime $B$, not the iteration count).
//! - **No Panics**: All fallible operations return `Result<_, KynVdfError>` — safe for WASM
//!   and adversarial inputs.
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
/// This is the primary entry point for most callers. It handles discriminant
/// generation, form deserialization, and the Wesolowski verification equation
/// in a single call.
///
/// # Parameters
/// - `challenge_seed`: The challenge byte slice used to derive the discriminant and
///   generator form. Typically 32 bytes (e.g. a block hash).
/// - `proof_bytes`: The serialized proof in Chia wire format — exactly 200 bytes
///   containing the VDF output `y` (bytes 0–99) concatenated with the proof `π`
///   (bytes 100–199), each in 100-byte BQFC format.
/// - `iterations`: Number of sequential squarings $T$ that were evaluated.
///   Must be ≥ 1.
/// - `discriminant_size_bits`: Bit-size of the class group discriminant (e.g. `1024`).
///   Must be a non-zero multiple of 8. Use `1024` unless you have a specific reason
///   to use a different size.
///
/// # Returns
/// - `Ok(true)` — proof is mathematically valid.
/// - `Ok(false)` — proof is rejected (valid inputs but incorrect proof).
/// - `Err(KynVdfError)` — inputs are malformed (wrong lengths, invalid seed, etc.).
///
/// # Errors
/// - [`KynVdfError::InvalidIterations`] if `iterations == 0`.
/// - [`KynVdfError::InvalidProofLength`] if `proof_bytes.len() < 200`.
/// - [`KynVdfError::InvalidDiscriminantSize`] if `discriminant_size_bits` is invalid.
/// - [`KynVdfError::FormDeserializationError`] if the proof bytes are corrupted.
/// - [`KynVdfError::InvalidDiscriminantIdentity`] if a form fails the discriminant check.
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

    let d = create_discriminant(challenge_seed, discriminant_size_bits)?;
    let x = Form::generator(&d).ok_or(KynVdfError::InvalidDiscriminantIdentity)?;

    let y_form = deserialize_form(&d, &proof_bytes[0..100])?;
    let proof_form = deserialize_form(&d, &proof_bytes[100..200])?;

    verify_wesolowski(&d, &x, &y_form, &proof_form, iterations)
}

/// Pure Rust Wesolowski VDF verifier.
///
/// A thin stateful wrapper around [`verify_chia_vdf`] that holds the discriminant
/// size so callers don't need to pass it on every verification call.
///
/// # Example
/// ```rust
/// use kyn_vdf::KynVdfVerifier;
///
/// # fn example(challenge: &[u8], proof: &[u8]) -> Result<(), kyn_vdf::KynVdfError> {
/// let verifier = KynVdfVerifier::new(); // 1024-bit discriminant
/// let is_valid = verifier.verify(challenge, proof, 100_000)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct KynVdfVerifier {
    /// Bit-size of the class group discriminant (default: 1024).
    discriminant_size_bits: usize,
}

impl KynVdfVerifier {
    /// Creates a new `KynVdfVerifier` with the standard 1024-bit discriminant.
    ///
    /// This matches the discriminant size used by the Chia Network mainnet VDF.
    pub fn new() -> Self {
        Self {
            discriminant_size_bits: 1024,
        }
    }

    /// Creates a new `KynVdfVerifier` with a custom discriminant bit-size.
    ///
    /// Use this when verifying proofs generated with a non-standard discriminant size.
    /// `discriminant_size_bits` must be a non-zero multiple of 8.
    pub fn with_bits(discriminant_size_bits: usize) -> Self {
        Self {
            discriminant_size_bits,
        }
    }

    /// Verifies a Wesolowski VDF proof.
    ///
    /// Delegates to [`verify_chia_vdf`] with this verifier's configured discriminant size.
    ///
    /// # Errors
    /// See [`verify_chia_vdf`] for the full list of possible errors.
    pub fn verify(
        &self,
        challenge: &[u8],
        proof_bytes: &[u8],
        iterations: u64,
    ) -> Result<bool, KynVdfError> {
        verify_chia_vdf(challenge, proof_bytes, iterations, self.discriminant_size_bits)
    }
}
