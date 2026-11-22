use kyn_vdf::chia::{
    CompressedForm, bqfc_decompr, create_discriminant, deserialize_form, hash_prime, serialize_form,
};
use kyn_vdf::error::KynVdfError;
use kyn_vdf::math::{Form, isqrt_fourth};
use kyn_vdf::verify_chia_vdf;
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};

#[test]
fn test_zero_iterations_rejected() {
    let challenge = [1u8; 32];
    let fake_proof = vec![0u8; 200];
    let res = verify_chia_vdf(&challenge, &fake_proof, 0, 1024);
    assert_eq!(res, Err(KynVdfError::InvalidIterations(0)));
}

#[test]
fn test_short_proof_bytes_rejected() {
    let challenge = [1u8; 32];
    let short_proof = vec![0u8; 150];
    let res = verify_chia_vdf(&challenge, &short_proof, 1000, 1024);
    assert!(matches!(res, Err(KynVdfError::InvalidProofLength { .. })));
}

#[test]
fn test_discriminant_sizes() {
    let challenge = [0x55u8; 32];
    for &bits in &[512, 1024, 2048] {
        let d = create_discriminant(&challenge, bits).expect("valid seed and size");
        assert!(d.is_negative());
        let x = Form::generator(&d).expect("generator should exist for D = -p, p = 7 mod 8");
        assert_eq!(x.a, BigInt::from(2));
        assert_eq!(x.b, BigInt::one());
        assert!(x.is_reduced());
    }
}

#[test]
fn test_form_identity_and_inversion() {
    let challenge = [0x12u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let id = Form::identity(&d);
    assert!(id.is_reduced());

    let generator_form = Form::generator(&d).unwrap();
    let gen_inv = Form::new(
        generator_form.a.clone(),
        -generator_form.b.clone(),
        generator_form.c.clone(),
    );

    // gen * gen^-1 == identity
    let res = generator_form.compose(&gen_inv, &d);
    assert_eq!(res, id);
}

#[test]
fn test_form_serialization_roundtrip() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let generator_form = Form::generator(&d).unwrap();

    let bytes = serialize_form(&generator_form, 1024).expect("serialization failed");
    assert_eq!(bytes.len(), 100);

    let deserialized = deserialize_form(&d, &bytes).expect("deserialization failed");
    assert_eq!(deserialized, generator_form);

    // Test a squared form roundtrip
    let gen2 = generator_form.square(&d);
    let bytes2 = serialize_form(&gen2, 1024).expect("serialization failed");
    let deserialized2 = deserialize_form(&d, &bytes2).expect("deserialization failed");
    assert_eq!(deserialized2, gen2);
}

#[test]
fn test_special_form_encodings_are_unique() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");

    for flag in [0x05, 0x06, 0x08 | 0x04] {
        let mut bytes = [0u8; 100];
        bytes[0] = flag;
        assert!(deserialize_form(&d, &bytes).is_err());
    }

    let mut dirty_padding = [0u8; 100];
    dirty_padding[0] = 0x04;
    dirty_padding[99] = 1;
    assert!(deserialize_form(&d, &dirty_padding).is_err());
}

#[test]
fn test_isqrt_fourth_negative_handled_safely() {
    assert_eq!(isqrt_fourth(&BigInt::from(-100)), BigInt::zero());
    assert_eq!(isqrt_fourth(&BigInt::from(0)), BigInt::zero());
    assert_eq!(isqrt_fourth(&BigInt::from(16)), BigInt::from(2));
}

#[test]
fn test_hash_prime_empty_seed_rejected() {
    let res = hash_prime(&[], 1024, &[0, 1, 2]);
    assert!(matches!(res, Err(KynVdfError::InvalidSeed(_))));
}

#[test]
fn test_hash_prime_bitmask_out_of_bounds_rejected() {
    let seed = [0x42u8; 32];
    let res = hash_prime(&seed, 1024, &[0, 1024]);
    assert!(matches!(
        res,
        Err(KynVdfError::InvalidDiscriminantSize(1024))
    ));
}

#[test]
fn test_bqfc_decompr_zero_a_rejected() {
    let compr = CompressedForm {
        a: BigInt::zero(),
        t: BigInt::zero(),
        g: BigInt::zero(),
        b0: BigInt::zero(),
        b_sign: false,
    };
    let d = BigInt::from(-7);
    let res = bqfc_decompr(&compr, &d);
    assert!(matches!(res, Err(KynVdfError::FormDeserializationError(_))));
}

#[test]
fn test_serialize_form_2048_bit_rejected_cleanly() {
    let form = Form::new(BigInt::from(3), BigInt::from(1), BigInt::from(2));
    let res = serialize_form(&form, 2048);
    assert!(matches!(
        res,
        Err(KynVdfError::InvalidDiscriminantSize(2048))
    ));
}

#[test]
fn test_slow_path_rejects_unused_upper_flag_bits() {
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let form = Form::generator(&d).expect("generator").square(&d);
    let mut bytes = serialize_form(&form, 1024).expect("serialization failed");
    // Flip unused upper bit 0x10 in bytes[0]
    bytes[0] |= 0x10;
    assert!(deserialize_form(&d, &bytes).is_err());
    // Flip unused upper bit 0x80 in bytes[0]
    bytes[0] ^= 0x10;
    bytes[0] |= 0x80;
    assert!(deserialize_form(&d, &bytes).is_err());
}

#[test]
fn test_slow_path_rejects_trailing_dirty_padding_sub_1024() {
    let challenge = [0x55u8; 32];
    let d = create_discriminant(&challenge, 512).expect("valid 512-bit seed");
    let form = Form::generator(&d).expect("generator").square(&d);
    let mut bytes = serialize_form(&form, 512).expect("serialization failed");
    // Trailing bytes (byte 99) in the 100-byte wire format must be 0
    bytes[99] = 1;
    let res = deserialize_form(&d, &bytes);
    assert!(matches!(res, Err(KynVdfError::FormDeserializationError(_))));
}

#[test]
fn test_reduce_degenerate_does_not_panic() {
    let mut f1 = Form::new(BigInt::zero(), BigInt::zero(), BigInt::zero());
    f1.reduce(&BigInt::from(-7));
    assert_eq!(f1.a, BigInt::zero());

    let mut f2 = Form::new(BigInt::from(-5), BigInt::one(), BigInt::from(2));
    f2.reduce(&BigInt::from(-7));
    assert_eq!(f2.a, BigInt::from(-5));

    let mut f3 = Form::new(BigInt::from(2), BigInt::one(), BigInt::from(2));
    f3.reduce(&BigInt::from(7)); // positive discriminant
    assert_eq!(f3.a, BigInt::from(2));
}

#[test]
fn test_slow_path_rejects_unflagged_identity() {
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");

    // Manually construct slow-path wire bytes for identity (a=1, b=1, t=0, g=1, b0=0)
    let mut bytes = [0u8; 100];
    bytes[0] = 0; // No fast-path BQFC_IS_1 flag
    bytes[1] = 0; // g_size = 0
    bytes[2] = 1; // a = 1
    bytes[98] = 1; // g = 1

    let res = deserialize_form(&d, &bytes);
    assert!(matches!(res, Err(KynVdfError::FormDeserializationError(_))));
}
