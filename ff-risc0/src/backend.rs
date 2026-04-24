//! [`FpConfig`] implementation that routes arithmetic through the R0VM backend.

use ark_ff::{AdditiveGroup, BigInt, Fp, FpConfig, SqrtPrecomputation, Zero};
use core::{marker::PhantomData, mem::MaybeUninit, ops::AddAssign, ptr};

use crate::{config::R0Config, const_helpers::const_sub_with_borrow, ffi::FieldFfi};

#[cold]
#[inline(never)]
fn non_canonical_sum_of_products() -> ! {
    panic!("sum_of_products: result is non-canonical (malformed risc0-bigint2 proof)")
}

/// R0VM backend for [`ark_ff::FpConfig`].
///
/// Parameterized by a user-provided [`R0Config`]. Field elements are stored as plain
/// integers in `[0, p)`; arithmetic dispatches to `risc0-bigint2` on the `zkvm` target
/// and to `num-bigint` on the host.
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
    const NEG_ONE: Fp<Self, N> = Fp(
        const_sub_with_borrow(P::MODULUS, &Self::ONE.0).0,
        PhantomData,
    );
    const TWO_ADICITY: u32 = P::TWO_ADICITY;
    const TWO_ADIC_ROOT_OF_UNITY: Fp<Self, N> = P::TWO_ADIC_ROOT_OF_UNITY;
    const SMALL_SUBGROUP_BASE: Option<u32> = P::SMALL_SUBGROUP_BASE;
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = P::SMALL_SUBGROUP_BASE_ADICITY;
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Fp<Self, N>> = P::LARGE_SUBGROUP_ROOT_OF_UNITY;
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Fp<Self, N>>> = P::SQRT_PRECOMP;

    #[inline(always)]
    fn add_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: `out` aliases `a`; see `FieldFfi::modadd` safety contract.
        unsafe { FieldFfi::modadd(ap, &b.0, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: `out` aliases `a`; see `FieldFfi::modsub` safety contract.
        unsafe { FieldFfi::modsub(ap, &b.0, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn double_in_place(a: &mut Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: `out` aliases both inputs; see `FieldFfi::modadd` safety contract.
        unsafe { FieldFfi::modadd(ap, ap, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn neg_in_place(a: &mut Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: `out` aliases `b`; see `FieldFfi::modsub` safety contract.
        unsafe { FieldFfi::modsub(&Self::ZERO.0, ap, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: `out` aliases `a`; see `FieldFfi::modmul` safety contract.
        unsafe { FieldFfi::modmul(ap, &b.0, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn sum_of_products<const T: usize>(a: &[Fp<Self, N>; T], b: &[Fp<Self, N>; T]) -> Fp<Self, N> {
        if T == 0 {
            return Self::ZERO;
        }

        let mut acc = MaybeUninit::<BigInt<N>>::uninit();
        let mut tmp = MaybeUninit::<BigInt<N>>::uninit();
        let acc_ptr = acc.as_mut_ptr();
        let tmp_ptr = tmp.as_mut_ptr();
        // SAFETY: `modmul_unchecked` and `modadd_unchecked` write all limbs of `out`;
        // the first `modmul_unchecked` initialises `acc` before any `modadd_unchecked` reads it.
        unsafe {
            // Skip a redundant `0 + x` by writing `a[0] * b[0]` straight into `acc`.
            FieldFfi::modmul_unchecked(&a[0].0, &b[0].0, &P::MODULUS, acc_ptr);
            for i in 1..T {
                FieldFfi::modmul_unchecked(&a[i].0, &b[i].0, &P::MODULUS, tmp_ptr);
                FieldFfi::modadd_unchecked(acc_ptr, tmp_ptr, &P::MODULUS, acc_ptr);
            }
        }
        // SAFETY: the `T > 0` branch above writes `acc` via the first `modmul_unchecked`.
        let acc = unsafe { acc.assume_init() };

        // Honest-prover check: unchecked variants omit the internal `result < modulus` assert.
        if acc >= P::MODULUS {
            non_canonical_sum_of_products();
        }
        Fp(acc, PhantomData)
    }

    #[inline(always)]
    fn square_in_place(a: &mut Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: `out` aliases both inputs; see `FieldFfi::modmul` safety contract.
        unsafe { FieldFfi::modmul(ap, ap, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        if a.is_zero() {
            return None;
        }
        let mut out = MaybeUninit::<BigInt<N>>::uninit();
        // SAFETY: `modinv` writes all limbs of `out`; `out` does not alias `a` (distinct slot).
        unsafe {
            FieldFfi::modinv(&a.0, &P::MODULUS, out.as_mut_ptr());
            Some(Fp(out.assume_init(), PhantomData))
        }
    }

    #[inline(always)]
    fn from_bigint(r: BigInt<N>) -> Option<Fp<Self, N>> {
        if r >= P::MODULUS {
            None
        } else {
            Some(Fp(r, PhantomData))
        }
    }

    #[inline(always)]
    fn into_bigint(r: Fp<Self, N>) -> BigInt<N> {
        r.0
    }
}

/// Extension trait providing `from_sign_and_limbs` on `Fp<R0Backend<_, _>, _>`.
///
/// Mirrors the inherent method on `MontBackend`-backed fields. Import this trait to resolve
/// `Fp::from_sign_and_limbs(is_positive, limbs)` calls on an R0-backed field. For `const`
/// callers, use [`crate::const_from_sign_and_limbs`] or the [`r0_fp!`](crate::r0_fp) macro
/// instead.
pub trait R0Fp: Sized {
    /// Builds a field element from a sign bit and little-endian `u64` magnitude limbs.
    fn from_sign_and_limbs(is_positive: bool, limbs: &[u64]) -> Self;
}

impl<P, const N: usize> R0Fp for Fp<R0Backend<P, N>, N>
where
    P: R0Config<N>,
    BigInt<N>: FieldFfi,
{
    #[inline]
    fn from_sign_and_limbs(is_positive: bool, limbs: &[u64]) -> Self {
        assert!(limbs.len() <= N);

        let mut repr = Self::ZERO;
        repr.0 .0[..limbs.len()].copy_from_slice(limbs);
        if !is_positive {
            repr.neg_in_place(); // modsub handles any magnitude
        } else if repr.0 >= P::MODULUS {
            repr.add_assign(&Self::ZERO); // FFI reduces
        }
        repr
    }
}
