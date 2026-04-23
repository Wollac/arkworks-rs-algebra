//! NIST P-256 / secp256r1 with R0VM-accelerated base and scalar fields.
//!
//! Equation: y² = x³ - 3·x + b. Cofactor 1.

use crate::{r0_fp, R0Backend, R0Config};
use ark_ec::{
    short_weierstrass::{Affine, Projective, SWCurveConfig},
    CurveConfig,
};
use ark_ff::{BigInt, Fp, Fp256};
use core::marker::PhantomData;

// --- Base field Fq -----------------------------------------------------------------------------

pub struct FqConfig;

impl R0Config<4> for FqConfig {
    // p = 2^256 - 2^224 + 2^192 + 2^96 - 1
    const MODULUS: BigInt<4> = BigInt::new([
        0xFFFFFFFFFFFFFFFF,
        0x00000000FFFFFFFF,
        0x0000000000000000,
        0xFFFFFFFF00000001,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
    const TWO_ADICITY: u32 = 1;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0xFFFFFFFFFFFFFFFE,
            0x00000000FFFFFFFF,
            0x0000000000000000,
            0xFFFFFFFF00000001,
        ]),
        PhantomData,
    );
}

pub type Fq = Fp256<R0Backend<FqConfig, 4>>;

// --- Scalar field Fr ---------------------------------------------------------------------------

pub struct FrConfig;

impl R0Config<4> for FrConfig {
    // n = group order of secp256r1
    const MODULUS: BigInt<4> = BigInt::new([
        0xF3B9CAC2FC632551,
        0xBCE6FAADA7179E84,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFF00000000,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("7");
    const TWO_ADICITY: u32 = 4;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0x0592D7FBB41E6602,
            0x1546CAD004378DAF,
            0xBA807ACE842A3DFC,
            0xFFC97F062A770992,
        ]),
        PhantomData,
    );
}

pub type Fr = Fp256<R0Backend<FrConfig, 4>>;

// --- Curve -------------------------------------------------------------------------------------

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
    // a = -3 mod p
    const COEFF_A: Fq =
        r0_fp!("0xffffffff00000001000000000000000000000000fffffffffffffffffffffffc");
    const COEFF_B: Fq =
        r0_fp!("0x5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b");
    const GENERATOR: G1Affine = G1Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    type ZeroFlag = ();
}

pub const G_GENERATOR_X: Fq =
    r0_fp!("0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296");
pub const G_GENERATOR_Y: Fq =
    r0_fp!("0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5");
