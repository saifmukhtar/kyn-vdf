use kyn_vdf::chia::{create_discriminant, deserialize_form, serialize_form};
use kyn_vdf::error::KynVdfError;
use kyn_vdf::math::Form;
use kyn_vdf::verify_chia_vdf;
use num_bigint::BigInt;
use num_traits::{One, Signed};

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
