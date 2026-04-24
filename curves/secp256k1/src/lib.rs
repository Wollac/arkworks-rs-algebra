#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    warnings,
    unused,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

//! This library implements the secp256k1 curve.
//! Source: <https://en.bitcoin.it/wiki/Secp256k1>
//!
//! Curve information:
//! * Base field: q =
//!   115792089237316195423570985008687907853269984665640564039457584007908834671663
//! * Scalar field: r =
//!   115792089237316195423570985008687907852837564279074904382605163141518161494337
//! * Curve equation: y^2 = x^3 + 7

#[cfg(feature = "r1cs")]
pub mod constraints;
mod curves;
mod fields;

pub use curves::*;
pub use fields::*;

// Target-conditional alias: `ark_ff::MontFp!` on host, `ark_ff_risc0::r0_fp!` on zkvm.
#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
pub(crate) use ark_ff::MontFp;
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
pub(crate) use ark_ff_risc0::r0_fp as MontFp;
