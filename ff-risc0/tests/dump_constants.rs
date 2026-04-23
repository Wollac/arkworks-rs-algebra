//! Dumps TWO_ADICITY + TWO_ADIC_ROOT_OF_UNITY + a confirmed multiplicative generator for every
//! curve we want to ship, in plain-integer form ready to paste into a `R0Config` impl.
//!
//! Constants for curves available in `ark-test-curves` (secp256k1, bls12_381) are read from the
//! reference `PrimeField` impls; others are computed from the canonical moduli via num-bigint.
//!
//! Run with `cargo test -p ark-ff-risc0 --test dump_constants -- --nocapture`.

use ark_ff::{BigInt, FftField, PrimeField};
use num_bigint::BigUint;
use num_traits::One;

fn dump_bigint<const N: usize>(label: &str, bi: BigInt<N>) {
    print!("  {label}: BigInt::new([");
    for i in 0..N {
        print!("0x{:016X}", bi.0[i]);
        if i + 1 < N {
            print!(", ");
        }
    }
    println!("])");
}

fn bigint_to_biguint<const N: usize>(x: &BigInt<N>) -> BigUint {
    let mut bytes = Vec::with_capacity(N * 8);
    for limb in &x.0 {
        bytes.extend_from_slice(&limb.to_le_bytes());
    }
    BigUint::from_bytes_le(&bytes)
}

fn biguint_to_bigint<const N: usize>(x: &BigUint) -> BigInt<N> {
    let bytes = x.to_bytes_le();
    let mut out: BigInt<N> = BigInt::new([0u64; N]);
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut buf = [0u8; 8];
        buf[..chunk.len()].copy_from_slice(chunk);
        if i < N {
            out.0[i] = u64::from_le_bytes(buf);
        }
    }
    out
}

/// For a prime modulus, find the smallest element in `{2, 3, 5, 6, 7, 10, 11, 13}` that is a
/// quadratic non-residue (Legendre symbol -1), which therefore has even order containing `2^s`.
/// Returns `(generator, two_adicity, two_adic_root_of_unity)`.
fn compute_fft_constants<const N: usize>(modulus: BigInt<N>) -> (u64, u32, BigInt<N>) {
    let p = bigint_to_biguint(&modulus);
    let p_minus_1 = &p - BigUint::one();

    let mut s: u32 = 0;
    let mut t = p_minus_1.clone();
    while !t.bit(0) {
        t >>= 1;
        s += 1;
    }

    let half = &p_minus_1 / 2u32;
    for cand in 2u64..200 {
        let g = BigUint::from(cand);
        let legendre = g.modpow(&half, &p);
        if legendre == &p - BigUint::one() {
            let rou = g.modpow(&t, &p);
            return (cand, s, biguint_to_bigint(&rou));
        }
    }
    panic!("no QNR found in 2..200");
}

#[test]
fn dump_secp256k1() {
    type Fq = ark_test_curves::secp256k1::Fq;
    type Fr = ark_test_curves::secp256k1::Fr;
    println!("--- secp256k1 Fq (from ark-test-curves) ---");
    println!("  TWO_ADICITY: {}", Fq::TWO_ADICITY);
    dump_bigint(
        "TWO_ADIC_ROOT_OF_UNITY",
        Fq::TWO_ADIC_ROOT_OF_UNITY.into_bigint(),
    );
    println!("--- secp256k1 Fr (from ark-test-curves) ---");
    println!("  TWO_ADICITY: {}", Fr::TWO_ADICITY);
    dump_bigint(
        "TWO_ADIC_ROOT_OF_UNITY",
        Fr::TWO_ADIC_ROOT_OF_UNITY.into_bigint(),
    );
}

