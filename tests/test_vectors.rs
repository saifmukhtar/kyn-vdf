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

    // The public API promises exactly 200 bytes; appended data must not be ignored.
    let mut extended_proof = proof.clone();
    extended_proof.push(0);
    assert!(verify_chia_vdf(&challenge, &extended_proof, iters, 1024).is_err());

    // 2. Tampered proof rejection
    let mut tampered_proof = proof.clone();
    tampered_proof[50] ^= 0x01;
    let is_tampered_valid =
        verify_chia_vdf(&challenge, &tampered_proof, iters, 1024).unwrap_or_default();
    assert!(!is_tampered_valid, "Tampered proof must be rejected");

    // 3. Mismatched iteration count rejection
    let is_wrong_iter_valid =
        verify_chia_vdf(&challenge, &proof, iters + 1, 1024).unwrap_or_default();
    assert!(
        !is_wrong_iter_valid,
        "Wrong iteration count must be rejected"
    );
}

#[test]
fn test_known_vector_1m_iters() {
    // Challenge seed: 32 bytes of 0x42
    // Generated via Chia reference prover (chiavdf.prove) at T = 1,000,000 (1024-bit)
    let challenge_hex = "4242424242424242424242424242424242424242424242424242424242424242";
    let proof_hex = "000084c4752cf998f8375766db49c8d07d6c40e044d4e06ad646d50f2c4e64e972fdd2d1e49c99c4a18a4eae87f9e4c5aeb92c38995df646965d7c3451fdb7f5ee23b56cc4c75522b2d8874a35bf921de7474061a6ce857c2461ca683d6c10c4101703010200baeee2d18f3d1990ad4d21b0f00a44b3e3c273c96ae66070c29d8134402e312f423f5a3f641a47c9498c149d660c11daf0ca182e603babfc22e01047212ed57453517599b1e21adec38299a7695fbe236b504e759a775e909c81722d4198f6260100";
    let iters = 1_000_000u64;

    let challenge = hex::decode(challenge_hex).expect("invalid challenge hex");
    let proof = hex::decode(proof_hex).expect("invalid proof hex");

    // 1. Valid verification at 1M iterations
    let is_valid = verify_chia_vdf(&challenge, &proof, iters, 1024).expect("verification errored");
    assert!(
        is_valid,
        "1,000,000-iteration Chia proof verification must pass"
    );

    // 2. Tampered proof rejection
    let mut tampered_proof = proof.clone();
    tampered_proof[50] ^= 0x01;
    let is_tampered_valid = match verify_chia_vdf(&challenge, &tampered_proof, iters, 1024) {
        Ok(v) => v,
        Err(_) => false,
    };
    assert!(!is_tampered_valid, "Tampered 1M proof must be rejected");

    // 3. Mismatched iteration count rejection (T + 1)
    let is_wrong_iter_valid = match verify_chia_vdf(&challenge, &proof, iters + 1, 1024) {
        Ok(v) => v,
        Err(_) => false,
    };
    assert!(
        !is_wrong_iter_valid,
        "Wrong iteration count (1M + 1) must be rejected"
    );
}
