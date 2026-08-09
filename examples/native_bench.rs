use kyn_vdf::verify_chia_vdf;
use std::time::Instant;

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║          kyn-vdf  ·  Pure Native CPU Benchmark (T=100k)          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    // Exact same test vector used in the WASM benchmark.html
    let challenge_hex = "4242424242424242424242424242424242424242424242424242424242424242";
    let proof_hex = "02006659312e4ea7fb4f6025f86cbcd40713613855024758762ee6752a61364aa11c855bf57478e053e9835c401a9f634d56543ba9641fe9939fc35ba3c9988c1e271d2727918ddfd5737b767f925af9c3cf1a51bfef842b090a33b12bf8922a4800010003000a7fe150d493b2687904b6a5b43874c125721085835cb4deaf0a25b4cbba6ca804d9136f53fb82ea9b5616261c5570bdccc9bc0522d738a8782b1eb3b3f81e2addb772e745324b0645b93f1a5eaf28ed4b63c46cc5b98454d04ebc85a788822d0100";

    let challenge = hex::decode(challenge_hex).unwrap();
    let proof = hex::decode(proof_hex).unwrap();
    let iters = 100_000u64;

    println!("Starting 1,000 rounds of native verification...");

    // Warmup round
    let _ = verify_chia_vdf(&challenge, &proof, iters, 1024).unwrap();

    let runs = 1000;
    let mut success = true;

    let t0 = Instant::now();

    for _ in 0..runs {
        let valid = verify_chia_vdf(&challenge, &proof, iters, 1024).unwrap();
        if !valid {
            success = false;
        }
    }

    let t1 = t0.elapsed();
    let total_time_ms = t1.as_secs_f64() * 1000.0;
    let avg_time_ms = total_time_ms / runs as f64;

    println!();
    println!(
        "Result: {}",
        if success { "✅ VALID" } else { "❌ INVALID" }
    );
    println!("Total time: {:.2} seconds", t1.as_secs_f64());
    println!("Average time per verification: {:.2} ms", avg_time_ms);
    println!("(Measured over {} runs natively)", runs);
    println!();
}
