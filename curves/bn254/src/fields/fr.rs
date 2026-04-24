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
    use ark_ff::Fp256;
    use ark_ff_risc0::{R0Backend, R0Config};

    #[derive(R0Config)]
    #[modulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
    #[generator = "5"]
    #[small_subgroup_base = "3"]
    #[small_subgroup_power = "2"]
    pub struct FrConfig;

    pub type Fr = Fp256<R0Backend<FrConfig, 4>>;
}

pub use inner::{Fr, FrConfig};
