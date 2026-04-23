//! Cross-check that the curve configs in `ark_ff_risc0::curves` compute the same points and
//! scalars as the reference implementations in `ark_test_curves`.
//!
//! Runs on the host via the `num-bigint` fallback.

use ark_ec::{AffineRepr, CurveConfig, CurveGroup};
use ark_ff::{PrimeField, UniformRand, Zero};
use ark_std::rand::SeedableRng;

fn lift_fr<A, B>(s: B) -> A
where
    A: PrimeField,
    B: PrimeField<BigInt = A::BigInt>,
{
    A::from_bigint(s.into_bigint()).expect("scalar < modulus")
}

mod secp256k1 {
    use super::*;
    use ark_ff_risc0::curves::secp256k1 as ours;
    use ark_test_curves::secp256k1 as reference;

    #[test]
    fn generator_matches_and_is_on_curve() {
        let ref_gen = <reference::G1Affine as AffineRepr>::generator();
        let our_gen = <ours::G1Affine as AffineRepr>::generator();

        assert_eq!(ref_gen.x.into_bigint(), our_gen.x.into_bigint(), "x");
        assert_eq!(ref_gen.y.into_bigint(), our_gen.y.into_bigint(), "y");
        assert!(our_gen.is_on_curve());
    }

    #[test]
    fn scalar_mult_matches_reference() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xBEEF);
        for _ in 0..20 {
            let k_ref = reference::Fr::rand(&mut rng);
            let k_our: ours::Fr = lift_fr(k_ref);

            let ref_p = <reference::G1Affine as AffineRepr>::generator().into_group();
            let our_p = <ours::G1Affine as AffineRepr>::generator().into_group();

            let ref_kp = (ref_p * k_ref).into_affine();
            let our_kp = (our_p * k_our).into_affine();

            assert_eq!(ref_kp.x.into_bigint(), our_kp.x.into_bigint(), "x");
            assert_eq!(ref_kp.y.into_bigint(), our_kp.y.into_bigint(), "y");
        }
    }

    #[test]
    fn modulus_and_cofactor() {
        assert_eq!(
            <ours::FqConfig as ark_ff_risc0::R0Config<4>>::MODULUS,
            <reference::Fq as PrimeField>::MODULUS,
        );
        assert_eq!(
            <ours::FrConfig as ark_ff_risc0::R0Config<4>>::MODULUS,
            <reference::Fr as PrimeField>::MODULUS,
        );
        assert_eq!(
            ours::Config::COFACTOR,
            <reference::G1Affine as AffineRepr>::Config::COFACTOR
        );
    }
}

// --- Self-consistency tests for curves without a reference impl in ark-test-curves -------------

fn generator_is_on_curve<C: ark_ec::short_weierstrass::SWCurveConfig>() {
    let g = <ark_ec::short_weierstrass::Affine<C> as AffineRepr>::generator();
    assert!(g.is_on_curve(), "generator off curve");
    assert!(!g.is_zero(), "generator is identity");
}

fn order_annihilates_generator<C: ark_ec::short_weierstrass::SWCurveConfig>() {
    use ark_ec::PrimeGroup;
    // [order] * G should be the identity, by definition of the scalar field.
    let g = <ark_ec::short_weierstrass::Projective<C> as PrimeGroup>::generator();
    let order_bytes = <C::ScalarField as PrimeField>::MODULUS;
    let order_limbs: &[u64] = order_bytes.as_ref();
    let kg = <C as ark_ec::short_weierstrass::SWCurveConfig>::mul_projective(&g, order_limbs);
    assert!(kg.is_zero(), "[order]G != identity");
}

mod bn254 {
    use super::*;
    use ark_ff_risc0::curves::bn254 as c;

    #[test]
    fn generator_on_curve() {
        generator_is_on_curve::<c::Config>();
    }
    #[test]
    fn order_annihilates() {
        order_annihilates_generator::<c::Config>();
    }
}

mod grumpkin {
    use super::*;
    use ark_ff_risc0::curves::grumpkin as c;

    #[test]
    fn generator_on_curve() {
        generator_is_on_curve::<c::Config>();
    }
    #[test]
    fn order_annihilates() {
        order_annihilates_generator::<c::Config>();
    }
}

mod secp256r1 {
    use super::*;
    use ark_ff_risc0::curves::secp256r1 as c;

    #[test]
    fn generator_on_curve() {
        generator_is_on_curve::<c::Config>();
    }
    #[test]
    fn order_annihilates() {
        order_annihilates_generator::<c::Config>();
    }
}

mod secp384r1 {
    use super::*;
    use ark_ff_risc0::curves::secp384r1 as c;

    #[test]
    fn generator_on_curve() {
        generator_is_on_curve::<c::Config>();
    }
    #[test]
    fn order_annihilates() {
        order_annihilates_generator::<c::Config>();
    }
}

mod bls12_381_g1 {
    use super::*;
    use ark_ff_risc0::curves::bls12_381 as ours;
    use ark_test_curves::bls12_381 as reference;

    #[test]
    fn generator_matches_and_is_on_curve() {
        let ref_gen = <reference::G1Affine as AffineRepr>::generator();
        let our_gen = <ours::G1Affine as AffineRepr>::generator();

        assert_eq!(ref_gen.x.into_bigint(), our_gen.x.into_bigint(), "x");
        assert_eq!(ref_gen.y.into_bigint(), our_gen.y.into_bigint(), "y");
        assert!(our_gen.is_on_curve());
    }

    #[test]
    fn scalar_mult_matches_reference() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xCAFE);
        for _ in 0..10 {
            let k_ref = reference::Fr::rand(&mut rng);
            let k_our: ours::Fr = lift_fr(k_ref);

            let ref_p = <reference::G1Affine as AffineRepr>::generator().into_group();
            let our_p = <ours::G1Affine as AffineRepr>::generator().into_group();

            let ref_kp = (ref_p * k_ref).into_affine();
            let our_kp = (our_p * k_our).into_affine();

            assert_eq!(ref_kp.x.into_bigint(), our_kp.x.into_bigint(), "x");
            assert_eq!(ref_kp.y.into_bigint(), our_kp.y.into_bigint(), "y");
        }
    }

    #[test]
    fn point_addition_matches() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(1);
        let ref_g = <reference::G1Affine as AffineRepr>::generator().into_group();
        let our_g = <ours::G1Affine as AffineRepr>::generator().into_group();

        for _ in 0..5 {
            let k1 = reference::Fr::rand(&mut rng);
            let k2 = reference::Fr::rand(&mut rng);
            let k1_our: ours::Fr = lift_fr(k1);
            let k2_our: ours::Fr = lift_fr(k2);

            let ref_sum = ((ref_g * k1) + (ref_g * k2)).into_affine();
            let our_sum = ((our_g * k1_our) + (our_g * k2_our)).into_affine();

            assert_eq!(ref_sum.x.into_bigint(), our_sum.x.into_bigint(), "x");
            assert_eq!(ref_sum.y.into_bigint(), our_sum.y.into_bigint(), "y");
        }
    }
}
