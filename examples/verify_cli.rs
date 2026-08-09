use std::env;
use std::io::{self, Read};
use std::time::Instant;
use kyn_vdf::verify_chia_vdf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (challenge_hex, proof_hex, iterations, discriminant_bits) = if args.len() >= 5 {
        (
            args[1].clone(),
            args[2].clone(),
            args[3].parse::<u64>().unwrap_or(0),
            args[4].parse::<usize>().unwrap_or(1024),
        )
    } else {
        // Read JSON or line from stdin
        let mut input = String::new();
        if io::stdin().read_to_string(&mut input).is_ok() && !input.trim().is_empty() {
            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.len() >= 4 {
                (
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].parse::<u64>().unwrap_or(0),
                    parts[3].parse::<usize>().unwrap_or(1024),
                )
            } else {
                eprintln!("Usage: verify_cli <challenge_hex> <proof_hex> <iterations> <bits>");
                std::process::exit(1);
            }
        } else {
            eprintln!("Usage: verify_cli <challenge_hex> <proof_hex> <iterations> <bits>");
            std::process::exit(1);
        }
    };

    let challenge = match hex::decode(&challenge_hex) {
        Ok(c) => c,
        Err(e) => {
            println!(
                r#"{{"valid": false, "error": "Invalid challenge hex: {}", "elapsed_ms": 0.0}}"#,
                e
            );
            std::process::exit(1);
        }
    };

    let proof = match hex::decode(&proof_hex) {
        Ok(p) => p,
        Err(e) => {
            println!(
                r#"{{"valid": false, "error": "Invalid proof hex: {}", "elapsed_ms": 0.0}}"#,
                e
            );
            std::process::exit(1);
        }
    };

    let start = Instant::now();
    let result = verify_chia_vdf(&challenge, &proof, iterations, discriminant_bits);
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(valid) => {
            println!(
                r#"{{"valid": {}, "error": null, "elapsed_ms": {:.3}}}"#,
                valid, elapsed_ms
            );
        }
        Err(err) => {
            println!(
                r#"{{"valid": false, "error": "{}", "elapsed_ms": {:.3}}}"#,
                err, elapsed_ms
            );
        }
    }
}
