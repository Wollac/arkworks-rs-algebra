//! RISC Zero zkVM backend for `ark-ff`.
//!
//! Provides [`R0Backend`] and [`R0Config`], an alternative to `MontBackend` that routes
//! field arithmetic through `risc0-bigint2` accelerator syscalls on the `zkvm` target.
//! On other targets the same crate builds against a `num-bigint` fallback, so host-side
//! tests keep working.
//!
//! # Representation
//!
//! Elements are stored as plain integers in `[0, p)` rather than in Montgomery form.
//! This avoids per-op Montgomery reductions, which would dominate the cost of a single
//! `modmul` syscall. `from_bigint` and `into_bigint` are therefore zero-cost.
//!
//! # Supported widths
//!
//! `N = 4` (256-bit) and `N = 6` (384-bit), matching the modular-arithmetic accelerator
//! blobs shipped by `risc0-bigint2`.
//!
//! # Example
//!
//! ```ignore
//! use ark_ff::Fp256;
//! use ark_ff_risc0::{R0Backend, R0Config};
//!
//! #[derive(R0Config)]
//! #[modulus = "115792089237316195423570985008687907853269984665640564039457584007908834671663"]
//! #[generator = "3"]
//! pub struct FqConfig;
//!
//! pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
//! ```
//!
//! The derive computes `TWO_ADICITY` and `TWO_ADIC_ROOT_OF_UNITY` for you. Manual impls of
//! [`R0Config`] are still supported.

#![cfg_attr(not(feature = "std"), no_std)]

mod backend;
mod config;
mod const_helpers;
mod ffi;

pub use ark_ff_risc0_macros::R0Config;
pub use backend::{R0Backend, R0Fp};
pub use config::R0Config;
pub use const_helpers::const_from_sign_and_limbs;
pub use ffi::FieldFfi;

/// Constructs a plain-integer field element `Fp<R0Backend<_, N>, N>` from a numeric literal.
///
/// Accepts decimal, hex (`0x...`), octal (`0o...`), or binary (`0b...`) strings, optionally
/// preceded by `-`; same surface as arkworks' [`ark_ff::MontFp!`]. Negative literals yield
/// `MODULUS - |value|`. The unsigned magnitude must be strictly less than the modulus;
/// this is checked at const-evaluation time.
///
/// # Examples
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
