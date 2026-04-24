//! RISC Zero zkVM backend for `ark-ff`.
//!
//! Provides [`R0Backend`] and [`R0Config`], an alternative to the default `MontBackend` that
//! routes field arithmetic through `risc0-bigint2` accelerator syscalls when compiled for the
//! `zkvm` target. On other targets the same crate still builds and runs using a `num-bigint`
//! fallback so host-side tests keep working.
//!
//! # Representation
//!
//! Unlike `MontBackend`, elements are stored as plain integers in `[0, p)` rather than in
//! Montgomery form. This avoids per-op Montgomery reductions, which would dominate the cost of a
//! single `modmul` syscall. `from_bigint`/`into_bigint` are therefore zero-cost.
//!
//! # Supported widths
//!
//! Currently `N = 4` (256-bit) and `N = 6` (384-bit), matching the modular-arithmetic accelerator
//! blobs shipped by `risc0-bigint2`.
//!
//! # Example
//!
//! ```ignore
//! use ark_ff::{BigInt, Fp, Fp256};
//! use ark_ff_risc0::{r0_fp, R0Backend, R0Config};
//!
//! pub struct FqConfig;
//! impl R0Config<4> for FqConfig {
//!     const MODULUS: BigInt<4> = ark_ff::BigInt!(
//!         "115792089237316195423570985008687907853269984665640564039457584007908834671663"
//!     );
//!     const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
//!     const TWO_ADICITY: u32 = 1;
//!     const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = r0_fp!("-1");
//! }
//!
//! pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

mod backend;
mod config;
mod const_helpers;
mod ffi;

pub use backend::{R0Backend, R0Fp};
pub use config::R0Config;
pub use const_helpers::const_from_sign_and_limbs;
pub use ffi::FieldFfi;

/// Construct a plain-integer field element `Fp<R0Backend<_, N>, N>` from a numeric literal.
///
/// Accepts decimal, hex (`0x...`), octal (`0o...`), or binary (`0b...`) strings, optionally
/// preceded by `-`; same surface as arkworks' [`ark_ff::MontFp!`]. Negative literals yield
/// `MODULUS - |value|`. The unsigned magnitude must be strictly less than the modulus — enforced
/// at const-evaluation time.
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
