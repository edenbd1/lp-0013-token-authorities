# Compute Unit Budget

## Per-Instruction Overhead

The authority check adds minimal overhead to each gated instruction:

| Operation | Overhead vs Baseline | Description |
|---|---|---|
| `NewFungibleDefinitionWithAuthority` | +33 bytes on-chain | One `Authority` field (1 discriminant + 32 `AccountId`) serialized into the definition account. No additional computation vs `NewFungibleDefinition`. |
| `MintWithAuthority` | +1 AccountId comparison | One `gate()` call: loads stored authority, compares to `authority_account.account_id`. Identical arithmetic to `Mint` otherwise. |
| `RotateAuthority` | +1 AccountId comparison, +1 field write | `gate()` then single field overwrite `authority = Authority::new(new_admin)`. |
| `RevokeAuthority` | +1 AccountId comparison, +1 field write | `gate()` then single field overwrite `authority = Authority::renounced()`. |

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
