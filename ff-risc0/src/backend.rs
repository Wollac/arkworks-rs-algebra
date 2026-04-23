//! [`FpConfig`] implementation that routes arithmetic through the R0VM backend.

use crate::{config::R0Config, ffi::FieldFfi};
use ark_ff::{BigInt, BigInteger, Fp, FpConfig, SqrtPrecomputation};
use core::marker::PhantomData;

/// `x - 1` as a `BigInt<N>`. Undefined for `x == 0`; never called on zero (modulus > 1).
const fn sub_one<const N: usize>(x: BigInt<N>) -> BigInt<N> {
    let mut limbs = x.0;
    let mut i = 0;
    while i < N {
        if limbs[i] > 0 {
            limbs[i] -= 1;
            return BigInt::new(limbs);
        }
        limbs[i] = u64::MAX;
        i += 1;
    }
    BigInt::new(limbs)
}

/// R0VM backend for [`ark_ff::FpConfig`].
///
/// Parametrised by a user-provided [`R0Config`]. Field elements are stored as plain integers in
/// `[0, p)`; arithmetic dispatches to `risc0-bigint2` on the `zkvm` target and to `num-bigint` on
/// the host.
pub struct R0Backend<P, const N: usize>(PhantomData<P>)
where
    P: R0Config<N>,
    BigInt<N>: FieldFfi;

impl<P, const N: usize> FpConfig<N> for R0Backend<P, N>
where
    P: R0Config<N>,
    BigInt<N>: FieldFfi,
{
    const MODULUS: BigInt<N> = P::MODULUS;
    const GENERATOR: Fp<Self, N> = P::GENERATOR;
    const ZERO: Fp<Self, N> = Fp(BigInt::zero(), PhantomData);
    const ONE: Fp<Self, N> = Fp(BigInt::one(), PhantomData);
    const NEG_ONE: Fp<Self, N> = Fp(sub_one::<N>(P::MODULUS), PhantomData);
    const TWO_ADICITY: u32 = P::TWO_ADICITY;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<Self, N> = P::TWO_ADIC_ROOT_OF_UNITY;
    const SMALL_SUBGROUP_BASE: Option<u32> = P::SMALL_SUBGROUP_BASE;
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = P::SMALL_SUBGROUP_BASE_ADICITY;
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Fp<Self, N>> = P::LARGE_SUBGROUP_ROOT_OF_UNITY;
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Fp<Self, N>>> = P::SQRT_PRECOMP;

    #[inline]
    fn add_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let m = P::MODULUS;
        let a_in = a.0;
        FieldFfi::modadd(&a_in, &b.0, &m, &mut a.0);
    }

    #[inline]
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let m = P::MODULUS;
        let a_in = a.0;
        FieldFfi::modsub(&a_in, &b.0, &m, &mut a.0);
    }

    #[inline]
    fn double_in_place(a: &mut Fp<Self, N>) {
        let m = P::MODULUS;
        let a_in = a.0;
        FieldFfi::modadd(&a_in, &a_in, &m, &mut a.0);
    }

    #[inline]
    fn neg_in_place(a: &mut Fp<Self, N>) {
        // `p - a` via plain BigInt subtraction bypasses the modsub syscall. Result is canonical
        // by construction since 0 <= a < p.
        if a.0 != BigInt::zero() {
            let mut tmp = P::MODULUS;
            tmp.sub_with_borrow(&a.0);
            a.0 = tmp;
        }
    }

    #[inline]
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let m = P::MODULUS;
        let a_in = a.0;
        FieldFfi::modmul(&a_in, &b.0, &m, &mut a.0);
    }

    #[inline]
    fn square_in_place(a: &mut Fp<Self, N>) {
        let m = P::MODULUS;
        let a_in = a.0;
        FieldFfi::modmul(&a_in, &a_in, &m, &mut a.0);
    }

    #[inline]
    fn sum_of_products<const T: usize>(a: &[Fp<Self, N>; T], b: &[Fp<Self, N>; T]) -> Fp<Self, N> {
        // risc0-bigint2 has no fused MAC syscall; naive loop.
        let m = P::MODULUS;
        let mut acc = BigInt::zero();
        let mut tmp = BigInt::zero();
        for i in 0..T {
            FieldFfi::modmul(&a[i].0, &b[i].0, &m, &mut tmp);
            let acc_in = acc;
            FieldFfi::modadd(&acc_in, &tmp, &m, &mut acc);
        }
        Fp(acc, PhantomData)
    }

    #[inline]
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        if a.0 == BigInt::zero() {
            return None;
        }
        let m = P::MODULUS;
        let mut out = BigInt::zero();
        FieldFfi::modinv(&a.0, &m, &mut out);
        Some(Fp(out, PhantomData))
    }

    #[inline]
    fn from_bigint(r: BigInt<N>) -> Option<Fp<Self, N>> {
        if r >= P::MODULUS {
            None
        } else {
            Some(Fp(r, PhantomData))
        }
    }

    #[inline]
    fn into_bigint(r: Fp<Self, N>) -> BigInt<N> {
        r.0
    }
}
