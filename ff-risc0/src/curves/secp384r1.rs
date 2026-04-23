//! NIST P-384 / secp384r1 with R0VM-accelerated base and scalar fields.
//!
//! Equation: y² = x³ - 3·x + b. Cofactor 1.

use crate::{r0_fp, R0Backend, R0Config};
use ark_ec::{
    short_weierstrass::{Affine, Projective, SWCurveConfig},
    CurveConfig,
};
use ark_ff::{BigInt, Fp, Fp384};
use core::marker::PhantomData;

// --- Base field Fq (384 bits, N = 6) -----------------------------------------------------------

pub struct FqConfig;

impl R0Config<6> for FqConfig {
    // p = 2^384 - 2^128 - 2^96 + 2^32 - 1
    const MODULUS: BigInt<6> = BigInt::new([
        0x00000000FFFFFFFF,
        0xFFFFFFFF00000000,
        0xFFFFFFFFFFFFFFFE,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 6>, 6> = r0_fp!("19");
    const TWO_ADICITY: u32 = 1;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 6>, 6> = Fp(
        BigInt::new([
            0x00000000FFFFFFFE,
            0xFFFFFFFF00000000,
            0xFFFFFFFFFFFFFFFE,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ]),
        PhantomData,
    );
}

pub type Fq = Fp384<R0Backend<FqConfig, 6>>;

// --- Scalar field Fr ---------------------------------------------------------------------------

pub struct FrConfig;

impl R0Config<6> for FrConfig {
    // n = group order
    const MODULUS: BigInt<6> = BigInt::new([
        0xECEC196ACCC52973,
        0x581A0DB248B0A77A,
        0xC7634D81F4372DDF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 6>, 6> = r0_fp!("2");
    const TWO_ADICITY: u32 = 1;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 6>, 6> = Fp(
        BigInt::new([
            0xECEC196ACCC52972,
            0x581A0DB248B0A77A,
            0xC7634D81F4372DDF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ]),
        PhantomData,
    );
}

pub type Fr = Fp384<R0Backend<FrConfig, 6>>;

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
    const COEFF_A: Fq = r0_fp!(
        "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000fffffffc"
    );
    const COEFF_B: Fq = r0_fp!(
        "0xb3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef"
    );
    const GENERATOR: G1Affine = G1Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    type ZeroFlag = ();
}

pub const G_GENERATOR_X: Fq = r0_fp!(
    "0xaa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7"
);
pub const G_GENERATOR_Y: Fq = r0_fp!(
    "0x3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f"
);
