//! Error types for VDF verification and Class Group arithmetic.

use thiserror::Error;

/// Domain-specific errors returned during Class Group operations and Wesolowski VDF verification.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KynVdfError {
    /// The proof byte length is invalid.
    #[error("Invalid proof length: expected at least {expected} bytes, got {actual}")]
    InvalidProofLength { expected: usize, actual: usize },

    /// The discriminant generation failed or input seed was malformed.
    #[error("Failed to generate discriminant: {0}")]
    DiscriminantError(String),

    /// Quadratic form serialization or deserialization failed.
    #[error("Quadratic form deserialization failed: {0}")]
    FormDeserializationError(String),

    /// The quadratic form does not satisfy the discriminant identity (b^2 - 4ac = D).
    #[error("Invalid quadratic form: discriminant mismatch (b^2 - 4ac != D)")]
    InvalidDiscriminantIdentity,

    /// Extended GCD or arithmetic inversion failed.
    #[error("Class group arithmetic error: {0}")]
    ArithmeticError(String),

    /// Challenge iteration count is zero or exceeds permitted bounds.
    #[error("Invalid iteration count: {0}")]
    InvalidIterations(u64),

    /// Verification was computed but proof is mathematically invalid.
    #[error("VDF proof verification failed: output identity does not hold")]
    VerificationFailed,
}
