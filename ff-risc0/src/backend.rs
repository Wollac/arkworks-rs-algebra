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
        // SAFETY: modadd reads all inputs before writing, so out = a is allowed.
        unsafe { FieldFfi::modadd(ap, &b.0, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: modsub reads all inputs before writing, so out = a is allowed.
        unsafe { FieldFfi::modsub(ap, &b.0, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn double_in_place(a: &mut Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: a aliases both inputs and output; modadd reads before writing.
        unsafe { FieldFfi::modadd(ap, ap, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn neg_in_place(a: &mut Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: modsub reads all inputs before writing, so out = b is allowed.
        unsafe { FieldFfi::modsub(&Self::ZERO.0, ap, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: modmul reads all inputs before writing, so out = a is allowed.
        unsafe { FieldFfi::modmul(ap, &b.0, &P::MODULUS, ap) }
    }

    #[inline]
    fn sum_of_products<const T: usize>(a: &[Fp<Self, N>; T], b: &[Fp<Self, N>; T]) -> Fp<Self, N> {
        if T == 0 {
            return Self::ZERO;
        }

        let mut acc = MaybeUninit::<BigInt<N>>::uninit();
        let mut tmp = MaybeUninit::<BigInt<N>>::uninit();
        let acc_ptr = acc.as_mut_ptr();
        let tmp_ptr = tmp.as_mut_ptr();
        // SAFETY: modmul_unchecked and modadd_unchecked writes all limbs of out
        unsafe {
            // First iteration writes `a[0]*b[0]` straight into `acc`, skipping a redundant `0 + x`.
            FieldFfi::modmul_unchecked(&a[0].0, &b[0].0, &P::MODULUS, acc_ptr);
            for i in 1..T {
                FieldFfi::modmul_unchecked(&a[i].0, &b[i].0, &P::MODULUS, tmp_ptr);
                FieldFfi::modadd_unchecked(acc_ptr, tmp_ptr, &P::MODULUS, acc_ptr);
            }
        }
        // SAFETY: the T > 0 branch above always wrote `acc` via the first modmul.
        let acc = unsafe { acc.assume_init() };

        // Verify result is canonical (honest prover check)
        if acc >= P::MODULUS {
            non_canonical_sum_of_products();
        }
        Fp(acc, PhantomData)
    }

    #[inline(always)]
    fn square_in_place(a: &mut Fp<Self, N>) {
        let ap = ptr::from_mut(&mut a.0);
        // SAFETY: a aliases both inputs and output; modmul reads before writing.
        unsafe { FieldFfi::modmul(ap, ap, &P::MODULUS, ap) }
    }

    #[inline(always)]
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        if a.is_zero() {
            return None;
        }
        let mut out = MaybeUninit::<BigInt<N>>::uninit();
        // SAFETY: modinv writes all limbs of out; out does not alias a (separate stack slot).
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

/// Extension trait providing `from_sign_and_limbs` on `Fp<R0Backend<_, _>, _>`, mirroring the
/// inherent method that `MontBackend` already offers. Host code that calls
/// `Fp::from_sign_and_limbs(is_positive, limbs)` on an R0-backed field resolves through this
/// trait when it is in scope.
///
/// This is runtime-only (trait methods can't be `const fn` on stable); const callers should go
/// through [`crate::const_from_sign_and_limbs`] or the [`r0_fp!`](crate::r0_fp) macro instead.
pub trait R0Fp: Sized {
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
        if is_positive {
            if repr.0 >= P::MODULUS {
                repr.add_assign(&Self::ZERO); // FFI reduces
            }
        } else {
            repr.neg_in_place(); // modsub handles any magnitude
        }
        repr
    }
}
