//! Const-evaluable helpers for constructing plain-integer `Fp<R0Backend<_, _>, _>` values.

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

// Little-endian limbs: compare from the most-significant limb down.
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

/// Builds a plain-integer `Fp<R0Backend<P, N>, N>` from a sign-and-limbs pair in `const`
/// context.
///
/// Expanded by the [`r0_fp!`](crate::r0_fp) macro; not intended for direct use. The unsigned
/// magnitude (`limbs` zero-padded to `N`) must be strictly less than `P::MODULUS`, otherwise
/// const-evaluation panics. Negative literals return `MODULUS - |value|`, with `-0`
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
