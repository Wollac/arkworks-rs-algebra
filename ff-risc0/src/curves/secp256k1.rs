//! secp256k1 (Bitcoin / Ethereum) curve with R0VM-accelerated field arithmetic.
//!
//! Parameters match `ark_test_curves::secp256k1` bit-for-bit; the only difference is that the
//! base and scalar fields use [`crate::R0Backend`] instead of the default `MontBackend`, so
//! every field op routes through `risc0-bigint2` on the zkvm target.

use crate::{r0_fp, R0Backend, R0Config};
use ark_ec::{
    short_weierstrass::{Affine, Projective, SWCurveConfig},
    CurveConfig,
};
use ark_ff::{AdditiveGroup, BigInt, Field, Fp, Fp256};
use core::marker::PhantomData;

// --- Base field Fq ------------------------------------------------------------------------------

pub struct FqConfig;

impl R0Config<4> for FqConfig {
    // p = 2^256 - 2^32 - 977
    const MODULUS: BigInt<4> = BigInt::new([
        0xFFFFFFFEFFFFFC2F,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
    const TWO_ADICITY: u32 = 1;
    // -1 mod p (primitive 2nd root of unity).
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0xFFFFFFFEFFFFFC2E,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ]),
        PhantomData,
    );
}

pub type Fq = Fp256<R0Backend<FqConfig, 4>>;

// --- Scalar field Fr ----------------------------------------------------------------------------

pub struct FrConfig;

impl R0Config<4> for FrConfig {
    // n = group order of secp256k1.
    const MODULUS: BigInt<4> = BigInt::new([
        0xBFD25E8CD0364141,
        0xBAAEDCE6AF48A03B,
        0xFFFFFFFFFFFFFFFE,
        0xFFFFFFFFFFFFFFFF,
    ]);
    const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("7");
    const TWO_ADICITY: u32 = 6;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = Fp(
        BigInt::new([
            0x992F4B5402B052F2,
            0x98BDEAB680756045,
            0xDF9879A3FBC483A8,
            0x0C1DC060E7A91986,
        ]),
        PhantomData,
    );
}

pub type Fr = Fp256<R0Backend<FrConfig, 4>>;

// --- Curve --------------------------------------------------------------------------------------

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
    const COEFF_B: Fq = r0_fp!("7");
    const GENERATOR: Affine<Self> = Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    #[inline(always)]
    fn mul_by_a(_: Self::BaseField) -> Self::BaseField {
        Self::BaseField::ZERO
    }

    type ZeroFlag = ();
}

pub const G_GENERATOR_X: Fq =
    r0_fp!("55066263022277343669578718895168534326250603453777594175500187360389116729240");

pub const G_GENERATOR_Y: Fq =
    r0_fp!("32670510020758816978083085130507043184471273380659243275938904335757337482424");

// mul_by_a needs Field trait in scope via unsigned import.
const _: () = {
    fn _assert_field<F: Field>() {}
    fn _check() {
        _assert_field::<Fq>();
        _assert_field::<Fr>();
    }
};
