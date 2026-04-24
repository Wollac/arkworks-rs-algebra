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
    use ark_ff::{BigInt, Fp, Fp256};
    use ark_ff_risc0::{r0_fp, R0Backend, R0Config};

    pub struct FqConfig;

    impl R0Config<4> for FqConfig {
        const MODULUS: BigInt<4> = BigInt!(
            "115792089237316195423570985008687907853269984665640564039457584007908834671663"
        );
        const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("3");
        const TWO_ADICITY: u32 = 1;
        // -1 mod p (primitive 2nd root of unity).
        const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = r0_fp!("-1");
    }

    pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
}

pub use inner::{Fq, FqConfig};
