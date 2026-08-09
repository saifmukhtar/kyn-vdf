//! Chia-compatible discriminant generation, BQFC form serialization, and Wesolowski
//! VDF verification.
//!
//! This module implements:
//! - [`create_discriminant`]: deterministic 1024-bit negative prime discriminant from a seed.
//! - [`hash_prime`]: Chia's `HashPrime` algorithm (iterated SHA-256 with Miller-Rabin).
//! - [`serialize_form`] / [`deserialize_form`]: Chia's Binary Quadratic Form Compression (BQFC).
//! - [`verify_wesolowski`]: the Wesolowski VDF verification equation $\pi^B \cdot x^r = y$.
//!
//! All fallible operations return [`KynVdfError`] rather than panicking, ensuring safe
//! execution in WASM and adversarial input environments.

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use sha2::{Digest, Sha256};

use crate::error::KynVdfError;
use crate::math::Form;

/// Size of the Fiat-Shamir prime $B$ in bits.
///
/// $B$ is a 264-bit prime derived from the serialized forms $x$ and $y$ via [`hash_prime`].
/// The choice of 264 bits provides ~128-bit security for the Wesolowski soundness argument.
const B_BITS: usize = 264;

/// Size in bytes of a single serialized binary quadratic form (Chia BQFC format).
const BQFC_FORM_SIZE: usize = 100;

// --- BQFC flag bits in the first byte of a serialized form ---

/// Bit 0 of byte 0: set when the `b` coefficient of the original form is negative.
const BQFC_B_SIGN: u8 = 1 << 0;
/// Bit 1 of byte 0: set when the partial-XGCD quotient `t` is negative.
const BQFC_T_SIGN: u8 = 1 << 1;
/// Bit 2 of byte 0: set when the form is the principal identity (a=1, b=1).
const BQFC_IS_1: u8 = 1 << 2;
/// Bit 3 of byte 0: set when the form is the canonical generator (a=2, b=1).
const BQFC_IS_GEN: u8 = 1 << 3;

/// Computes the integer square root $\lfloor \sqrt{n} \rfloor$ for a non-negative `BigInt`.
///
/// # Errors
/// Returns [`KynVdfError::ArithmeticError`] if `n` is negative, which indicates a
/// programming error in the caller (reduced form coefficients `a` and `c` must be positive).
fn isqrt(n: &BigInt) -> Result<BigInt, KynVdfError> {
    if n.is_negative() {
        return Err(KynVdfError::ArithmeticError(
            "cannot compute integer square root of a negative number; \
             form coefficient 'a' must be positive for a valid reduced form"
                .to_string(),
        ));
    }
    if n.is_zero() {
        return Ok(BigInt::zero());
    }
    let uint_sqrt = n.to_biguint().unwrap().sqrt();
    Ok(BigInt::from_biguint(Sign::Plus, uint_sqrt))
}

