#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
use ark_ff::fields::{MontBackend as Backend, MontConfig as Config};
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
use ark_ff_risc0::{R0Backend as Backend, R0Config as Config};

use ark_ff::Fp256;

#[derive(Config)]
#[modulus = "115792089237316195423570985008687907853269984665640564039457584007908834671663"]
#[generator = "3"]
#[small_subgroup_base = "3"]
#[small_subgroup_power = "1"]
pub struct FqConfig;
pub type Fq = Fp256<Backend<FqConfig, 4>>;
