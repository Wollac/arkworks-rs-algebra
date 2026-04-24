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
//!
//! # Pointer-based FFI
//!
//! The trait uses raw pointers so that in-place callers (`a += b`, `a *= b`, `-a`, etc.) can
//! pass `a` as both input and output without a stack copy. The risc0-bigint2 syscalls for
//! `modadd`/`modsub`/`modmul` read all inputs before any writes, so aliasing `out` with `a` or
//! `b` is safe. `modinv` does NOT support aliasing — `out` must not alias `a`.

use ark_ff::BigInt;

/// Width-specific modular-arithmetic dispatch.
///
/// Implementation detail: exposed as `pub` only so that [`crate::R0Config`] can reference it in
/// its where clause. Users should not implement this trait directly; impls are provided by this
/// crate for [`BigInt<4>`] and [`BigInt<6>`] on the `zkvm` target, and for all [`BigInt<N>`] on
/// the host fallback.
pub trait FieldFfi: Sized {
    /// # Safety
    /// - All pointers must be readable / writable and properly aligned for `Self`.
    /// - `out` may alias `a` or `b` (risc0-bigint2 reads before writing).
    unsafe fn modadd(a: *const Self, b: *const Self, m: *const Self, out: *mut Self);
    /// # Safety
    /// Same aliasing rules as [`FieldFfi::modadd`].
    unsafe fn modsub(a: *const Self, b: *const Self, m: *const Self, out: *mut Self);
    /// # Safety
    /// Same aliasing rules as [`FieldFfi::modadd`].
    unsafe fn modmul(a: *const Self, b: *const Self, m: *const Self, out: *mut Self);
    /// # Safety
    /// - All pointers must be readable / writable and properly aligned for `Self`.
    /// - `out` must NOT alias `a` (the `modinv` circuit has multi-constraint structure that
    ///   miscompiles on aliased input/output).
    /// - `*a` must not be the zero element of the field (no inverse exists; zkvm panics,
    ///   host fallback panics via Fermat returning zero).
    unsafe fn modinv(a: *const Self, m: *const Self, out: *mut Self);

    /// Unchecked modular add: same as [`FieldFfi::modadd`] but without the internal
    /// `assert!(result < modulus)` canonicality check. Output is correct mod `m` but may be
    /// anywhere in `[0, 2^{64·N})`. Caller is responsible for canonicalising before treating
    /// the output as an arkworks `Fp`.
    ///
    /// # Safety
    /// Same aliasing rules as [`FieldFfi::modadd`].
    unsafe fn modadd_unchecked(a: *const Self, b: *const Self, m: *const Self, out: *mut Self);

    /// Unchecked modular multiply. See [`FieldFfi::modadd_unchecked`].
    ///
    /// # Safety
    /// Same aliasing rules as [`FieldFfi::modmul`].
    unsafe fn modmul_unchecked(a: *const Self, b: *const Self, m: *const Self, out: *mut Self);
}

#[cfg(target_os = "zkvm")]
mod zkvm_impl {
    use super::*;
    use risc0_bigint2::field::{
        modadd_256, modadd_384, modinv_256, modinv_384, modmul_256, modmul_384, modsub_256,
        modsub_384,
        unchecked::{
            modadd_256 as modadd_256_unchecked, modadd_384 as modadd_384_unchecked,
            modmul_256 as modmul_256_unchecked, modmul_384 as modmul_384_unchecked,
        },
    };

