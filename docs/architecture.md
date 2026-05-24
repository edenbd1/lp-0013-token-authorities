# Architecture

## Overview

The mint authority model is layered **additively** on top of the existing LEZ Token program. Four new instruction variants coexist with the seven original variants — pre-existing code paths are untouched.

## Components

```
┌─────────────────────────────────────┐
│          lez-approval (RFP-001)     │
│  Authority · ApprovalError          │
│  gate() · rotate() · revoke()      │
└──────────────┬──────────────────────┘
               │ depends on
┌──────────────▼──────────────────────┐
│       token-authority-core          │
│  Instruction enum (11 variants)     │
│  TokenDefinition (3 variants)       │
│  FungibleWithAuthority { authority }│
└──────────────┬──────────────────────┘
               │ depends on
┌──────────────▼──────────────────────┐
│      token-authority-program        │
│  authority.rs handlers              │
│  burn.rs · mint.rs (patched)        │
│  55 unit tests                      │
└──────────────┬──────────────────────┘
               │ used by
┌──────────────▼──────────────────────┐
│       token-authority-sdk           │
│  Transaction builders               │
│  Type re-exports                    │
└─────────────────────────────────────┘
```

## Authority State Machine

```
Authority::new(admin)
       │
       ├─ gate(admin)       → OK
       ├─ gate(other)       → panic!(Unauthorized)
       ├─ rotate(admin, B)  → Authority::new(B)
       └─ revoke(admin)     → Authority::renounced()
                                   │
                                   ├─ gate(any)     → panic!(Renounced)
                                   ├─ rotate(any,_) → panic!(Renounced)
                                   └─ revoke(any)   → panic!(Renounced)
```

Three states, no intermediate state, terminal revocation.

## Security Model

Authority is verified **cryptographically** at the account level:

1. The `authority_signer` is a separate pre_state account in the transaction.
2. The NSSA runtime verifies the signature and sets `is_authorized: true`.
3. The handler checks both `is_authorized` AND that the account_id matches the stored authority.

This means the authority's AccountId is never passed as instruction data — it is always resolved from a signed account in the transaction's account list.

## Instruction → Account Mapping

| Instruction | Accounts (pre_states) | Instruction Data |
|---|---|---|
| `NewFungibleDefinitionWithAuthority` | definition (auth), holding (auth) | name, initial_supply, authority AccountId |
| `MintWithAuthority` | definition (auth), holding, authority_signer (auth) | amount_to_mint |
| `RotateAuthority` | definition (auth), authority_signer (auth) | new_authority AccountId |
| `RevokeAuthority` | definition (auth), authority_signer (auth) | _(none)_ |

## Backward Compatibility

- `TokenDefinition::FungibleWithAuthority` is a **new enum variant** — existing `Fungible` and `NonFungible` variants are unchanged.
- Original `Mint` instruction is explicitly **rejected** on `FungibleWithAuthority` tokens (forces callers to use `MintWithAuthority`).
- `Burn` works normally on `FungibleWithAuthority` tokens — burn is holder-authorized, not authority-gated.
- All other instructions (`Transfer`, `InitializeAccount`, `PrintNft`, etc.) are unaffected.
