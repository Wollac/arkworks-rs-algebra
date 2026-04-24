//! Modular-arithmetic dispatch shim.
//!
//! On `target_os = "zkvm"`, calls into `risc0-bigint2`'s checked entry points, which
//! internally `assert!(result < modulus)` so their outputs are canonical. On other
//! targets, falls back to `num-bigint` so the crate still builds and host-side tests
//! can exercise the full `FpConfig` surface.
//!
//! The trait takes raw pointers so in-place callers (`a += b`, `-a`, etc.) can pass
//! `a` as both input and output without a stack copy. The `modadd`, `modsub`, and
//! `modmul` syscalls read all inputs before any writes, so `out` may alias `a` or `b`.
//! `modinv` does not support aliasing; `out` must not alias `a`.

use ark_ff::BigInt;

/// Width-specific modular-arithmetic dispatch.
///
/// Exposed as `pub` only so that [`crate::R0Config`] can reference it in a where clause.
/// Users should not implement this trait directly; impls are provided by this crate for
/// [`BigInt<4>`] and [`BigInt<6>`] on the `zkvm` target, and for all [`BigInt<N>`] on the
/// host fallback.
pub trait FieldFfi: Sized {
    /// # Safety
    ///
    /// - `a`, `b`, and `out` must be readable/writable and properly aligned for `Self`.
    /// - `out` may alias `a` or `b` (risc0-bigint2 reads before writing).
    unsafe fn modadd(a: *const Self, b: *const Self, m: &Self, out: *mut Self);

    /// # Safety
    ///
    /// Same aliasing rules as [`FieldFfi::modadd`].
    unsafe fn modsub(a: *const Self, b: *const Self, m: &Self, out: *mut Self);

    /// # Safety
    ///
    /// Same aliasing rules as [`FieldFfi::modadd`].
    unsafe fn modmul(a: *const Self, b: *const Self, m: &Self, out: *mut Self);

    /// # Safety
    ///
    /// - `out` must be writable and properly aligned for `Self`, and must point to
    ///   memory distinct from `a` (the `modinv` circuit has multi-constraint structure
    ///   that miscompiles on aliased input/output).
    /// - `*a` must not be zero; the zkvm path panics and the host path panics via
    ///   `BigUint::modinv` returning `None`.
    unsafe fn modinv(a: &Self, m: &Self, out: *mut Self);

    /// Unchecked modular add.
    ///
    /// Same as [`FieldFfi::modadd`] but without the internal `assert!(result < modulus)`
    /// canonicality check. The output is correct mod `m` but may fall anywhere in
    /// `[0, 2^{64·N})`; the caller is responsible for canonicalising before treating it
    /// as an arkworks `Fp`.
    ///
    /// # Safety
    ///
    /// Same aliasing rules as [`FieldFfi::modadd`].
    unsafe fn modadd_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self);

    /// Unchecked modular multiply. See [`FieldFfi::modadd_unchecked`].
    ///
    /// # Safety
    ///
    /// Same aliasing rules as [`FieldFfi::modmul`].
    unsafe fn modmul_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self);
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

    /// Reinterprets `&BigInt<N>` as `&[u32; 2N]` for the risc0-bigint2 FFI.
    #[inline(always)]
    fn limbs<const N: usize, const M: usize>(x: &BigInt<N>) -> &[u32; M]
    where
        [u64; N]: bytemuck::NoUninit,
        [u32; M]: bytemuck::AnyBitPattern,
    {
        bytemuck::cast_ref(&x.0)
    }

    // The `*const BigInt<N>` → `*const [u32; 2N]` pointer casts below rely on `BigInt<N>`
    // being a tuple struct with `[u64; N]` at offset 0 and matching alignment. On the zkvm
    // (little-endian RISC-V), `[u64; N]` is bit-identical to `[u32; 2N]`, so the cast is a
    // no-op.
    impl FieldFfi for BigInt<4> {
        #[inline(always)]
        unsafe fn modadd(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modadd_256(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modsub(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modsub_256(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modmul(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modmul_256(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modinv(a: &Self, m: &Self, out: *mut Self) {
            unsafe { modinv_256(limbs(a), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modadd_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modadd_256_unchecked(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modmul_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modmul_256_unchecked(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
    }

    impl FieldFfi for BigInt<6> {
        #[inline(always)]
        unsafe fn modadd(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modadd_384(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modsub(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modsub_384(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modmul(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modmul_384(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modinv(a: &Self, m: &Self, out: *mut Self) {
            unsafe { modinv_384(limbs(a), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modadd_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modadd_384_unchecked(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
        #[inline(always)]
        unsafe fn modmul_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { modmul_384_unchecked(&*a.cast(), &*b.cast(), limbs(m), &mut *out.cast()) }
        }
    }
}

#[cfg(not(target_os = "zkvm"))]
mod host_impl {
    use super::*;
    use num_bigint::BigUint;

    // The host impl copies each input to an owned local at the top of the function, so the
    // subsequent `BigUint` conversions never hold references into memory that `out` might
    // alias. This keeps the host path UB-free regardless of aliasing.
    impl<const N: usize> FieldFfi for BigInt<N> {
        unsafe fn modadd(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            let (a, b) = unsafe { (*a, *b) };
            let r = (BigUint::from(a) + BigUint::from(b)) % BigUint::from(*m);
            unsafe { *out = BigInt::try_from(r).expect("result < m fits in BigInt<N>") }
        }
        unsafe fn modsub(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            let (a, b) = unsafe { (*a, *b) };
            let (a, b, m) = (BigUint::from(a), BigUint::from(b), BigUint::from(*m));
            let r = (&a + &m - b) % &m;
            unsafe { *out = BigInt::try_from(r).expect("result < m fits in BigInt<N>") }
        }
        unsafe fn modmul(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            let (a, b) = unsafe { (*a, *b) };
            let r = (BigUint::from(a) * BigUint::from(b)) % BigUint::from(*m);
            unsafe { *out = BigInt::try_from(r).expect("result < m fits in BigInt<N>") }
        }
        unsafe fn modinv(a: &Self, m: &Self, out: *mut Self) {
            let inv = BigUint::from(*a)
                .modinv(&BigUint::from(*m))
                .expect("modinv: non-invertible input");
            unsafe { *out = BigInt::try_from(inv).expect("inv < m fits in BigInt<N>") }
        }
        // Host fallback: `num-bigint` always reduces via `%`, so checked and unchecked behave
        // identically on the host. We simply forward.
        unsafe fn modadd_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { <Self as FieldFfi>::modadd(a, b, m, out) }
        }
        unsafe fn modmul_unchecked(a: *const Self, b: *const Self, m: &Self, out: *mut Self) {
            unsafe { <Self as FieldFfi>::modmul(a, b, m, out) }
        }
    }
}
