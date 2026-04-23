//! BLS12-381 G1 with R0VM-accelerated base and scalar fields.
//!
//! Only the base field `Fq`, the scalar field `Fr`, and the G1 curve are provided. The G2 curve
//! and the pairing live over `Fq2` (a quadratic extension), which arkworks expresses via
//! `Fp2Config`. Extending this integration to G2/pairings requires supplying an extension-field
//! layer and is deliberately out of scope here.

use crate::{r0_fp, R0Backend, R0Config};
use ark_ec::{
    short_weierstrass::{Affine, Projective, SWCurveConfig},
    CurveConfig,
};
use ark_ff::{AdditiveGroup, BigInt, Fp, Fp256, Fp384};
use core::marker::PhantomData;

// --- Base field Fq (381 bits, N = 6) -----------------------------------------------------------

pub struct FqConfig;

impl R0Config<6> for FqConfig {
    const MODULUS: BigInt<6> = BigInt::new([
        0xB9FEFFFFFFFFAAAB,
        0x1EABFFFEB153FFFF,
        0x6730D2A0F6B0F624,
        0x64774B84F38512BF,
        0x4B1BA7B6434BACD7,
        0x1A0111EA397FE69A,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 6>, 6> = r0_fp!("2");
    const TWO_ADICITY: u32 = 1;
    // p - 1 (primitive 2nd root of unity).
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 6>, 6> = Fp(
        BigInt::new([
            0xB9FEFFFFFFFFAAAA,
            0x1EABFFFEB153FFFF,
            0x6730D2A0F6B0F624,
            0x64774B84F38512BF,
            0x4B1BA7B6434BACD7,
            0x1A0111EA397FE69A,
        ]),
        PhantomData,
    );
}

pub type Fq = Fp384<R0Backend<FqConfig, 6>>;

// --- Scalar field Fr (255 bits, N = 4) ---------------------------------------------------------

pub struct FrConfig;

impl R0Config<4> for FrConfig {
    // r = 52435875175126190479447740508185965837690552500527637822603658699938581184513
    const MODULUS: BigInt<4> = BigInt::new([
        0xFFFFFFFF00000001,
        0x53BDA402FFFE5BFE,
        0x3339D80809A1D805,
        0x73EDA753299D7D48,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("7");
    const TWO_ADICITY: u32 = 32;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0x3829971F439F0D2B,
            0xB63683508C2280B9,
            0xD09B681922C813B4,
            0x16A2A19EDFE81F20,
        ]),
        PhantomData,
    );
}

pub type Fr = Fp256<R0Backend<FrConfig, 4>>;

// --- G1 -----------------------------------------------------------------------------------------

pub type G1Affine = Affine<Config>;
pub type G1Projective = Projective<Config>;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Config;

impl CurveConfig for Config {
    type BaseField = Fq;
    type ScalarField = Fr;
    // (x - 1)^2 / 3 = 76329603384216526031706109802092473003
    const COFACTOR: &'static [u64] = &[0x8C00AAAB0000AAAB, 0x396C8C005555E156];
    const COFACTOR_INV: Fr =
        r0_fp!("52435875175126190458656871551744051925719901746859129887267498875565241663483");
}

impl SWCurveConfig for Config {
    const COEFF_A: Fq = Fq::ZERO;
    const COEFF_B: Fq = r0_fp!("4");
    const GENERATOR: G1Affine = G1Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    #[inline(always)]
    fn mul_by_a(_: Self::BaseField) -> Self::BaseField {
        Self::BaseField::ZERO
    }

    type ZeroFlag = ();
}

pub const G_GENERATOR_X: Fq = r0_fp!(
    "3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507"
);

pub const G_GENERATOR_Y: Fq = r0_fp!(
    "1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569"
);
