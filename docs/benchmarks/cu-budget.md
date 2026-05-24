# Compute Unit Budget

## Measured Cycle Costs

Measured via `risc0_zkvm::default_executor` (no proving, cycle counts only) on Apple Silicon with `risc0-zkvm 3.0.5`, guest ELFs built via `cargo risczero build` (Docker reproducible build). 5 samples per instruction.

| Instruction | User Cycles | Segments | Exec Time (ms) | vs Baseline `Mint` |
|---|---|---|---|---|
| `Mint` (baseline) | 117,287 | 1 | 28.02 | — |
| **`MintWithAuthority`** | **166,746** | 1 | 28.55 | +42% (+49,459 cycles) |
| **`RotateAuthority`** | **124,085** | 1 | 27.62 | +6% (+6,798 cycles) |
| **`RevokeAuthority`** | **101,425** | 1 | 27.22 | -14% (-15,862 cycles) |

The `MintWithAuthority` overhead comes from deserializing the larger `FungibleWithAuthority` definition and the additional `is_authorized` + `gate()` check. `RevokeAuthority` is cheaper than baseline `Mint` because it only writes a single field without arithmetic.

## Per-Instruction Overhead (Qualitative)

| Operation | Overhead vs Baseline | Description |
|---|---|---|
| `NewFungibleDefinitionWithAuthority` | +33 bytes on-chain | One `Authority` field (1 discriminant + 32 `AccountId`) serialized into the definition account. No additional computation vs `NewFungibleDefinition`. |
| `MintWithAuthority` | +49,459 cycles (+42%) | One `gate()` call (AccountId comparison) + larger definition deserialization. Same arithmetic otherwise. |
| `RotateAuthority` | +6,798 cycles (+6%) | `gate()` then single field overwrite. |
| `RevokeAuthority` | -15,862 cycles (-14%) | `gate()` then single field write to `None`. Cheaper than Mint because no balance arithmetic. |

## Transaction Size Overhead

| Instruction | Accounts in TX | vs Original `Mint` |
|---|---|---|
| `MintWithAuthority` | 3 (definition, holding, authority_signer) | +1 account (authority_signer) |
| `RotateAuthority` | 2 (definition, authority_signer) | N/A (new instruction) |
| `RevokeAuthority` | 2 (definition, authority_signer) | N/A (new instruction) |

The additional account in `MintWithAuthority` adds one nonce fetch and one signature to the witness set, increasing transaction size by approximately 96-128 bytes.

## On-Chain Storage

`FungibleWithAuthority` occupies 33 bytes more than `Fungible` per definition account:
- 1 byte: `Option` discriminant inside `Authority(Option<AccountId>)` — `Some` when active, `None` when renounced
- 32 bytes: `AccountId` payload (when active) or 0 bytes (when renounced, just the `None` discriminant)
