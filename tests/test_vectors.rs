use kyn_vdf::verify_chia_vdf;

#[test]
fn test_known_vector_challenge_42() {
    let challenge_hex = "4242424242424242424242424242424242424242424242424242424242424242";
    let proof_hex = "0300032167dfd0eb393ed5d544e6499ba24def860ecd8a3600490f2f87b003c3e7855763969d34e2d1c60910297df3aead9f078a1f4d3973903f532977f9639f693cdbd331e8ba96bd61c895726dd157d67310ae98d1632c9bb9f28e0d7337403c0a010004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let iters = 100u64;

    let challenge = hex::decode(challenge_hex).expect("invalid challenge hex");
    let proof = hex::decode(proof_hex).expect("invalid proof hex");

    // 1. Valid verification
    let is_valid = verify_chia_vdf(&challenge, &proof, iters, 1024).expect("verification errored");
    assert!(is_valid, "Known vector verification must pass");

    // 2. Tampered proof rejection
    let mut tampered_proof = proof.clone();
    tampered_proof[50] ^= 0x01;
    let is_tampered_valid = match verify_chia_vdf(&challenge, &tampered_proof, iters, 1024) {
        Ok(v) => v,
        Err(_) => false,
    };
    assert!(!is_tampered_valid, "Tampered proof must be rejected");

    // 3. Mismatched iteration count rejection
    let is_wrong_iter_valid = match verify_chia_vdf(&challenge, &proof, iters + 1, 1024) {
        Ok(v) => v,
        Err(_) => false,
    };
    assert!(
        !is_wrong_iter_valid,
        "Wrong iteration count must be rejected"
    );
}
