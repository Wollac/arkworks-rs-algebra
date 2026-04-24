#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod inner {
    use ark_ff::fields::{Fp256, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "21888242871839275222246405745257275088696311157297823662689037894645226208583"]
    #[generator = "3"]
    pub struct FqConfig;
    pub type Fq = Fp256<MontBackend<FqConfig, 4>>;
}

#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
mod inner {
    use ark_ff::Fp256;
    use ark_ff_risc0::{R0Backend, R0Config};

    #[derive(R0Config)]
    #[modulus = "21888242871839275222246405745257275088696311157297823662689037894645226208583"]
    #[generator = "3"]
    pub struct FqConfig;

    pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
}

pub use inner::{Fq, FqConfig};
