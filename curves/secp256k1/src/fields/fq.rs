#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod inner {
    use ark_ff::fields::{Fp256, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "115792089237316195423570985008687907853269984665640564039457584007908834671663"]
    #[generator = "3"]
    #[small_subgroup_base = "3"]
    #[small_subgroup_power = "1"]
    pub struct FqConfig;
    pub type Fq = Fp256<MontBackend<FqConfig, 4>>;
}

#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
mod inner {
    use ark_ff::Fp256;
    use ark_ff_risc0::{R0Backend, R0Config};

    #[derive(R0Config)]
    #[modulus = "115792089237316195423570985008687907853269984665640564039457584007908834671663"]
    #[generator = "3"]
    #[small_subgroup_base = "3"]
    #[small_subgroup_power = "1"]
    pub struct FqConfig;

    pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
}

pub use inner::{Fq, FqConfig};
