use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use kyn_vdf::chia::{create_discriminant, get_b, verify_wesolowski};
use kyn_vdf::math::Form;
use num_bigint::BigUint;

fn bench_class_group_arithmetic(c: &mut Criterion) {
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let x = Form::generator(&d).unwrap();

    let mut group = c.benchmark_group("class_group_arithmetic");

    group.bench_function("nudupl_squaring_1024bit", |b| {
        b.iter(|| black_box(&x).square(black_box(&d)))
    });

    let x2 = x.square(&d);
    group.bench_function("nucomp_composition_1024bit", |b| {
        b.iter(|| black_box(&x).compose(black_box(&x2), black_box(&d)))
    });

    let exp = BigUint::from(2u32).pow(64);
    group.bench_function("pow_2pow64_exponent", |b| {
        b.iter(|| black_box(&x).pow(black_box(&exp), black_box(&d)))
    });

    group.finish();
}

/// Bench end-to-end Wesolowski verification.
/// Key property: verify time must be flat (O(log T)) regardless of iteration count.
fn bench_wesolowski_verification(c: &mut Criterion) {
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024).expect("valid seed");
    let x = Form::generator(&d).unwrap();

    // Pre-generate proofs for each iteration count using fast_pow
    // proof = x^floor(2^T / B), y = x^(2^T)
    // For benchmarking: we use a fixed pre-computed y and proof
    // so we only measure verify cost, not prove cost.
    let iter_counts: &[u64] = &[100, 1_000, 10_000, 100_000, 500_000];

    let mut group = c.benchmark_group("wesolowski_verification");
    group.sample_size(20); // verification is slow enough

    for &iters in iter_counts {
        // compute y = x^(2^iters) via repeated squaring
        let exp = BigUint::from(2u32).pow(iters as u32);
        let y = x.pow(&exp, &d);

        // derive B, compute r = 2^iters mod B, compute proof = x^floor(2^iters/B)
        let b_val = get_b(&d, &x, &y).unwrap();
        let r = BigUint::from(2u32).modpow(&BigUint::from(iters), &b_val);
        let proof_exp = BigUint::from(2u32).pow(iters as u32) / &b_val;
        let proof = x.pow(&proof_exp, &d);

        group.bench_with_input(
            BenchmarkId::new("verify_iters", iters),
            &iters,
            |b, &_iters| {
                b.iter(|| {
                    verify_wesolowski(
                        black_box(&d),
                        black_box(&x),
                        black_box(&y),
                        black_box(&proof),
                        black_box(iters),
                    )
                })
            },
        );

        let _ = r; // used above
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_class_group_arithmetic,
    bench_wesolowski_verification
);
criterion_main!(benches);
