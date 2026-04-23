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
//! use ark_ff::{BigInt, Fp256};
//! use ark_ff_risc0::{R0Backend, R0Config, SqrtPrecomputation};
//!
//! pub struct Secp256k1FqConfig;
//! impl R0Config<4> for Secp256k1FqConfig {
//!     const MODULUS: BigInt<4> = BigInt::new([
//!         0xFFFFFFFEFFFFFC2F, 0xFFFFFFFFFFFFFFFF,
//!         0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF,
//!     ]);
//!     const GENERATOR: ark_ff::Fp<R0Backend<Self, 4>, 4> = /* ... */;
//!     const TWO_ADICITY: u32 = 1;
//!     const TWO_ADIC_ROOT_OF_UNITY: ark_ff::Fp<R0Backend<Self, 4>, 4> = /* ... */;
//! }
//!
//! pub type Fq = Fp256<R0Backend<Secp256k1FqConfig, 4>>;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

mod backend;
mod config;
mod ffi;

pub use backend::R0Backend;
pub use config::R0Config;
pub use ffi::FieldFfi;

pub mod curves;

/// Construct a plain-integer field element `Fp<R0Backend<_, N>, N>` from a numeric literal.
///
/// Accepts decimal, hex (`0x...`), octal (`0o...`), or binary (`0b...`) strings, same as
/// arkworks' [`ark_ff::BigInt!`]. The value must be strictly less than the field modulus; this
/// is the caller's responsibility (not checked at const time).
///
/// ```ignore
/// const COEFF_B: Fq = r0_fp!("7");
/// const G_X: Fq = r0_fp!("0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
/// ```
#[macro_export]
macro_rules! r0_fp {
    ($lit:expr) => {
        ::ark_ff::Fp(::ark_ff::BigInt!($lit), ::core::marker::PhantomData)
    };
}
