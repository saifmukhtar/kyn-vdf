use kyn_vdf::math::Form;
use num_bigint::{BigInt, BigUint};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_class_group_axioms(k in 1u32..50u32) {
        // Construct negative fundamental discriminant D = -(8k + 7) so 2 is a generator
        let p = (k * 8 + 7) as i64;
        let d = BigInt::from(-p);

        let id = Form::identity(&d);
        prop_assert!(id.is_reduced());

        if let Some(mut generator_form) = Form::generator(&d) {
            generator_form.reduce(&d);
            prop_assert!(generator_form.is_reduced());

            // Axiom 1: Identity composition
            let comp_id = id.compose(&generator_form, &d);
            prop_assert_eq!(&comp_id, &generator_form);

            // Axiom 2: Squaring equals composition with self
            let sq = generator_form.square(&d);
            let comp_self = generator_form.compose(&generator_form, &d);
            prop_assert_eq!(&sq, &comp_self);

            // Axiom 3: Inverse composition equals identity
            let inv = Form::new(generator_form.a.clone(), -generator_form.b.clone(), generator_form.c.clone());
            let comp_inv = generator_form.compose(&inv, &d);
            prop_assert_eq!(&comp_inv, &id);

            // Axiom 4: Exponentiation consistency
            let pow_0 = generator_form.pow(&BigUint::from(0u32), &d);
            prop_assert_eq!(&pow_0, &id);

            let pow_1 = generator_form.pow(&BigUint::from(1u32), &d);
            prop_assert_eq!(&pow_1, &generator_form);

            let pow_2 = generator_form.pow(&BigUint::from(2u32), &d);
            prop_assert_eq!(&pow_2, &sq);
        }
    }
}
