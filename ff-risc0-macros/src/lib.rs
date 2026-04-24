//! Derive macro for `ark_ff_risc0::R0Config`.
//!
//! Precomputes `TWO_ADIC_ROOT_OF_UNITY` (and, if a small subgroup is configured,
//! `LARGE_SUBGROUP_ROOT_OF_UNITY`) at macro-expansion time via `num-bigint::modpow`,
//! so users only need to supply `modulus` and `generator`. The emitted constants are
//! stored as plain integers in `[0, p)`, matching the R0 backend's representation.

#![forbid(unsafe_code)]

use num_bigint::BigUint;
use num_traits::{Num, Zero};
use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
use syn::{DeriveInput, Expr, ExprLit, Lit, Meta};

/// Derive the [`R0Config`](https://docs.rs/ark-ff-risc0/latest/ark_ff_risc0/trait.R0Config.html)
/// trait.
///
/// Attributes:
/// * `modulus` (required): the prime modulus, as a decimal / hex / oct / bin string.
/// * `generator` (required): a multiplicative generator of `F_p^*`; must be a quadratic
///   non-residue.
/// * `small_subgroup_base`, `small_subgroup_power` (optional): together specify a mixed-radix
///   subgroup of size `base^power` within the two-adic part of `p − 1`.
///
/// Only `N = 4` (256-bit) and `N = 6` (384-bit) moduli are accepted; this matches the widths
/// implemented by `ark_ff_risc0::FieldFfi`.
#[proc_macro_derive(
    R0Config,
    attributes(modulus, generator, small_subgroup_base, small_subgroup_power)
)]
pub fn r0_config_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("failed to parse derive input");
    let name = &ast.ident;

    let modulus_str = fetch_attr("modulus", &ast.attrs)
        .expect("`#[modulus = \"...\"]` attribute is required");
    let generator_str = fetch_attr("generator", &ast.attrs)
        .expect("`#[generator = \"...\"]` attribute is required");

    let modulus = parse_positive(&modulus_str).expect("`modulus` must be a positive integer");
    let generator =
        parse_positive(&generator_str).expect("`generator` must be a positive integer");

    let small_subgroup_base: Option<u32> = fetch_attr("small_subgroup_base", &ast.attrs)
        .map(|s| s.parse().expect("`small_subgroup_base` must be a u32"));
    let small_subgroup_power: Option<u32> = fetch_attr("small_subgroup_power", &ast.attrs)
        .map(|s| s.parse().expect("`small_subgroup_power` must be a u32"));

    // Limb count: smallest `N` such that `modulus < 2^{64N}`.
    let mut limbs = 1usize;
    {
        let mut cur = BigUint::from(1u32) << 64;
        while cur < modulus {
            limbs += 1;
            cur <<= 64;
        }
    }

    assert!(
        limbs == 4 || limbs == 6,
        "ark-ff-risc0 supports only N=4 (256-bit) and N=6 (384-bit) moduli; got {} limbs \
         ({} bits)",
        limbs,
        modulus.bits()
    );

    // Reduce generator into `[0, p)`.
    let generator = &generator % &modulus;
    assert!(
        !generator.is_zero(),
        "`generator` must be nonzero mod `modulus`"
    );

    // p − 1 = 2^s * t, with t odd.
    let mut trace = &modulus - BigUint::from(1u32);
    while !trace.bit(0) {
        trace >>= 1u32;
    }

    // 2^s-th root of unity = generator^t mod p.
    let two_adic_root = generator.modpow(&trace, &modulus);

    let modulus_limbs = biguint_to_limbs(&modulus, limbs);
    let generator_limbs = biguint_to_limbs(&generator, limbs);
    let root_limbs = biguint_to_limbs(&two_adic_root, limbs);

    let mixed_radix = match (small_subgroup_base, small_subgroup_power) {
        (Some(base), Some(power)) => {
            let divisor = BigUint::from(base).pow(power);
            assert!(
                (&trace % &divisor).is_zero(),
                "small_subgroup_base^small_subgroup_power = {}^{} does not divide the odd part \
                 of p - 1",
                base,
                power
            );
            let remaining = &trace / &divisor;
            let large_root = generator.modpow(&remaining, &modulus);
            let large_limbs = biguint_to_limbs(&large_root, limbs);
            quote! {
                const SMALL_SUBGROUP_BASE: ::core::option::Option<u32> = ::core::option::Option::Some(#base);
                const SMALL_SUBGROUP_BASE_ADICITY: ::core::option::Option<u32> = ::core::option::Option::Some(#power);
                const LARGE_SUBGROUP_ROOT_OF_UNITY:
                    ::core::option::Option<::ark_ff::Fp<::ark_ff_risc0::R0Backend<Self, #limbs>, #limbs>> =
                    ::core::option::Option::Some(::ark_ff::Fp(
                        ::ark_ff::BigInt([#(#large_limbs),*]),
                        ::core::marker::PhantomData,
                    ));
            }
        },
        (None, None) => quote! {},
        _ => panic!("must specify both `small_subgroup_base` and `small_subgroup_power`"),
    };

    let expanded = quote! {
        #[automatically_derived]
        impl ::ark_ff_risc0::R0Config<#limbs> for #name {
            const MODULUS: ::ark_ff::BigInt<#limbs> =
                ::ark_ff::BigInt([#(#modulus_limbs),*]);

            const GENERATOR: ::ark_ff::Fp<::ark_ff_risc0::R0Backend<Self, #limbs>, #limbs> =
                ::ark_ff::Fp(
                    ::ark_ff::BigInt([#(#generator_limbs),*]),
                    ::core::marker::PhantomData,
                );

            const TWO_ADIC_ROOT_OF_UNITY:
                ::ark_ff::Fp<::ark_ff_risc0::R0Backend<Self, #limbs>, #limbs> =
                ::ark_ff::Fp(
                    ::ark_ff::BigInt([#(#root_limbs),*]),
                    ::core::marker::PhantomData,
                );

            #mixed_radix
        }
    };

    expanded.into()
}

fn parse_positive(s: &str) -> Option<BigUint> {
    if s.starts_with('-') {
        return None;
    }
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        BigUint::from_str_radix(rest, 16).ok()
    } else if let Some(rest) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        BigUint::from_str_radix(rest, 8).ok()
    } else if let Some(rest) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        BigUint::from_str_radix(rest, 2).ok()
    } else {
        BigUint::from_str(s).ok()
    }
}

fn biguint_to_limbs(n: &BigUint, count: usize) -> Vec<u64> {
    let mut limbs = n.to_u64_digits();
    assert!(
        limbs.len() <= count,
        "value does not fit in {count} limbs"
    );
    limbs.resize(count, 0);
    limbs
}

fn fetch_attr(name: &str, attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Meta::NameValue(nv) = &attr.meta {
            if nv.path.is_ident(name) {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
                {
                    return Some(s.value());
                }
                panic!("attribute `{name}` must be a string literal");
            }
        }
    }
    None
}
