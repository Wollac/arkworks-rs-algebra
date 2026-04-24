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
    use ark_ff::{BigInt, Fp, Fp256};
    use ark_ff_risc0::{r0_fp, R0Backend, R0Config};

    pub struct FqConfig;

    impl R0Config<4> for FqConfig {
        const MODULUS: BigInt<4> = BigInt!(
            "21888242871839275222246405745257275088696311157297823662689037894645226208583"
        );
        const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
        const TWO_ADICITY: u32 = 1;
        const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = r0_fp!("-1");
    }

    pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
}

pub use inner::{Fq, FqConfig};
