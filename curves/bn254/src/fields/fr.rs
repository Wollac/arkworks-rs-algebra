#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
use ark_ff::fields::{MontBackend as Backend, MontConfig as Config};
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
use ark_ff_risc0::{R0Backend as Backend, R0Config as Config};

use ark_ff::Fp256;

#[derive(Config)]
#[modulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[generator = "5"]
#[small_subgroup_base = "3"]
#[small_subgroup_power = "2"]
pub struct FrConfig;
pub type Fr = Fp256<Backend<FrConfig, 4>>;
