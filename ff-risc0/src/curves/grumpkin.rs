//! Grumpkin curve with R0VM-accelerated fields.
//!
//! Grumpkin is BN254's paired "inner" curve: its base field is BN254's scalar field (`Fr_bn254`)
//! and vice versa. Equation: y² = x³ - 17. Cofactor 1.

use crate::curves::bn254;
use crate::{r0_fp, R0Backend};
use ark_ec::{
    short_weierstrass::{Affine, Projective, SWCurveConfig},
    CurveConfig,
};
use ark_ff::{AdditiveGroup, Fp256};

pub type Fq = bn254::Fr;
pub type Fr = bn254::Fq;
pub type FqConfig = bn254::FrConfig;
pub type FrConfig = bn254::FqConfig;

// convenience re-export for explicit Fp256 typing below
type _Fp256R0<P> = Fp256<R0Backend<P, 4>>;

pub type G1Affine = Affine<Config>;
pub type G1Projective = Projective<Config>;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Config;

impl CurveConfig for Config {
    type BaseField = Fq;
    type ScalarField = Fr;
    const COFACTOR: &'static [u64] = &[1];
    const COFACTOR_INV: Fr = <Fr as ark_ff::Field>::ONE;
}

impl SWCurveConfig for Config {
    const COEFF_A: Fq = Fq::ZERO;
    // -17 mod Fr_bn254 = Fr_bn254_modulus - 17
    const COEFF_B: Fq =
        r0_fp!("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593effffff0");
    const GENERATOR: G1Affine = G1Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    #[inline(always)]
    fn mul_by_a(_: Self::BaseField) -> Self::BaseField {
        Self::BaseField::ZERO
    }

    type ZeroFlag = ();
}

pub const G_GENERATOR_X: Fq = r0_fp!("1");
pub const G_GENERATOR_Y: Fq = r0_fp!("0x2cf135e7506a45d632d270d45f1181294833fc48d823f272c");
