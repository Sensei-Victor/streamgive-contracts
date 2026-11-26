# Wasm size investigation (donation-vault)

## Summary
Baseline and experiment measurements for reducing Soroban contract wasm size, focusing on `donation_vault.wasm` (budget 98304 bytes) and `ngo_registry.wasm` (budget 65536 bytes).

## Environment
- Rust: rustc 1.99.1 / cargo 1.99.1
- Target: `wasm32v1-none`
- Build: `cargo build --workspace --target wasm32v1-none --release`
- Measurement: file byte size (`wc -c` equivalent / PowerShell `Get-Item.Length`), compared to `scripts/check-wasm-size.sh` budgets

## Baseline
Measured after fixing build blockers in this environment (donation-vault borrow-after-move in stream constructors, cfg-gating `ngo-registry` tests out of non-test builds, and repairing unparsable/corrupted test fragments so formatting/build tooling could run). These fixes were required to produce wasm artifacts and run checks.

| Contract | Size (bytes) | Size (KiB) | Budget (bytes) | Status |
| --- | ---: | ---: | ---: | --- |
| `ngo_registry.wasm` | 68098 | 66.50 | 65536 | FAIL (+2562) |
| `donation_vault.wasm` | 136969 | 133.76 | 98304 | FAIL (+38665) |

## Experiments

| Change | ngo_registry.wasm | donation_vault.wasm | Notes |
| --- | ---: | ---: | --- |
| Baseline (buildable) | 68098 | 136969 | Starting point for optimization deltas |
| Replace `NgoRegistryClient` (`#[contractclient]`) with manual `env.try_invoke_contract` for registry verification | 68098 (0) | 136958 (-11) | Small code-size win; removes generated client wrapper/dispatch for a single cross-contract call. Kept. |
| Disable release `overflow-checks` (experiment, reverted) | 68098 (0) | 136130 (-828 vs previous) | Measured impact of arithmetic overflow checks in release profile; not kept because overflow checks are an important safety guard for contract math. |

## Size contributors / observations

- **Release profile is already size-oriented:** `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`, `debug = 0`.
- **Dependency footprint:** `donation-vault`'s normal dependency is `soroban-sdk 27.0.6` (and its transitive Soroban host/env/crypto stack), which dominates contract binary size in typical Soroban builds.
- **Contract surface area:** `donation-vault` exports a large public API (~74 entry points) with extensive state/event/types and batch/settlement logic in [`contracts/donation-vault/src/lib.rs`](../contracts/donation-vault/src/lib.rs) (~3.6k lines including docs). Soroban contract specs/dispatch are generated per exported interface, so feature breadth directly affects wasm size.
- **Build correctness fix discovered:** `contracts/ngo-registry/src/lib.rs` declared `mod test;` without `#[cfg(test)]`, which pulled test code into non-test builds and blocked wasm compilation until gated. Gating tests is required for correct release artifacts.
- **Tooling:** `stellar` CLI / `twiggy` / `wasm-opt` / `wasm-tools` were not available in this environment, so analysis is based on successful release builds, script budgets, and targeted experiments rather than section-level wasm breakdown.

## Before/after (kept changes)

| Contract | Before (bytes) | After (bytes) | Delta | Budget | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| `ngo_registry.wasm` | 68098 | 68098 | 0 | 65536 | over budget (+2562) |
| `donation_vault.wasm` | 136969 | 136958 | -11 | 98304 | over budget (+38654) |

*“Before” is the first successful release build after required build-enabling fixes in this environment. “After” includes kept code changes in this branch, rebuilt with `cargo build --workspace --target wasm32v1-none --release` (overflow checks enabled).*

## Validation

- `cargo fmt --all -- --check`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- `cargo build --workspace --target wasm32v1-none --release`: pass (artifacts measured above)
- `cargo test --workspace`: not runnable in this Windows `x86_64-pc-windows-gnullvm` environment due to linker failure `ld.lld: error: too many exported symbols` while linking contract `cdylib` test artifacts (DLL export limit). This appears environment/toolchain-specific rather than caused by the wasm-size changes; CI runs these checks on Linux.

## Notes on additional required fixes

To produce reproducible builds and run required checks in this environment, this branch also includes minimal build/test repairs discovered during investigation:
- `donation-vault`: fixed borrow-after-move in stream creation wrappers and a minor unused-mut/clippy issue.
- `ngo-registry`: gated `mod test` with `#[cfg(test)]` so non-test/wasm builds do not compile tests.
- Repaired corrupted/unparsable test fragments that blocked formatting and compilation.