/// Miller-Rabin probabilistic primality test.
///
/// Tests `n` using 12 deterministic small prime bases (2, 3, 5, …, 37) followed by
/// sequential odd witness bases up to `rounds`. For SHA-256 hash-derived candidate primes,
/// performing $k = 25$ rounds heuristically bounds the composite false-positive probability
/// near $4^{-25} \approx 2^{-50}$.
///
/// # Parameters
/// - `n`: The candidate integer to test (as `BigUint`).
/// - `rounds`: Minimum number of Miller-Rabin witness rounds to perform.
///
/// # Returns
/// `true` if `n` is probably prime; `false` if `n` is definitely composite.
pub fn is_probable_prime(n: &BigUint, rounds: usize) -> bool {
    if n < &BigUint::from(2u32) {
        return false;
    }
    if n == &BigUint::from(2u32) || n == &BigUint::from(3u32) {
        return true;
    }
    if n.is_even() {
        return false;
    }

    // Fast elimination via small prime trial division
    let small_primes = [3u32, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
    for &p in &small_primes {
        let bp = BigUint::from(p);
        if n == &bp {
            return true;
        }
        if (n % &bp).is_zero() {
            return false;
        }
    }

    // Factor out powers of 2: write n - 1 = 2^r * d
    let n_minus_1 = n - BigUint::one();
    let mut d = n_minus_1.clone();
    let mut r = 0usize;
    while d.is_even() {
        d >>= 1;
        r += 1;
    }

    // Deterministic bases sufficient for numbers up to ~3.3 × 10^24; extend with more rounds
    let bases = [2u32, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    let test_rounds = std::cmp::max(rounds, bases.len());

    'outer: for i in 0..test_rounds {
        let a = if i < bases.len() {
            BigUint::from(bases[i])
        } else {
            BigUint::from((i as u32) * 2 + 39)
        };
        if &a >= n {
            break;
        }

        let mut x = a.modpow(&d, n);
        if x.is_one() || x == n_minus_1 {
            continue;
        }

        for _ in 0..(r - 1) {
            x = x.modpow(&BigUint::from(2u32), n);
            if x == n_minus_1 {
                continue 'outer;
            }
        }

        return false; // Composite witness found
    }

    true
}

/// Generates a pseudoprime matching Chia's `HashPrime` algorithm.
///
/// Produces a prime of exactly `length_bits` bits by:
/// 1. Incrementing `seed` byte-by-byte (big-endian counter) and hashing with SHA-256.
/// 2. Concatenating hash outputs until `length_bits / 8` bytes are collected.
/// 3. Setting specific bits from `bitmask` and forcing the lowest bit to 1 (odd).
/// 4. Testing with Miller-Rabin (25 rounds); looping until a prime is found.
///
/// This is a deterministic function: the same seed always produces the same prime.
///
/// # Parameters
/// - `seed`: Non-empty byte slice used as the initial hash input.
/// - `length_bits`: Desired bit-length of the output prime. **Must be a non-zero multiple of 8.**
/// - `bitmask`: Indices of bits to force to 1 in the candidate (used to set the MSB and
///   ensure the prime has the required congruence properties).
///
/// # Errors
/// Returns [`KynVdfError::InvalidDiscriminantSize`] if `length_bits` is zero or not a
/// multiple of 8.
pub fn hash_prime(
    seed: &[u8],
    length_bits: usize,
    bitmask: &[usize],
) -> Result<BigUint, KynVdfError> {
    if length_bits == 0 || !length_bits.is_multiple_of(8) {
        return Err(KynVdfError::InvalidDiscriminantSize(length_bits));
    }

    let mut sprout = seed.to_vec();

    loop {
        let mut blob = Vec::new();
        while blob.len() * 8 < length_bits {
            // Increment the counter (big-endian) before each hash
            for i in (0..sprout.len()).rev() {
                sprout[i] = sprout[i].wrapping_add(1);
                if sprout[i] != 0 {
                    break;
                }
            }
            let hash = Sha256::digest(&sprout);
            let needed = (length_bits / 8) - blob.len();
            let take = std::cmp::min(hash.len(), needed);
            blob.extend_from_slice(&hash[..take]);
        }

        let mut p = BigUint::from_bytes_be(&blob);
        // Set required bits (MSB, congruence constraints)
        for &b in bitmask {
            p.set_bit(b as u64, true);
        }
        // Force odd — even numbers cannot be prime (except 2)
        p.set_bit(0, true);

        if is_probable_prime(&p, 25) {
            return Ok(p);
        }
        // Not prime: loop with the incremented sprout to try the next candidate
    }
}

/// Creates a negative fundamental prime discriminant $D = -p$ from a seed byte slice.
///
/// Matches Chia's `CreateDiscriminant` function exactly, producing a 1024-bit (or
/// other size) negative prime $p \equiv 7 \pmod 8$, returned as $D = -p$.
///
/// The discriminant is deterministic: the same `seed` and `length_bits` always
/// produce the same $D$.
///
/// # Parameters
/// - `seed`: Non-empty byte slice (use ≥ 32 bytes for security).
/// - `length_bits`: Bit-length of the discriminant. Must be a non-zero multiple of 8.
///   Typical values: `512`, `1024`, `2048`.
///
/// # Errors
/// - [`KynVdfError::InvalidSeed`] if `seed` is empty.
/// - [`KynVdfError::InvalidDiscriminantSize`] if `length_bits` is zero or not a multiple of 8.
///
/// # Example
/// ```rust
/// use kyn_vdf::create_discriminant;
///
/// let d = create_discriminant(&[0x42u8; 32], 1024).expect("valid seed");
/// assert!(d < num_bigint::BigInt::from(0)); // D is always negative
/// ```
pub fn create_discriminant(seed: &[u8], length_bits: usize) -> Result<BigInt, KynVdfError> {
    if seed.is_empty() {
        return Err(KynVdfError::InvalidSeed(
            "seed must be a non-empty byte slice".to_string(),
        ));
    }
    if length_bits == 0 || !length_bits.is_multiple_of(8) {
        return Err(KynVdfError::InvalidDiscriminantSize(length_bits));
    }

    let p = hash_prime(seed, length_bits, &[0, 1, 2, length_bits - 1])?;
    Ok(-BigInt::from_biguint(Sign::Plus, p))
}

/// Performs partial Extended Euclidean Algorithm for BQFC form compression.
///
/// Identical in semantics to [`crate::math::xgcd_partial`] but used internally
/// within the BQFC compression pipeline on `BigInt` values.
fn xgcd_partial_chia(a: &BigInt, b: &BigInt, l: &BigInt) -> (BigInt, BigInt, BigInt, BigInt) {
    let mut r2 = a.clone();
    let mut r1 = b.clone();
    let mut co2 = BigInt::zero();
    let mut co1 = BigInt::from(-1);

    while r1 > BigInt::zero() && &r1 > l {
        let q = &r2 / &r1;
        let t1 = &r2 - &q * &r1;
        let t2 = &co2 - &q * &co1;
        r2 = r1;
        r1 = t1;
        co2 = co1;
        co1 = t2;
    }
    (co2, co1, r2, r1)
}

/// Compressed representation of a binary quadratic form $(a, b)$ in Chia's BQFC format.
///
/// BQFC (Binary Quadratic Form Compression) reduces the storage of a 1024-bit form
/// from 128+ bytes to exactly 100 bytes by encoding the partial XGCD decomposition of
/// $(a, b)$ rather than storing the coefficients directly.
#[derive(Debug, Clone)]
pub struct CompressedForm {
    /// Compressed form of the `a` coefficient (divided by `g` if `g > 1`).
    pub a: BigInt,
    /// Partial XGCD quotient $t$ such that $t \cdot b \equiv \pm\sqrt{D} \pmod{a}$.
    pub t: BigInt,
    /// Common divisor $g = \gcd(a, t)$; equal to 1 when no further factoring is needed.
    pub g: BigInt,
    /// High-order correction term $b_0 = b / a'$ (non-zero only when `g > 1`).
    pub b0: BigInt,
    /// Sign of the original `b` coefficient (`true` = negative).
    pub b_sign: bool,
}

/// Compresses a reduced binary quadratic form $(a, b)$ into Chia's BQFC components.
///
/// This is the inverse of [`bqfc_decompr`]. The compressed form can be serialized
/// to exactly 100 bytes by [`serialize_form`].
///
/// # Errors
/// Returns [`KynVdfError::ArithmeticError`] if `a` is negative (which would indicate
/// a non-reduced or invalid input form).
pub fn bqfc_compr(a: &BigInt, b: &BigInt) -> Result<CompressedForm, KynVdfError> {
    if a == b {
        return Ok(CompressedForm {
            a: a.clone(),
            t: BigInt::zero(),
            g: BigInt::zero(),
            b0: BigInt::zero(),
            b_sign: false,
        });
    }

    let sign = b.is_negative();
    let a_sqrt = isqrt(a)?; // a must be positive for a valid reduced form
    let a_copy = a.clone();
    let b_copy = if sign { -b } else { b.clone() };

    let (_dummy, mut t, _r2, _r1) = xgcd_partial_chia(&a_copy, &b_copy, &a_sqrt);
    t = -t;

    let g = a.gcd(&t);
    let (out_a, out_t, mut out_b0) = if g == BigInt::one() {
        (a.clone(), t, BigInt::zero())
    } else {
        let out_a = a / &g;
        let out_t = &t / &g;
        let b0 = b / &out_a;
        (out_a, out_t, b0)
    };

    if sign {
        out_b0 = -out_b0;
    }

    Ok(CompressedForm {
        a: out_a,
        t: out_t,
        g,
        b0: out_b0,
        b_sign: sign,
    })
}

/// Decompresses Chia BQFC components back into the original $(a, b)$ coefficients.
///
/// This reconstructs $b$ from the partial XGCD decomposition stored in the
/// [`CompressedForm`], using modular inversion and a square-root recovery step.
///
/// # Errors
/// - [`KynVdfError::FormDeserializationError`] if:
///   - The compressed `a` coefficient is zero (malformed input).
///   - `t` and `a` are not coprime (modular inverse does not exist).
///   - The discriminant residue $t^2 \cdot D \bmod a$ is not a perfect square
///     (proof bytes are corrupted or tampered).
pub fn bqfc_decompr(c: &CompressedForm, d: &BigInt) -> Result<(BigInt, BigInt), KynVdfError> {
    // Special case: a == b (identity or generator detection)
    if c.t.is_zero() {
        return Ok((c.a.clone(), c.a.clone()));
    }

    if c.a.is_zero() {
        return Err(KynVdfError::FormDeserializationError(
            "compressed form has zero 'a' coefficient; the proof bytes are malformed".to_string(),
        ));
    }

    let mut t = c.t.clone();
    if t.is_negative() {
        t += &c.a;
    }

    // Compute modular inverse of t modulo a via extended GCD
    let ext = t.extended_gcd(&c.a);
    if ext.gcd != BigInt::one() {
        return Err(KynVdfError::FormDeserializationError(format!(
            "partial quotient 't' (= {}) is not coprime with 'a' (= {}); \
             the BQFC decompression inverse does not exist — proof bytes are corrupted",
            c.t, c.a
        )));
    }
    let mut t_inv = ext.x;
    if t_inv.is_negative() {
        t_inv += &c.a;
    }

    // Recover b from the quadratic residue: b ≡ sqrt(t² · D) · t⁻¹ (mod a)
    let d_mod_a = d.mod_floor(&c.a);
    let t_sq = (&c.t * &c.t).mod_floor(&c.a);
    let tmp = (t_sq * d_mod_a).mod_floor(&c.a);

    let root = isqrt(&tmp)?;
    if &root * &root != tmp {
        return Err(KynVdfError::FormDeserializationError(
            "discriminant residue t²·D mod a is not a perfect square; \
             the form cannot be reconstructed — proof bytes are corrupted or tampered"
                .to_string(),
        ));
    }

    let mut out_b = (&root * &t_inv).mod_floor(&c.a);
    let out_a = if c.g > BigInt::one() {
        &c.a * &c.g
    } else {
        c.a.clone()
    };

    if c.b0 > BigInt::zero() {
        out_b += &c.a * &c.b0;
    }

    if c.b_sign {
        out_b = -out_b;
    }

    Ok((out_a, out_b))
}

/// Writes `val` as little-endian bytes into `out_str[offset..offset+size]` with zero-padding.
///
/// Matches the byte layout of Chia's `bqfc.c` export routine.
///
/// # Errors
/// Returns [`KynVdfError::FormDeserializationError`] if `val` requires more than `size` bytes.
fn export_le(
    val: &BigInt,
    out_str: &mut [u8],
    offset: &mut usize,
    size: usize,
) -> Result<(), KynVdfError> {
    let bytes = val.to_biguint().unwrap_or_else(BigUint::zero).to_bytes_le();
    if bytes.len() > size {
        return Err(KynVdfError::FormDeserializationError(format!(
            "integer value requires {} bytes but only {} bytes are available in the BQFC slot; \
             the form coefficient is too large for the given discriminant size",
            bytes.len(),
            size
        )));
    }
    out_str[*offset..*offset + bytes.len()].copy_from_slice(&bytes);
    out_str[*offset + bytes.len()..*offset + size].fill(0);
    *offset += size;
    Ok(())
}

/// Reads a little-endian `BigInt` from a byte slice (positive, matching Chia's `bqfc.c`).
fn import_le(data: &[u8]) -> BigInt {
    BigInt::from_biguint(Sign::Plus, BigUint::from_bytes_le(data))
}

/// Serializes a reduced binary quadratic form into Chia's 100-byte BQFC wire format.
///
/// The output is always exactly [`BQFC_FORM_SIZE`] (100) bytes:
/// - **Byte 0**: Flag byte (`BQFC_B_SIGN`, `BQFC_T_SIGN`, `BQFC_IS_1`, `BQFC_IS_GEN`).
/// - **Byte 1**: `g_size` — the byte-length of the `g` coefficient minus 1.
/// - **Bytes 2…end**: Little-endian packed fields `(a, t, g, b0)`.
///
/// Special cases (identity and generator) are encoded with a single flag byte.
///
/// # Parameters
/// - `form`: The reduced binary quadratic form to serialize.
/// - `d_bits`: The bit-size of the discriminant (e.g. `1024`).
///
/// # Errors
/// Returns [`KynVdfError::FormDeserializationError`] if any coefficient overflows its
/// allocated slot (which would indicate a form with a mismatched discriminant size).
pub fn serialize_form(form: &Form, d_bits: usize) -> Result<Vec<u8>, KynVdfError> {
    let mut res = vec![0u8; BQFC_FORM_SIZE];

    // Fast path: identity (a=1, b=1) and generator (a=2, b=1) use a single flag byte
    if form.b == BigInt::one() && form.a <= BigInt::from(2) {
        res[0] = if form.a == BigInt::from(2) {
            BQFC_IS_GEN
        } else {
            BQFC_IS_1
        };
        return Ok(res);
    }

    let d_bits_rounded = (d_bits + 31) & !31;
    let compr = bqfc_compr(&form.a, &form.b)?;

    // Encode sign flags
    res[0] = if compr.b_sign { BQFC_B_SIGN } else { 0 };
    if compr.t.is_negative() {
        res[0] |= BQFC_T_SIGN;
    }

    // Compute field widths from discriminant size
    let g_biguint = compr.g.to_biguint().unwrap_or_else(BigUint::zero);
    let g_size = if compr.g.is_zero() {
        0
    } else {
        (g_biguint.bits() as usize).div_ceil(8) - 1
    };
    res[1] = g_size as u8;

    let mut offset = 2;
    let a_bytes_len = d_bits_rounded / 16 - g_size;
    let t_bytes_len = d_bits_rounded / 32 - g_size;
    let g_bytes_len = g_size + 1;

    export_le(&compr.a, &mut res, &mut offset, a_bytes_len)?;
    let t_abs = compr.t.abs();
    export_le(&t_abs, &mut res, &mut offset, t_bytes_len)?;
    export_le(&compr.g, &mut res, &mut offset, g_bytes_len)?;
    let b0_abs = compr.b0.abs();
    export_le(&b0_abs, &mut res, &mut offset, g_bytes_len)?;

    Ok(res)
}

/// Deserializes a Chia 100-byte BQFC-compressed form into a reduced [`Form`].
///
/// Performs full validation after decompression:
/// - Checks that the decompressed $(a, b)$ satisfy the discriminant identity $b^2 - 4ac = D$.
/// - Checks that the resulting form is in reduced normal form.
///
/// # Parameters
/// - `d`: The negative fundamental discriminant used to verify the form.
/// - `bytes`: Exactly [`BQFC_FORM_SIZE`] (100) bytes in Chia BQFC wire format.
///
/// # Errors
/// - [`KynVdfError::InvalidProofLength`] if `bytes.len() != 100`.
/// - [`KynVdfError::FormDeserializationError`] for any structural corruption.
/// - [`KynVdfError::InvalidDiscriminantIdentity`] if the form does not satisfy $b^2 - 4ac = D$.
pub fn deserialize_form(d: &BigInt, bytes: &[u8]) -> Result<Form, KynVdfError> {
    if bytes.len() != BQFC_FORM_SIZE {
        return Err(KynVdfError::InvalidProofLength {
            expected: BQFC_FORM_SIZE,
            actual: bytes.len(),
        });
    }

    // Fast path: identity and generator flags bypass full decompression
    if bytes[0] & (BQFC_IS_1 | BQFC_IS_GEN) != 0 {
        // Enforce canonical wire format: trailing padding bytes must be zero to prevent proof malleability
        if bytes[1..].iter().any(|&b| b != 0) {
            return Err(KynVdfError::FormDeserializationError(
                "Non-canonical wire format: trailing padding bytes in identity/generator form must be zero"
                    .to_string(),
            ));
        }
        let a = if bytes[0] & BQFC_IS_GEN != 0 {
            BigInt::from(2)
        } else {
            BigInt::from(1)
        };
        let b = BigInt::one();
        return Form::from_abd(&a, &b, d).ok_or(KynVdfError::InvalidDiscriminantIdentity);
    }

    let d_bits = d.abs().to_biguint().unwrap().bits() as usize;
    let d_bits_rounded = (d_bits + 31) & !31;

    let g_size = bytes[1] as usize;
    if g_size >= d_bits_rounded / 32 {
        return Err(KynVdfError::FormDeserializationError(format!(
            "g_size field ({}) exceeds the maximum allowed value ({}) for a {}-bit discriminant; \
             the proof bytes are corrupted",
            g_size,
            d_bits_rounded / 32 - 1,
            d_bits
        )));
    }

    let mut offset = 2;
    let a_len = d_bits_rounded / 16 - g_size;
    let t_len = d_bits_rounded / 32 - g_size;
    let g_len = g_size + 1;

    if offset + a_len + t_len + 2 * g_len > bytes.len() {
        return Err(KynVdfError::FormDeserializationError(
            "encoded field sizes exceed the 100-byte form buffer; \
             the proof bytes are truncated or corrupted"
                .to_string(),
        ));
    }

    // Parse each little-endian field
    let a_part = import_le(&bytes[offset..offset + a_len]);
    offset += a_len;

    let mut t_part = import_le(&bytes[offset..offset + t_len]);
    offset += t_len;

    let g_part = import_le(&bytes[offset..offset + g_len]);
    offset += g_len;

    let b0_part = import_le(&bytes[offset..offset + g_len]);

    // Decode sign flags
    let b_sign = (bytes[0] & BQFC_B_SIGN) != 0;
    if (bytes[0] & BQFC_T_SIGN) != 0 {
        t_part = -t_part;
    }

    let compr = CompressedForm {
        a: a_part,
        t: t_part,
        g: g_part,
        b0: b0_part,
        b_sign,
    };

    let (dec_a, dec_b) = bqfc_decompr(&compr, d)?;

    // Validate the discriminant identity: b² - 4ac = D
    let form = Form::from_abd(&dec_a, &dec_b, d).ok_or(KynVdfError::InvalidDiscriminantIdentity)?;

    // Validate the form is in canonical reduced form
    if !form.is_reduced() {
        return Err(KynVdfError::FormDeserializationError(
            "decompressed form is not in reduced normal form; \
             the proof bytes may be corrupted or produced by an incompatible implementation"
                .to_string(),
        ));
    }

    Ok(form)
}

/// Derives the 264-bit Fiat-Shamir prime challenge $B$ from the serialized generator $x$
/// and VDF output $y$.
///
/// $B = \text{HashPrime}(\text{serialize}(x) \| \text{serialize}(y), 264)$
///
/// This makes $B$ a deterministic function of the public inputs, binding the proof $\pi$
/// to a specific $(x, y)$ pair and preventing the prover from choosing $B$ adaptively.
///
/// # Errors
/// Propagates [`KynVdfError`] from [`serialize_form`] or [`hash_prime`].
pub fn get_b(d: &BigInt, x: &Form, y: &Form) -> Result<BigUint, KynVdfError> {
    let d_bits = d.abs().to_biguint().unwrap().bits() as usize;
    let ser_x = serialize_form(x, d_bits)?;
    let ser_y = serialize_form(y, d_bits)?;

    let mut concat = ser_x;
    concat.extend_from_slice(&ser_y);

    hash_prime(&concat, B_BITS, &[B_BITS - 1])
}

/// Verifies a Wesolowski VDF proof.
///
/// Checks the Wesolowski verification equation:
///
/// $$\pi^B \cdot x^r = y$$
///
/// where:
/// - $B = \text{HashPrime}(\text{ser}(x) \| \text{ser}(y), 264)$ is the Fiat-Shamir prime.
/// - $r = 2^T \bmod B$ is the remainder term.
/// - $\pi$ is the proof form.
/// - $x$ is the generator (challenge) form.
/// - $y = x^{2^T}$ is the claimed VDF output.
///
/// Verification runs in $\mathcal{O}(\log T)$ time because $B$ is a fixed-size (264-bit)
/// prime regardless of $T$, so both `pow` calls are bounded by $\log_2(B) \approx 264$
/// class group squarings.
///
/// # Parameters
/// - `d`: Negative fundamental discriminant.
/// - `x`: Generator form (challenge input).
/// - `y`: Claimed VDF output form (result of $T$ sequential squarings of $x$).
/// - `proof`: Wesolowski proof form $\pi$.
/// - `iterations`: Number of sequential squarings $T$ that were evaluated.
///
/// # Returns
/// - `Ok(true)` if $\pi^B \cdot x^r = y$ — proof is valid.
/// - `Ok(false)` if the equation does not hold — proof is invalid.
/// - `Err(KynVdfError)` if any input is malformed.
pub fn verify_wesolowski(
    d: &BigInt,
    x: &Form,
    y: &Form,
    proof: &Form,
    iterations: u64,
) -> Result<bool, KynVdfError> {
    // Derive the 264-bit Fiat-Shamir prime B = HashPrime(ser(x) || ser(y))
    let b = get_b(d, x, y)?;

    // r = 2^T mod B  (this is cheap: T is just a u64 used as exponent)
    let r = BigUint::from(2u32).modpow(&BigUint::from(iterations), &b);

    // f1 = π^B  (O(log B) ≈ O(264) squarings — independent of T)
    let f1 = proof.pow(&b, d);
    // f2 = x^r  (O(log r) ≤ O(264) squarings — independent of T)
    let f2 = x.pow(&r, d);

    // Check the verification equation: π^B · x^r == y
    let result = f1.compose(&f2, d);
    Ok(&result == y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminant_chia_test_vector() {
        let challenge = [42u8; 32];
        let d = create_discriminant(&challenge, 1024).expect("valid seed and size");
        assert!(d.is_negative());
        let p_bytes = d.abs().to_biguint().unwrap().to_bytes_be();

        let expected_prefix = [
            237, 89, 165, 1, 5, 76, 207, 152, 207, 134, 182, 117, 254, 184, 124, 248,
        ];
        assert_eq!(&p_bytes[0..16], &expected_prefix[..]);
    }

    #[test]
    fn test_create_discriminant_rejects_empty_seed() {
        let res = create_discriminant(&[], 1024);
        assert!(matches!(res, Err(KynVdfError::InvalidSeed(_))));
    }

    #[test]
    fn test_create_discriminant_rejects_bad_size() {
        let res = create_discriminant(&[1u8; 32], 0);
        assert!(matches!(res, Err(KynVdfError::InvalidDiscriminantSize(0))));

        let res2 = create_discriminant(&[1u8; 32], 100); // 100 % 8 = 4 (not a multiple of 8)
        assert!(matches!(res2, Err(KynVdfError::InvalidDiscriminantSize(100))));
    }
}
