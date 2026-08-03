use kyn_vdf::chia::{create_discriminant, get_b, serialize_form, verify_wesolowski};
use kyn_vdf::math::Form;
use num_bigint::BigUint;

fn main() {
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let x = Form::generator(&d).expect("valid generator");

    let iters = 100_000u64;

    let exp = BigUint::from(2u32).pow(iters as u32);
    let y = x.pow(&exp, &d);

    let b_val = get_b(&d, &x, &y).expect("get_b failed");
    let proof_exp = &exp / &b_val;
    let proof = x.pow(&proof_exp, &d);

    let valid = verify_wesolowski(&d, &x, &y, &proof, iters).expect("verify failed");
    assert!(valid);

    let y_bytes = serialize_form(&y, 1024).unwrap();
    let proof_bytes = serialize_form(&proof, 1024).unwrap();

    let mut combined = Vec::new();
    combined.extend_from_slice(&y_bytes);
    combined.extend_from_slice(&proof_bytes);

    println!("CHALLENGE_HEX: {}", hex::encode(challenge));
    println!("PROOF_HEX: {}", hex::encode(combined));
}
