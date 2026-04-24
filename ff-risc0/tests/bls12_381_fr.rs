//! Cross-check the R0 backend — including the `R0Config` derive — against arkworks'
//! default `MontBackend` on BLS12-381's scalar field.
//!
//! Exercises:
//! * Arithmetic parity with `MontBackend` (add/sub/mul/double/square/neg/inv, sum-of-products).
//! * The derive's macro-time `modpow` path: `TWO_ADIC_ROOT_OF_UNITY = generator^t mod p` for
//!   a field with two-adicity 32 (not the trivial s=1 case).
//! * The mixed-radix path: `SMALL_SUBGROUP_BASE` + `LARGE_SUBGROUP_ROOT_OF_UNITY`.

use ark_ff::{AdditiveGroup, BigInt, FftField, Field, Fp256, PrimeField, UniformRand};
use ark_ff_risc0::{R0Backend, R0Config};
use ark_std::rand::SeedableRng;
use ark_test_curves::bls12_381::Fr as ArkFr;

#[derive(R0Config)]
#[modulus = "52435875175126190479447740508185965837690552500527637822603658699938581184513"]
#[generator = "7"]
#[small_subgroup_base = "3"]
#[small_subgroup_power = "1"]
pub struct OurFrConfig;

pub type OurFr = Fp256<R0Backend<OurFrConfig, 4>>;

fn lift(x: ArkFr) -> OurFr {
    OurFr::from_bigint(x.into_bigint()).expect("ark value < p")
}

#[test]
fn constants_match() {
    assert_eq!(ArkFr::ZERO.into_bigint(), OurFr::ZERO.into_bigint());
    assert_eq!(ArkFr::ONE.into_bigint(), OurFr::ONE.into_bigint());
    assert_eq!(ArkFr::NEG_ONE.into_bigint(), OurFr::NEG_ONE.into_bigint());
    assert_eq!(ArkFr::GENERATOR.into_bigint(), OurFr::GENERATOR.into_bigint());
}

#[test]
fn two_adic_constants_match() {
    assert_eq!(ArkFr::TWO_ADICITY, OurFr::TWO_ADICITY);
    assert_eq!(
        ArkFr::TWO_ADIC_ROOT_OF_UNITY.into_bigint(),
        OurFr::TWO_ADIC_ROOT_OF_UNITY.into_bigint(),
        "TWO_ADIC_ROOT_OF_UNITY (generator^t mod p) mismatch",
    );
}

#[test]
fn mixed_radix_constants_match() {
    assert_eq!(ArkFr::SMALL_SUBGROUP_BASE, OurFr::SMALL_SUBGROUP_BASE);
    assert_eq!(
        ArkFr::SMALL_SUBGROUP_BASE_ADICITY,
        OurFr::SMALL_SUBGROUP_BASE_ADICITY,
    );
    let ark_large = ArkFr::LARGE_SUBGROUP_ROOT_OF_UNITY.expect("bls12_381 Fr has mixed radix");
    let our_large = OurFr::LARGE_SUBGROUP_ROOT_OF_UNITY.expect("ours too");
    assert_eq!(ark_large.into_bigint(), our_large.into_bigint());
}

#[test]
fn cross_check_ops() {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
    for _ in 0..200 {
        let a_ark = ArkFr::rand(&mut rng);
        let b_ark = ArkFr::rand(&mut rng);
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
    let z = OurFr::ZERO;
    assert_eq!((-z).into_bigint(), z.into_bigint());
}

#[test]
fn from_bigint_out_of_range() {
    let p = <OurFrConfig as R0Config<4>>::MODULUS;
    assert!(OurFr::from_bigint(p).is_none(), "p is not in range [0, p)");
    let p_plus_one = {
        let mut limbs = p.0;
        limbs[0] += 1;
        BigInt::new(limbs)
    };
    assert!(OurFr::from_bigint(p_plus_one).is_none());
}

#[test]
fn sum_of_products_matches() {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(1);
    let a_ark: [ArkFr; 5] = core::array::from_fn(|_| ArkFr::rand(&mut rng));
    let b_ark: [ArkFr; 5] = core::array::from_fn(|_| ArkFr::rand(&mut rng));
    let a: [OurFr; 5] = core::array::from_fn(|i| lift(a_ark[i]));
    let b: [OurFr; 5] = core::array::from_fn(|i| lift(b_ark[i]));

    let ref_sop = ArkFr::sum_of_products(&a_ark, &b_ark);
    let our_sop = OurFr::sum_of_products(&a, &b);
    assert_eq!(ref_sop.into_bigint(), our_sop.into_bigint());
}
