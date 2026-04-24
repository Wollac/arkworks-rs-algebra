#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
use ark_ff::fields::{MontBackend as Backend, MontConfig as Config};
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
use ark_ff_risc0::{R0Backend as Backend, R0Config as Config};

use ark_ff::Fp256;

#[derive(Config)]
#[modulus = "21888242871839275222246405745257275088696311157297823662689037894645226208583"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp256<Backend<FqConfig, 4>>;
