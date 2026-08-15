use kyn_vdf_core::verify_chia_vdf;
use wasm_bindgen::prelude::*;

/// Verifies a Chia-compatible Wesolowski VDF proof from JavaScript.
#[wasm_bindgen(js_name = verifyChiaVdf)]
pub fn verify_vdf_js(
    challenge_seed: &[u8],
    proof_bytes: &[u8],
    iterations: u64,
    discriminant_size_bits: usize,
) -> Result<bool, String> {
    verify_chia_vdf(
        challenge_seed,
        proof_bytes,
        iterations,
        discriminant_size_bits,
    )
    .map_err(|error| error.to_string())
}
