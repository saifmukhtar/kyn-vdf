#![no_main]

use libfuzzer_sys::fuzz_target;
use kyn_vdf::verify_chia_vdf;

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 + 8 + 200 {
        return;
    }

    let challenge = &data[0..32];
    let iters_bytes: [u8; 8] = data[32..40].try_into().unwrap();
    let iters = u64::from_le_bytes(iters_bytes);
    let proof = &data[40..];

    // Fuzz execution: Must NEVER panic or crash regardless of malformed/adversarial inputs
    let _ = verify_chia_vdf(challenge, proof, iters, 1024);
});
