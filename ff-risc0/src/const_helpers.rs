//! Const-evaluable helpers used when constructing plain-form `Fp<R0Backend<_, _>, _>` values
//! from numeric literals.
//!
//! The only item exposed outside the crate is [`const_from_sign_and_limbs`], which the
//! [`r0_fp!`](crate::r0_fp) macro expands into via `$crate::const_from_sign_and_limbs`. Everything
//! else in this module is `pub(crate)` — small BigInt utilities needed to implement both the
//! literal conversion and the `NEG_ONE` constant in [`crate::R0Backend`]'s
//! [`FpConfig`](ark_ff::FpConfig) impl.

use ark_ff::{BigInt, Fp};
use core::marker::PhantomData;

use crate::{config::R0Config, ffi::FieldFfi, R0Backend};

pub(crate) const fn const_is_zero<const N: usize>(a: &BigInt<N>) -> bool {
    ark_ff::const_for!((i in 0..N) {
        if a.0[i] != 0 {
            return false;
        }
    });
    true
}

/// `a < b` on little-endian limbs: compare from the most-significant limb down.
pub(crate) const fn const_lt<const N: usize>(a: &BigInt<N>, b: &BigInt<N>) -> bool {
    let mut i = N;
    while i > 0 {
        i -= 1;
        if a.0[i] != b.0[i] {
            return a.0[i] < b.0[i];
        }
    }
    false
}

pub(crate) const fn const_sub_with_borrow<const N: usize>(
    mut a: BigInt<N>,
    b: &BigInt<N>,
) -> (BigInt<N>, bool) {
    let mut borrow: u64 = 0;
    ark_ff::const_for!((i in 0..N) {
        let (r1, b1) = a.0[i].overflowing_sub(b.0[i]);
        let (r2, b2) = r1.overflowing_sub(borrow);
        a.0[i] = r2;
        borrow = (b1 as u64) | (b2 as u64);
    });
    (a, borrow != 0)
}

/// Const-evaluable construction of a plain-integer `Fp<R0Backend<P, N>, N>` from a sign-and-limbs
/// pair. Sibling of the runtime [`R0Fp::from_sign_and_limbs`](crate::R0Fp::from_sign_and_limbs)
/// trait method; this variant is usable in `const` contexts (macros, const fns) but is strict
/// about its input range.
///
/// Not intended for direct use; call the [`r0_fp!`](crate::r0_fp) macro, which expands a numeric
/// literal into `(is_positive, &limbs)` via `ark_ff_macros::to_sign_and_limbs!` and forwards here.
///
/// The unsigned magnitude (`limbs` zero-padded to `N`) must be strictly less than the modulus;
/// otherwise const-evaluation panics. Negative literals return `MODULUS - |value|`, with `-0`
/// normalised to `0`.
#[doc(hidden)]
pub const fn const_from_sign_and_limbs<P, const N: usize>(
    is_positive: bool,
    limbs: &[u64],
) -> Fp<R0Backend<P, N>, N>
where
    P: R0Config<N>,
    BigInt<N>: FieldFfi,
{
    assert!(limbs.len() <= N);
    let mut repr = BigInt([0; N]);
    ark_ff::const_for!((i in 0..(limbs.len())) {
        repr.0[i] = limbs[i];
    });
    if const_is_zero(&repr) {
        return Fp(repr, PhantomData);
    }

    assert!(const_lt(&repr, &P::MODULUS), "literal >= modulus");
    if is_positive {
        Fp(repr, PhantomData)
    } else {
        Fp(const_sub_with_borrow(repr, &P::MODULUS).0, PhantomData)
    }
}
