# Error Codes

All errors are surfaced as panics (the canonical LEZ guest rejection mechanism).

## Authority Errors (from `lez-approval`)

| Code | Name | Panic Message | When |
|---|---|---|---|
| 2001 | `Unauthorized` | "Unauthorized: signer is not the current authority" | A gated operation is called by an account that is not the current admin. The `authority_account.account_id` does not match the stored `Authority` admin. |
| 2002 | `Renounced` | "Renounced: authority has been permanently revoked" | Any gated operation (`gate`, `rotate`, `revoke`, `MintWithAuthority`) is called on an authority that has been permanently revoked via `RevokeAuthority`. This state is terminal and cannot be reversed. |

## Handler Errors (from `token-authority-program`)

| Panic Message | When |
|---|---|
| "Definition authorization is missing" | The definition account in pre_states does not have `is_authorized: true`. |
| "Authority signer authorization is missing" | The authority signer account in pre_states does not have `is_authorized: true`. The transaction was not properly signed by the authority. |
| "Token definition must have an authority" | `MintWithAuthority`, `RotateAuthority`, or `RevokeAuthority` was called on a `TokenDefinition` variant that does not have an authority field (e.g. plain `Fungible`). |
| "MintWithAuthority requires FungibleWithAuthority definition" | The token definition is not `FungibleWithAuthority` after the authority gate passed — this should not happen in normal use. |
| "Use MintWithAuthority for authority-gated tokens" | The regular `Mint` instruction was called on a `FungibleWithAuthority` definition. Use `MintWithAuthority` instead. |
| "Definition target account must have default values" | `NewFungibleDefinitionWithAuthority` was called with a definition account that already contains data. |
| "Holding target account must have default values" | `NewFungibleDefinitionWithAuthority` was called with a holding account that already contains data. |

## What's NOT in the Error Message

Error messages intentionally do **not** include:
- The stored authority's AccountId (would leak the admin's identity in the sequencer error log)
- The signer's AccountId (would confirm to an attacker that they reached the gate check)
- Any private account data
