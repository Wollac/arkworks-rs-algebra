//! Cross-check the R0VM backend against arkworks' default `MontBackend`.
//!
//! Instantiates a secp256k1 base field through [`R0Backend`] and verifies that every field
//! operation produces the same canonical integer as `ark_test_curves::secp256k1::Fq`. Runs on
//! the host using the `num-bigint` fallback in `crate::ffi`, so no zkvm is required.

use ark_ff::{AdditiveGroup, BigInt, Field, Fp, Fp256, PrimeField, UniformRand};
use ark_ff_risc0::{r0_fp, R0Backend, R0Config};
use ark_std::rand::SeedableRng;
use ark_test_curves::secp256k1::Fq as ArkFq;

pub struct OurFqConfig;

impl R0Config<4> for OurFqConfig {
    const MODULUS: BigInt<4> = ark_ff::BigInt!(
        "115792089237316195423570985008687907853269984665640564039457584007908834671663"
    );
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
    const TWO_ADICITY: u32 = 1;
    // -1 mod p (primitive 2nd root of unity).
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = r0_fp!("-1");
}

pub type OurFq = Fp256<R0Backend<OurFqConfig, 4>>;

fn lift(x: ArkFq) -> OurFq {
    OurFq::from_bigint(x.into_bigint()).expect("ark value < p")
}

#[test]
fn constants_match() {
    assert_eq!(ArkFq::ZERO.into_bigint(), OurFq::ZERO.into_bigint());
    assert_eq!(ArkFq::ONE.into_bigint(), OurFq::ONE.into_bigint());
    assert_eq!(ArkFq::NEG_ONE.into_bigint(), OurFq::NEG_ONE.into_bigint());
}

#[test]
fn cross_check_ops() {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
    for _ in 0..200 {
        let a_ark = ArkFq::rand(&mut rng);
        let b_ark = ArkFq::rand(&mut rng);
        let a = lift(a_ark);
        let b = lift(b_ark);

        assert_eq!((a_ark + b_ark).into_bigint(), (a + b).into_bigint(), "add");
        assert_eq!((a_ark - b_ark).into_bigint(), (a - b).into_bigint(), "sub");
        assert_eq!((a_ark * b_ark).into_bigint(), (a * b).into_bigint(), "mul");
        assert_eq!((-a_ark).into_bigint(), (-a).into_bigint(), "neg");
        assert_eq!(
            a_ark.double().into_bigint(),
            a.double().into_bigint(),
            "double"
        );
        assert_eq!(
            a_ark.square().into_bigint(),
            a.square().into_bigint(),
            "square"
        );

        if let Some(i_ark) = a_ark.inverse() {
            let i = a.inverse().expect("a != 0 matches");
            assert_eq!(i_ark.into_bigint(), i.into_bigint(), "inv");
        } else {
            assert!(a.inverse().is_none(), "zero: both None");
        }
    }
}

#[test]
fn neg_zero_is_zero() {
    let z = OurFq::ZERO;
    assert_eq!((-z).into_bigint(), z.into_bigint());
}

#[test]
fn from_bigint_out_of_range() {
    let p = <OurFqConfig as R0Config<4>>::MODULUS;
    assert!(OurFq::from_bigint(p).is_none(), "p is not in range [0, p)");
    let p_plus_one = {
        let mut limbs = p.0;
        limbs[0] += 1;
        BigInt::new(limbs)
    };
    assert!(OurFq::from_bigint(p_plus_one).is_none());
}

#[test]
fn sum_of_products_matches() {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(1);
    let a_ark: [ArkFq; 5] = core::array::from_fn(|_| ArkFq::rand(&mut rng));
    let b_ark: [ArkFq; 5] = core::array::from_fn(|_| ArkFq::rand(&mut rng));
    let a: [OurFq; 5] = core::array::from_fn(|i| lift(a_ark[i]));
    let b: [OurFq; 5] = core::array::from_fn(|i| lift(b_ark[i]));

    let ref_sop = ArkFq::sum_of_products(&a_ark, &b_ark);
    let our_sop = OurFq::sum_of_products(&a, &b);
    assert_eq!(ref_sop.into_bigint(), our_sop.into_bigint());
}
