//! User-facing field-parameter trait. Analog of `MontConfig` for the R0VM backend.
//!
//! Implement this to define a field whose arithmetic routes through `risc0-bigint2`.

use crate::{ffi::FieldFfi, R0Backend};
use ark_ff::{BigInt, Fp, SqrtPrecomputation};

/// Parameters of a prime field whose arithmetic is executed by the R0VM backend.
///
/// Constants carry the same semantics as in [`ark_ff::FpConfig`] but are stored as plain
/// integers in `[0, p)` rather than in Montgomery form.
///
/// `N` is the limb count in `u64` units. Currently only `N = 4` (256-bit) and `N = 6` (384-bit)
/// are supported on the `zkvm` target, enforced by the `BigInt<N>: FieldFfi` bound.
pub trait R0Config<const N: usize>: 'static + Send + Sync + Sized
where
    BigInt<N>: FieldFfi,
{
    /// The modulus `p` of the field.
    const MODULUS: BigInt<N>;

    /// A multiplicative generator of the field of order `p - 1`.
    const GENERATOR: Fp<R0Backend<Self, N>, N>;

    /// `s` such that `p - 1 = 2^s * t` for some odd integer `t`.
    const TWO_ADICITY: u32;

    /// `GENERATOR^t mod p`, i.e. a primitive `2^s`-th root of unity.
    const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, N>, N>;

    /// Optional base `b` for a size-`b^k` multiplicative subgroup.
    const SMALL_SUBGROUP_BASE: Option<u32> = None;

    /// The exponent `k` in the previous optional.
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;

    /// Root used for mixed-radix FFT when `SMALL_SUBGROUP_BASE` is set.
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Fp<R0Backend<Self, N>, N>> = None;

    /// Precomputed material for square roots. `None` disables `sqrt`.
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Fp<R0Backend<Self, N>, N>>> = None;
}