    // Reinterpret `*const BigInt<N>` as `&[u32; 2N]` for the risc0-bigint2 FFI.
    //
    // BigInt<N> is `pub struct BigInt<const N: usize>(pub [u64; N])` with default layout, and
    // the inner field starts at offset 0 with matching alignment. On the zkvm (little-endian
    // RISC-V), `[u64; N]` is bit-identical to `[u32; 2N]`, so the cast is a no-op.
    impl FieldFfi for BigInt<4> {
        #[inline(always)]
        unsafe fn modadd(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modadd_256(
                    &*a.cast::<[u32; 8]>(),
                    &*b.cast::<[u32; 8]>(),
                    &*m.cast::<[u32; 8]>(),
                    &mut *out.cast::<[u32; 8]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modsub(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modsub_256(
                    &*a.cast::<[u32; 8]>(),
                    &*b.cast::<[u32; 8]>(),
                    &*m.cast::<[u32; 8]>(),
                    &mut *out.cast::<[u32; 8]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modmul(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modmul_256(
                    &*a.cast::<[u32; 8]>(),
                    &*b.cast::<[u32; 8]>(),
                    &*m.cast::<[u32; 8]>(),
                    &mut *out.cast::<[u32; 8]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modinv(a: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modinv_256(
                    &*a.cast::<[u32; 8]>(),
                    &*m.cast::<[u32; 8]>(),
                    &mut *out.cast::<[u32; 8]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modadd_unchecked(
            a: *const Self,
            b: *const Self,
            m: *const Self,
            out: *mut Self,
        ) {
            unsafe {
                modadd_256_unchecked(
                    &*a.cast::<[u32; 8]>(),
                    &*b.cast::<[u32; 8]>(),
                    &*m.cast::<[u32; 8]>(),
                    &mut *out.cast::<[u32; 8]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modmul_unchecked(
            a: *const Self,
            b: *const Self,
            m: *const Self,
            out: *mut Self,
        ) {
            unsafe {
                modmul_256_unchecked(
                    &*a.cast::<[u32; 8]>(),
                    &*b.cast::<[u32; 8]>(),
                    &*m.cast::<[u32; 8]>(),
                    &mut *out.cast::<[u32; 8]>(),
                )
            }
        }
    }

    impl FieldFfi for BigInt<6> {
        #[inline(always)]
        unsafe fn modadd(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modadd_384(
                    &*a.cast::<[u32; 12]>(),
                    &*b.cast::<[u32; 12]>(),
                    &*m.cast::<[u32; 12]>(),
                    &mut *out.cast::<[u32; 12]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modsub(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modsub_384(
                    &*a.cast::<[u32; 12]>(),
                    &*b.cast::<[u32; 12]>(),
                    &*m.cast::<[u32; 12]>(),
                    &mut *out.cast::<[u32; 12]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modmul(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modmul_384(
                    &*a.cast::<[u32; 12]>(),
                    &*b.cast::<[u32; 12]>(),
                    &*m.cast::<[u32; 12]>(),
                    &mut *out.cast::<[u32; 12]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modinv(a: *const Self, m: *const Self, out: *mut Self) {
            unsafe {
                modinv_384(
                    &*a.cast::<[u32; 12]>(),
                    &*m.cast::<[u32; 12]>(),
                    &mut *out.cast::<[u32; 12]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modadd_unchecked(
            a: *const Self,
            b: *const Self,
            m: *const Self,
            out: *mut Self,
        ) {
            unsafe {
                modadd_384_unchecked(
                    &*a.cast::<[u32; 12]>(),
                    &*b.cast::<[u32; 12]>(),
                    &*m.cast::<[u32; 12]>(),
                    &mut *out.cast::<[u32; 12]>(),
                )
            }
        }
        #[inline(always)]
        unsafe fn modmul_unchecked(
            a: *const Self,
            b: *const Self,
            m: *const Self,
            out: *mut Self,
        ) {
            unsafe {
                modmul_384_unchecked(
                    &*a.cast::<[u32; 12]>(),
                    &*b.cast::<[u32; 12]>(),
                    &*m.cast::<[u32; 12]>(),
                    &mut *out.cast::<[u32; 12]>(),
                )
            }
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

    // The host impl copies each input to an owned local at the top of the function, so the
    // subsequent `BigUint` conversions never hold references into memory that `out` might
    // alias. This keeps the host path UB-free regardless of aliasing.
    impl<const N: usize> FieldFfi for BigInt<N> {
        unsafe fn modadd(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            let (a, b, m) = unsafe { (*a, *b, *m) };
            let r = (to_biguint(&a) + to_biguint(&b)) % to_biguint(&m);
            unsafe { from_biguint(&r, &mut *out) }
        }
        unsafe fn modsub(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            let (a, b, m) = unsafe { (*a, *b, *m) };
            let (a, b, m) = (to_biguint(&a), to_biguint(&b), to_biguint(&m));
            let r = (&a + &m - b) % &m;
            unsafe { from_biguint(&r, &mut *out) }
        }
        unsafe fn modmul(a: *const Self, b: *const Self, m: *const Self, out: *mut Self) {
            let (a, b, m) = unsafe { (*a, *b, *m) };
            let r = (to_biguint(&a) * to_biguint(&b)) % to_biguint(&m);
            unsafe { from_biguint(&r, &mut *out) }
        }
        unsafe fn modinv(a: *const Self, m: *const Self, out: *mut Self) {
            // Fermat: a^(p - 2) mod p for prime p.
            let (a, m) = unsafe { (*a, *m) };
            let (a_u, m_u) = (to_biguint(&a), to_biguint(&m));
            let exp = &m_u - BigUint::from(2u32);
            unsafe { from_biguint(&a_u.modpow(&exp, &m_u), &mut *out) }
        }
        // Host fallback: `num-bigint` always reduces via `%`, so checked and unchecked behave
        // identically on the host. We simply forward.
        unsafe fn modadd_unchecked(
            a: *const Self,
            b: *const Self,
            m: *const Self,
            out: *mut Self,
        ) {
            unsafe { <Self as FieldFfi>::modadd(a, b, m, out) }
        }
        unsafe fn modmul_unchecked(
            a: *const Self,
            b: *const Self,
            m: *const Self,
            out: *mut Self,
        ) {
            unsafe { <Self as FieldFfi>::modmul(a, b, m, out) }
        }
    }
}
