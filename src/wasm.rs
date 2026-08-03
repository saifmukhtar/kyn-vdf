use wasm_bindgen::prelude::*;
use crate::verify_chia_vdf;

/// Verifies a Chia-compatible Wesolowski VDF proof from JavaScript.
/// 
/// Takes JavaScript `Uint8Array`s for the challenge and proof.
#[wasm_bindgen(js_name = verifyChiaVdf)]
pub fn verify_vdf_js(
    challenge_seed: &[u8],
    proof_bytes: &[u8],
    iterations: u64,
    discriminant_size_bits: usize,
) -> Result<bool, String> {
    verify_chia_vdf(challenge_seed, proof_bytes, iterations, discriminant_size_bits)
        .map_err(|e| e.to_string())
}