#[test]
fn dump_bls12_381() {
    type Fq = ark_test_curves::bls12_381::Fq;
    type Fr = ark_test_curves::bls12_381::Fr;
    println!("--- bls12_381 Fq (from ark-test-curves) ---");
    println!("  TWO_ADICITY: {}", Fq::TWO_ADICITY);
    dump_bigint(
        "TWO_ADIC_ROOT_OF_UNITY",
        Fq::TWO_ADIC_ROOT_OF_UNITY.into_bigint(),
    );
    println!("--- bls12_381 Fr (from ark-test-curves) ---");
    println!("  TWO_ADICITY: {}", Fr::TWO_ADICITY);
    dump_bigint(
        "TWO_ADIC_ROOT_OF_UNITY",
        Fr::TWO_ADIC_ROOT_OF_UNITY.into_bigint(),
    );
}

#[test]
fn dump_bn254() {
    // Fq: 21888242871839275222246405745257275088696311157297823662689037894645226208583
    let fq: BigInt<4> = BigInt::new([
        0x3C208C16D87CFD47,
        0x97816A916871CA8D,
        0xB85045B68181585D,
        0x30644E72E131A029,
    ]);
    let (g, s, rou) = compute_fft_constants::<4>(fq);
    println!("--- bn254 Fq ---");
    println!("  GENERATOR: {g}");
    println!("  TWO_ADICITY: {s}");
    dump_bigint("TWO_ADIC_ROOT_OF_UNITY", rou);

    // Fr: 21888242871839275222246405745257275088548364400416034343698204186575808495617
    let fr: BigInt<4> = BigInt::new([
        0x43E1F593F0000001,
        0x2833E84879B97091,
        0xB85045B68181585D,
        0x30644E72E131A029,
    ]);
    let (g, s, rou) = compute_fft_constants::<4>(fr);
    println!("--- bn254 Fr ---");
    println!("  GENERATOR: {g}");
    println!("  TWO_ADICITY: {s}");
    dump_bigint("TWO_ADIC_ROOT_OF_UNITY", rou);
}

#[test]
fn dump_secp256r1() {
    // Fq: p = 2^256 - 2^224 + 2^192 + 2^96 - 1
    let fq: BigInt<4> = BigInt::new([
        0xFFFFFFFFFFFFFFFF,
        0x00000000FFFFFFFF,
        0x0000000000000000,
        0xFFFFFFFF00000001,
    ]);
    let (g, s, rou) = compute_fft_constants::<4>(fq);
    println!("--- secp256r1 Fq ---");
    println!("  GENERATOR: {g}");
    println!("  TWO_ADICITY: {s}");
    dump_bigint("TWO_ADIC_ROOT_OF_UNITY", rou);

    // Fr: order of secp256r1
    let fr: BigInt<4> = BigInt::new([
        0xF3B9CAC2FC632551,
        0xBCE6FAADA7179E84,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFF00000000,
    ]);
    let (g, s, rou) = compute_fft_constants::<4>(fr);
    println!("--- secp256r1 Fr ---");
    println!("  GENERATOR: {g}");
    println!("  TWO_ADICITY: {s}");
    dump_bigint("TWO_ADIC_ROOT_OF_UNITY", rou);
}

#[test]
fn dump_secp384r1() {
    // Fq: p = 2^384 - 2^128 - 2^96 + 2^32 - 1
    let fq: BigInt<6> = BigInt::new([
        0x00000000FFFFFFFF,
        0xFFFFFFFF00000000,
        0xFFFFFFFFFFFFFFFE,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ]);
    let (g, s, rou) = compute_fft_constants::<6>(fq);
    println!("--- secp384r1 Fq ---");
    println!("  GENERATOR: {g}");
    println!("  TWO_ADICITY: {s}");
    dump_bigint("TWO_ADIC_ROOT_OF_UNITY", rou);

    // Fr: order
    let fr: BigInt<6> = BigInt::new([
        0xECEC196ACCC52973,
        0x581A0DB248B0A77A,
        0xC7634D81F4372DDF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ]);
    let (g, s, rou) = compute_fft_constants::<6>(fr);
    println!("--- secp384r1 Fr ---");
    println!("  GENERATOR: {g}");
    println!("  TWO_ADICITY: {s}");
    dump_bigint("TWO_ADIC_ROOT_OF_UNITY", rou);
}
