#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    unused,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility
)]
#![doc = include_str!("../README.md")]

mod backend;
mod config;
mod const_helpers;
mod ffi;

pub use ark_ff_risc0_macros::R0Config;
pub use backend::{R0Backend, R0Fp};
pub use config::R0Config;
#[doc(hidden)]
pub use const_helpers::const_from_sign_and_limbs;
#[doc(hidden)]
pub use ffi::FieldFfi;

/// Constructs a plain-integer field element `Fp<R0Backend<_, N>, N>` from a numeric literal.
///
/// Accepts decimal, hex (`0x...`), octal (`0o...`), or binary (`0b...`) strings, optionally
/// preceded by `-`; same surface as arkworks' [`ark_ff::MontFp!`]. Negative literals yield
/// `MODULUS - |value|`. The unsigned magnitude must be strictly less than the modulus;
/// this is checked at const-evaluation time.
///
/// # Example
///
/// ```ignore
/// const COEFF_B: Fq = r0_fp!("7");
/// const NEG_ONE: Fq = r0_fp!("-1");
/// const G_X: Fq = r0_fp!("0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
/// ```
#[macro_export]
macro_rules! r0_fp {
    ($lit:expr) => {{
        let (is_positive, limbs) = ::ark_ff::ark_ff_macros::to_sign_and_limbs!($lit);
        $crate::const_from_sign_and_limbs(is_positive, &limbs)
    }};
}
