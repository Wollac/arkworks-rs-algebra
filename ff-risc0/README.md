<h1 align="center">ark-ff-risc0</h1>

RISC Zero zkVM backend for [`ark-ff`](https://crates.io/crates/ark-ff). Provides [`R0Backend`] and [`R0Config`], an alternative to `MontBackend` that routes field arithmetic through `risc0-bigint2` accelerator syscalls on the `zkvm` target. On other targets the same crate builds against a `num-bigint` fallback, so host-side tests keep working.

## Representation

Elements are stored as plain integers in `[0, p)` rather than in Montgomery form. This avoids per-op Montgomery reductions, which would dominate the cost of a single `modmul` syscall. `from_bigint` and `into_bigint` are therefore zero-cost.

## Supported widths

`N = 4` (256-bit) and `N = 6` (384-bit), matching the modular-arithmetic accelerator blobs shipped by `risc0-bigint2`.

## Example

```rust
use ark_ff::Fp256;
use ark_ff_risc0::{R0Backend, R0Config};

#[derive(R0Config)]
#[modulus = "115792089237316195423570985008687907853269984665640564039457584007908834671663"]
#[generator = "3"]
pub struct FqConfig;

pub type Fq = Fp256<R0Backend<FqConfig, 4>>;
```

The derive computes `TWO_ADICITY` and `TWO_ADIC_ROOT_OF_UNITY` for you. Manual impls of `R0Config` are still supported.
