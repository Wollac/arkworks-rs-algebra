#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
use ark_ff::fields::{MontBackend as Backend, MontConfig as Config};
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
use ark_ff_risc0::{R0Backend as Backend, R0Config as Config};

use ark_ff::Fp256;

#[derive(Config)]
#[modulus = "115792089237316195423570985008687907852837564279074904382605163141518161494337"]
#[generator = "7"]
#[small_subgroup_base = "3"]
#[small_subgroup_power = "1"]
pub struct FrConfig;
pub type Fr = Fp256<Backend<FrConfig, 4>>;
