//! BN254 (alt_bn128) with R0VM-accelerated base and scalar fields.
//!
//! Equation: y² = x³ + 3. Cofactor 1. Only G1 is provided; G2/pairings require an Fq2 extension
//! layer that is out of scope for this backend.

use crate::{r0_fp, R0Backend, R0Config};
use ark_ec::{
    short_weierstrass::{Affine, Projective, SWCurveConfig},
    CurveConfig,
};
use ark_ff::{AdditiveGroup, BigInt, Field, Fp, Fp256};
use core::marker::PhantomData;

// --- Base field Fq -----------------------------------------------------------------------------

pub struct FqConfig;

impl R0Config<4> for FqConfig {
    const MODULUS: BigInt<4> = BigInt::new([
        0x3C208C16D87CFD47,
        0x97816A916871CA8D,
        0xB85045B68181585D,
        0x30644E72E131A029,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
    const TWO_ADICITY: u32 = 1;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0x3C208C16D87CFD46,
            0x97816A916871CA8D,
            0xB85045B68181585D,
            0x30644E72E131A029,
        ]),
        PhantomData,
    );
}

pub type Fq = Fp256<R0Backend<FqConfig, 4>>;

// --- Scalar field Fr ---------------------------------------------------------------------------

pub struct FrConfig;

impl R0Config<4> for FrConfig {
    const MODULUS: BigInt<4> = BigInt::new([
        0x43E1F593F0000001,
        0x2833E84879B97091,
        0xB85045B68181585D,
        0x30644E72E131A029,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("5");
    const TWO_ADICITY: u32 = 28;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0x9BD61B6E725B19F0,
            0x402D111E41112ED4,
            0x00E0A7EB8EF62ABC,
            0x2A3C09F0A58A7E85,
        ]),
        PhantomData,
    );
}

pub type Fr = Fp256<R0Backend<FrConfig, 4>>;

// --- G1 ----------------------------------------------------------------------------------------

pub type G1Affine = Affine<Config>;
pub type G1Projective = Projective<Config>;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Config;

impl CurveConfig for Config {
    type BaseField = Fq;
    type ScalarField = Fr;
    const COFACTOR: &'static [u64] = &[1];
    const COFACTOR_INV: Fr = Fr::ONE;
}

impl SWCurveConfig for Config {
    const COEFF_A: Fq = Fq::ZERO;
    const COEFF_B: Fq = r0_fp!("3");
    const GENERATOR: G1Affine = G1Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    #[inline(always)]
    fn mul_by_a(_: Self::BaseField) -> Self::BaseField {
        Self::BaseField::ZERO
    }

    type ZeroFlag = ();
}

pub const G_GENERATOR_X: Fq = r0_fp!("1");
pub const G_GENERATOR_Y: Fq = r0_fp!("2");
