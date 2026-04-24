#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod inner {
    use ark_ff::fields::{Fp256, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
    #[generator = "5"]
    #[small_subgroup_base = "3"]
    #[small_subgroup_power = "2"]
    pub struct FrConfig;
    pub type Fr = Fp256<MontBackend<FrConfig, 4>>;
}

#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
mod inner {
    use ark_ff::{BigInt, Fp, Fp256};
    use ark_ff_risc0::{r0_fp, R0Backend, R0Config};

    pub struct FrConfig;

    impl R0Config<4> for FrConfig {
        const MODULUS: BigInt<4> = BigInt!(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        );
        const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("5");
        const TWO_ADICITY: u32 = 28;
        const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = r0_fp!(
            "0x2A3C09F0A58A7E8500E0A7EB8EF62ABC402D111E41112ED49BD61B6E725B19F0"
        );
    }

    pub type Fr = Fp256<R0Backend<FrConfig, 4>>;
}

pub use inner::{Fr, FrConfig};
