//! Error types for VDF verification and Class Group arithmetic.
//!
//! All public functions in `kyn-vdf` return [`KynVdfError`] on failure rather than
//! panicking, ensuring the library is safe for use in WASM and adversarial input
//! environments.

use thiserror::Error;

/// Domain-specific errors returned during Class Group operations and Wesolowski VDF verification.
///
/// Every variant carries a human-readable message describing the exact failure point.
/// Callers should treat [`KynVdfError::VerificationFailed`] as a clean "proof rejected"
/// signal, and all other variants as malformed or invalid inputs.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KynVdfError {
    /// The proof byte slice is shorter than the required minimum.
    ///
    /// Chia-format proofs must be exactly 200 bytes (`y || π`, each 100 bytes).
    #[error("Invalid proof length: expected {expected} bytes, got {actual}")]
    InvalidProofLength { expected: usize, actual: usize },

    /// The challenge seed passed to [`crate::create_discriminant`] was empty or invalid.
    ///
    /// Seeds must be non-empty byte slices. Longer seeds (≥32 bytes) are recommended
    /// for security.
    #[error("Invalid seed: {0}")]
    InvalidSeed(String),

    /// The discriminant bit-size is zero, not a multiple of 8, or otherwise unsupported.
    ///
    /// Valid values are multiples of 8 (e.g. `512`, `1024`, `2048`).
    #[error("Invalid discriminant size: {0} bits (must be a non-zero multiple of 8)")]
    InvalidDiscriminantSize(usize),

    /// Discriminant generation or prime hashing failed unexpectedly.
    #[error("Failed to generate discriminant: {0}")]
    DiscriminantError(String),

    /// A quadratic form could not be serialized or deserialized from the Chia BQFC format.
    ///
    /// This typically means the proof bytes are corrupted, truncated, or were produced
    /// by an incompatible implementation.
    #[error("Quadratic form (de)serialization failed: {0}")]
    FormDeserializationError(String),

    /// The deserialized or constructed quadratic form does not satisfy the fundamental
    /// discriminant identity $b^2 - 4ac = D$.
    ///
    /// Indicates a corrupted proof or an incorrect discriminant was used for verification.
    #[error("Invalid quadratic form: discriminant identity b² - 4ac = D does not hold")]
    InvalidDiscriminantIdentity,

    /// An extended GCD, modular inverse, or other class group arithmetic step failed.
    #[error("Class group arithmetic error: {0}")]
    ArithmeticError(String),

    /// The iteration count passed to verify is zero.
    ///
    /// A VDF with zero iterations is undefined — the minimum meaningful value is 1.
    #[error("Invalid iteration count: {0} (must be ≥ 1)")]
    InvalidIterations(u64),

    /// Verification completed without error but the proof equation does not hold:
    /// `π^B · x^r ≠ y`.
    ///
    /// This is the expected error when a valid but incorrect proof is presented.
    #[error("VDF proof rejected: π^B · x^r ≠ y (proof does not verify)")]
    VerificationFailed,
}
