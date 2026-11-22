use kyn_vdf::chia::{create_discriminant, deserialize_form};
use kyn_vdf::error::KynVdfError;
use kyn_vdf::verify_chia_vdf;

const BQFC_B_SIGN: u8 = 0x01;
const BQFC_T_SIGN: u8 = 0x02;
const BQFC_IS_1: u8 = 0x04;
const BQFC_IS_GEN: u8 = 0x08;
const FORM_BYTES: usize = 100;
const PROOF_BYTES: usize = 200;

fn known_vector() -> (Vec<u8>, Vec<u8>, u64) {
    let challenge = hex::decode("4242424242424242424242424242424242424242424242424242424242424242")
        .expect("challenge hex");
    let proof = hex::decode(
        "0300032167dfd0eb393ed5d544e6499ba24def860ecd8a3600490f2f87b003c3e7855763969d34e2d1c60910297df3aead9f078a1f4d3973903f532977f9639f693cdbd331e8ba96bd61c895726dd157d67310ae98d1632c9bb9f28e0d7337403c0a010004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("proof hex");
    (challenge, proof, 100)
}

fn special_form(flag: u8) -> [u8; FORM_BYTES] {
    let mut bytes = [0u8; FORM_BYTES];
    bytes[0] = flag;
    bytes
}

fn assert_rejected(result: Result<bool, KynVdfError>) {
    match result {
        Ok(false) | Err(_) => {}
        Ok(true) => panic!("expected rejection, verifier accepted"),
    }
}

#[test]
fn rejects_199_byte_proof() {
    let (challenge, proof, iters) = known_vector();
    let short = &proof[..199];
    assert!(matches!(
        verify_chia_vdf(&challenge, short, iters, 1024),
        Err(KynVdfError::InvalidProofLength {
            expected: 200,
            actual: 199
        })
    ));
}

#[test]
fn accepts_exact_200_byte_known_vector() {
    let (challenge, proof, iters) = known_vector();
    assert_eq!(proof.len(), PROOF_BYTES);
    assert!(
        verify_chia_vdf(&challenge, &proof, iters, 1024).expect("verification errored"),
        "Known vector verification must pass"
    );
}

#[test]
fn rejects_201_byte_proof() {
    let (challenge, mut proof, iters) = known_vector();
    proof.push(0);
    assert!(matches!(
        verify_chia_vdf(&challenge, &proof, iters, 1024),
        Err(KynVdfError::InvalidProofLength {
            expected: 200,
            actual: 201
        })
    ));
}

#[test]
fn rejects_short_and_trailing_lengths() {
    let (challenge, proof, iters) = known_vector();
    for len in [0, 1, 99, 100, 101, 150, 198, 202, 400] {
        let mut bytes = proof.clone();
        bytes.resize(len, 0);
        assert!(
            matches!(
                verify_chia_vdf(&challenge, &bytes, iters, 1024),
                Err(KynVdfError::InvalidProofLength { expected: 200, actual }) if actual == len
            ),
            "length {len} must be rejected"
        );
    }
}

#[test]
fn rejects_dirty_special_form_padding_at_every_position() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    for flag in [BQFC_IS_1, BQFC_IS_GEN] {
        for index in 1..FORM_BYTES {
            let mut bytes = special_form(flag);
            bytes[index] = 1;
            assert!(
                deserialize_form(&d, &bytes).is_err(),
                "flag 0x{flag:02x} padding at {index} must be rejected"
            );
        }
    }
}

#[test]
fn rejects_identity_and_generator_sign_aliases() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    for flag in [
        BQFC_IS_1 | BQFC_B_SIGN,
        BQFC_IS_1 | BQFC_T_SIGN,
        BQFC_IS_GEN | BQFC_B_SIGN,
        BQFC_IS_GEN | BQFC_T_SIGN,
    ] {
        assert!(deserialize_form(&d, &special_form(flag)).is_err());
    }
}

#[test]
fn rejects_mixed_identity_and_generator_flags() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    assert!(deserialize_form(&d, &special_form(BQFC_IS_1 | BQFC_IS_GEN)).is_err());
}

#[test]
fn rejects_unknown_special_form_flag_aliases() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    for high_bit in [0x10u8, 0x20, 0x40, 0x80] {
        assert!(deserialize_form(&d, &special_form(BQFC_IS_1 | high_bit)).is_err());
        assert!(deserialize_form(&d, &special_form(BQFC_IS_GEN | high_bit)).is_err());
    }
}

#[test]
fn accepts_canonical_identity_and_generator() {
    let challenge = [0x77u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let identity = deserialize_form(&d, &special_form(BQFC_IS_1)).expect("identity");
    assert_eq!(identity.a, num_bigint::BigInt::from(1));
    assert_eq!(identity.b, num_bigint::BigInt::from(1));
    let generator = deserialize_form(&d, &special_form(BQFC_IS_GEN)).expect("generator");
    assert_eq!(generator.a, num_bigint::BigInt::from(2));
    assert_eq!(generator.b, num_bigint::BigInt::from(1));
}

#[test]
fn rejects_malformed_bqfc_size_fields() {
    let (challenge, mut proof, iters) = known_vector();
    proof[1] = 0xFF;
    assert_rejected(verify_chia_vdf(&challenge, &proof, iters, 1024));

    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let mut y = proof[..FORM_BYTES].to_vec();
    y[1] = 0xFF;
    assert!(deserialize_form(&d, &y).is_err());
}

#[test]
fn rejects_challenge_mutation() {
    let (mut challenge, proof, iters) = known_vector();
    challenge[0] ^= 0x01;
    assert_rejected(verify_chia_vdf(&challenge, &proof, iters, 1024));
}

#[test]
fn rejects_iteration_mutation_including_zero() {
    let (challenge, proof, iters) = known_vector();
    assert!(matches!(
        verify_chia_vdf(&challenge, &proof, 0, 1024),
        Err(KynVdfError::InvalidIterations(0))
    ));
    assert_rejected(verify_chia_vdf(&challenge, &proof, iters + 1, 1024));
    assert_rejected(verify_chia_vdf(&challenge, &proof, 1, 1024));
}

#[test]
fn rejects_y_and_pi_mutations() {
    let (challenge, proof, iters) = known_vector();
    let mut y_mutated = proof.clone();
    y_mutated[50] ^= 0x01;
    assert_rejected(verify_chia_vdf(&challenge, &y_mutated, iters, 1024));

    let mut pi_mutated = proof.clone();
    pi_mutated[150] ^= 0x01;
    assert_rejected(verify_chia_vdf(&challenge, &pi_mutated, iters, 1024));
}

#[test]
fn deterministic_repeated_verification() {
    let (challenge, proof, iters) = known_vector();
    let first = verify_chia_vdf(&challenge, &proof, iters, 1024).expect("first");
    let second = verify_chia_vdf(&challenge, &proof, iters, 1024).expect("second");
    assert!(first);
    assert_eq!(first, second);
}

#[test]
fn known_vector_y_and_pi_are_exact_100_byte_forms() {
    let (_challenge, proof, _iters) = known_vector();
    assert_eq!(proof.len(), 200);
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    deserialize_form(&d, &proof[..100]).expect("y");
    deserialize_form(&d, &proof[100..]).expect("pi");
}
