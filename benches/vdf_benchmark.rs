use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kyn_vdf::chia::create_discriminant;
use kyn_vdf::math::Form;
use num_bigint::BigUint;

fn bench_nucomp_nudupl(c: &mut Criterion) {
    let challenge = [0x42u8; 32];
    let d = create_discriminant(&challenge, 1024);
    let x = Form::generator(&d).unwrap();

    let mut group = c.benchmark_group("class_group_arithmetic");

    group.bench_function("nudupl_squaring_1024bit", |b| {
        b.iter(|| {
            black_box(&x).square(black_box(&d))
        })
    });

    let x2 = x.square(&d);
    group.bench_function("nucomp_composition_1024bit", |b| {
        b.iter(|| {
            black_box(&x).compose(black_box(&x2), black_box(&d))
        })
    });

    let exp = BigUint::from(2u32).pow(64);
    group.bench_function("pow_64bit_exponent", |b| {
        b.iter(|| {
            black_box(&x).pow(black_box(&exp), black_box(&d))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_nucomp_nudupl);
criterion_main!(benches);
