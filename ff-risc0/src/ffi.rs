//! Modular-arithmetic dispatch shim.
//!
//! On `target_os = "zkvm"`, calls into `risc0-bigint2`'s checked entry points. These already
//! `assert!(result < modulus)` internally, so their outputs are canonical and no extra reduction
//! is needed on our side.
//!
//! On other targets, falls back to `num-bigint` so the crate still builds and host-side tests
//! can exercise the full `FpConfig` surface without a zkvm.
//!
//! The host fallback computes inverses via Fermat (`a^(p - 2) mod p`), which is only valid when
//! the modulus is prime. All supported fields satisfy this.

use ark_ff::BigInt;

/// Width-specific modular-arithmetic dispatch.
///
/// Implementation detail: exposed as `pub` only so that [`crate::R0Config`] can reference it in
/// its where clause. Users should not implement this trait directly; impls are provided by this
/// crate for [`BigInt<4>`] and [`BigInt<6>`] on the `zkvm` target, and for all [`BigInt<N>`] on
/// the host fallback.
///
/// Safety note for zkvm: `out` in [`FieldFfi::modinv`] must not alias `a` per the multi-constraint
/// structure of the modinv circuit. The other ops are allowed to alias (risc0-bigint2 reads all
/// inputs before writing).
pub trait FieldFfi: Sized {
    fn modadd(a: &Self, b: &Self, m: &Self, out: &mut Self);
    fn modsub(a: &Self, b: &Self, m: &Self, out: &mut Self);
    fn modmul(a: &Self, b: &Self, m: &Self, out: &mut Self);
    /// Must not be called with `a == 0`. Prime modulus assumed (for the host fallback).
    fn modinv(a: &Self, m: &Self, out: &mut Self);
}

#[cfg(target_os = "zkvm")]
mod zkvm_impl {
    use super::*;
    use risc0_bigint2::field::{
        modadd_256, modadd_384, modinv_256, modinv_384, modmul_256, modmul_384, modsub_256,
        modsub_384,
    };

    /// Reinterpret a `&BigInt<N>` as `&[u32; 2N]` for the risc0-bigint2 FFI.
    ///
    /// Safe on little-endian targets (zkvm is RISC-V LE): byte layout is identical and u64
    /// alignment subsumes u32 alignment.
    ///
    /// # Safety
    /// - `N2` must equal `2 * N`.
    /// - Target must be little-endian.
    #[inline(always)]
    unsafe fn as_u32<const N: usize, const N2: usize>(x: &BigInt<N>) -> &[u32; N2] {
        unsafe { &*(core::ptr::addr_of!(x.0).cast::<[u32; N2]>()) }
    }

    #[inline(always)]
    unsafe fn as_u32_mut<const N: usize, const N2: usize>(x: &mut BigInt<N>) -> &mut [u32; N2] {
        unsafe { &mut *(core::ptr::addr_of_mut!(x.0).cast::<[u32; N2]>()) }
    }

    impl FieldFfi for BigInt<4> {
        #[inline(always)]
        fn modadd(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            unsafe { modadd_256(as_u32(a), as_u32(b), as_u32(m), as_u32_mut(out)) }
        }
        #[inline(always)]
        fn modsub(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            unsafe { modsub_256(as_u32(a), as_u32(b), as_u32(m), as_u32_mut(out)) }
        }
        #[inline(always)]
        fn modmul(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            unsafe { modmul_256(as_u32(a), as_u32(b), as_u32(m), as_u32_mut(out)) }
        }
        #[inline(always)]
        fn modinv(a: &Self, m: &Self, out: &mut Self) {
            unsafe { modinv_256(as_u32(a), as_u32(m), as_u32_mut(out)) }
        }
    }

    impl FieldFfi for BigInt<6> {
        #[inline(always)]
        fn modadd(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            unsafe { modadd_384(as_u32(a), as_u32(b), as_u32(m), as_u32_mut(out)) }
        }
        #[inline(always)]
        fn modsub(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            unsafe { modsub_384(as_u32(a), as_u32(b), as_u32(m), as_u32_mut(out)) }
        }
        #[inline(always)]
        fn modmul(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            unsafe { modmul_384(as_u32(a), as_u32(b), as_u32(m), as_u32_mut(out)) }
        }
        #[inline(always)]
        fn modinv(a: &Self, m: &Self, out: &mut Self) {
            unsafe { modinv_384(as_u32(a), as_u32(m), as_u32_mut(out)) }
        }
    }
}

#[cfg(not(target_os = "zkvm"))]
mod host_impl {
    use super::*;
    use ark_std::vec::Vec;
    use num_bigint::BigUint;

    const MAX_BYTES: usize = 8 * 16;

    fn to_biguint<const N: usize>(x: &BigInt<N>) -> BigUint {
        let mut bytes = [0u8; MAX_BYTES];
        for i in 0..N {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&x.0[i].to_le_bytes());
        }
        BigUint::from_bytes_le(&bytes[..N * 8])
    }

    fn from_biguint<const N: usize>(x: &BigUint, out: &mut BigInt<N>) {
        let bytes_le: Vec<u8> = x.to_bytes_le();
        let mut buf = [0u8; MAX_BYTES];
        buf[..bytes_le.len()].copy_from_slice(&bytes_le);
        for i in 0..N {
            let mut chunk = [0u8; 8];
            chunk.copy_from_slice(&buf[i * 8..(i + 1) * 8]);
            out.0[i] = u64::from_le_bytes(chunk);
        }
    }

    impl<const N: usize> FieldFfi for BigInt<N> {
        fn modadd(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            let (a, b, m) = (to_biguint(a), to_biguint(b), to_biguint(m));
            from_biguint(&((a + b) % &m), out);
        }
        fn modsub(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            let (a, b, m) = (to_biguint(a), to_biguint(b), to_biguint(m));
            from_biguint(&((&a + &m - b) % &m), out);
        }
        fn modmul(a: &Self, b: &Self, m: &Self, out: &mut Self) {
            let (a, b, m) = (to_biguint(a), to_biguint(b), to_biguint(m));
            from_biguint(&((a * b) % &m), out);
        }
        fn modinv(a: &Self, m: &Self, out: &mut Self) {
            // Fermat: a^(p - 2) mod p for prime p.
            let a_u = to_biguint(a);
            let m_u = to_biguint(m);
            let exp = &m_u - BigUint::from(2u32);
            from_biguint(&a_u.modpow(&exp, &m_u), out);
        }
    }
}
