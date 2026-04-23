//! Curve configurations that use [`crate::R0Backend`] as the field backend.
//!
//! These are thin `SWCurveConfig` impls that reuse arkworks' stock Jacobian group arithmetic
//! while every underlying field operation routes through `risc0-bigint2` on the zkvm target.

pub mod bls12_381;
pub mod bn254;
pub mod grumpkin;
pub mod secp256k1;
pub mod secp256r1;
pub mod secp384r1;
