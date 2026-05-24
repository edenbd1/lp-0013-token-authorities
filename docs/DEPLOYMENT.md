# Deployment Guide

## Prerequisites

- Rust stable toolchain
- Risc0 toolchain (`cargo-risczero 3.0.5`, `r0vm 3.0.5`)
- Docker + Docker Compose
- The [LEZ fork](https://github.com/edenbd1/logos-execution-zone/tree/lp-0013-token-authorities) (for wallet CLI and sequencer)

## Building Guest ELFs

The token program runs inside a Risc0 guest. After modifying the program, rebuild the ELFs:

```bash
cd logos-execution-zone  # the LEZ fork
just build-artifacts
# This runs cargo risczero build in Docker and copies .bin files to artifacts/
```

## Running the Demo

```bash
# From the LEZ fork root:
RISC0_DEV_MODE=0 ./demo.sh
```

The demo script:
1. Starts a local sequencer via `docker compose`
2. Initializes a wallet with a fresh keypair
3. Creates 4 public accounts (definition, supply, holder, new_admin)
4. Executes the full authority lifecycle:
   - `new-with-authority` — create token with 1000 supply
   - `mint-with-authority` — mint 500 to holder
   - `rotate-authority` — transfer control to new_admin
   - `mint-with-authority` — new admin mints 300
   - `revoke-authority` — permanently fix supply
   - Verify mint-after-revoke is rejected

## Cycle Cost Measurements

Measured via `risc0_zkvm::default_executor` (no proving, cycle counts only) using the `cycle_bench` tool in the LEZ fork:

```bash
cargo run --release -p cycle_bench
```

### Results (Apple Silicon, `RISC0_DEV_MODE=0`, `risc0-zkvm 3.0.5`)

| Instruction | User Cycles | Segments | Exec Time (ms) | vs Baseline `Mint` |
|---|---|---|---|---|
| `Mint` (baseline) | 117,287 | 1 | 28.02 | — |
| `MintWithAuthority` | **166,746** | 1 | 28.55 | +42% (+49,459 cycles) |
| `RotateAuthority` | **124,085** | 1 | 27.62 | +6% (+6,798 cycles) |
| `RevokeAuthority` | **101,425** | 1 | 27.22 | -14% (-15,862 cycles) |

Measured with `cargo run --release -p cycle_bench` on the [LEZ fork](https://github.com/edenbd1/logos-execution-zone/tree/lp-0013-token-authorities). 5 samples per instruction, best-of-5 reported. Guest ELF built via `cargo risczero build` (Docker reproducible build).

### Overhead Analysis

The authority instructions add minimal overhead vs baseline:
- **MintWithAuthority vs Mint**: +1 `AccountId` equality check (32-byte comparison) for the `gate()` call. Same arithmetic otherwise.
- **RotateAuthority**: 1 `gate()` check + 1 field write. No arithmetic.
- **RevokeAuthority**: 1 `gate()` check + 1 field write to `None`. No arithmetic.

### On-Chain Storage

`FungibleWithAuthority` occupies 33 bytes more than `Fungible`:
- 1 byte: `Option` discriminant (`Some`/`None`)
- 32 bytes: `AccountId` payload (when authority is active)
