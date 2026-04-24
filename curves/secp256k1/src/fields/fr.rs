#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod inner {
    use ark_ff::fields::{Fp256, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "115792089237316195423570985008687907852837564279074904382605163141518161494337"]
    #[generator = "7"]
    #[small_subgroup_base = "3"]
    #[small_subgroup_power = "1"]
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
            "115792089237316195423570985008687907852837564279074904382605163141518161494337"
        );
        const GENERATOR: Fp<R0Backend<Self, 4>, 4> = r0_fp!("7");
        const TWO_ADICITY: u32 = 6;
        const TWO_ADIC_ROOT_OF_UNITY: Fp<R0Backend<Self, 4>, 4> = r0_fp!(
            "0x0C1DC060E7A91986DF9879A3FBC483A898BDEAB680756045992F4B5402B052F2"
        );
    }

    pub type Fr = Fp256<R0Backend<FrConfig, 4>>;
}

pub use inner::{Fr, FrConfig};
