# Security

## Threat Model

The authority model protects against:

1. **Unauthorized minting** — only the recorded admin can call `MintWithAuthority`.
2. **Authority spoofing** — the authority signer is verified cryptographically via `is_authorized` on a pre_state account, not via instruction data.
3. **Post-revocation minting** — once revoked, the authority is permanently `None`. No instruction can re-introduce an admin.
4. **Bypassing authority via regular Mint** — the original `Mint` instruction explicitly rejects `FungibleWithAuthority` tokens.

## Properties Enforced by the Logos Stack

- **Trustless execution.** Every state transition is proved in a RISC0 guest. A renounced authority is provably renounced — no off-chain admin key can secretly revive it.
- **Censorship resistance.** Authority rotation is a standard transaction. The sequencer cannot selectively reject it without dropping the whole block.
- **Atomicity.** LEZ's per-account read-modify-write semantics guarantee that `rotate_authority` either updates the admin field or panics — no intermediate state where the authority is `None` but the new admin is not yet recorded.

## Known Limitations

- **Single admin only.** The authority model supports one admin at a time. Multi-sig governance requires LP-0002.
- **No freeze authority.** Only the mint authority is implemented. Freeze authority (preventing transfers) is out of scope per the prize spec.
- **No on-curve key validation.** The admin AccountId is not validated to be an on-curve key or deployed PDA. This is a soft requirement from RFP-001 that we document but do not enforce — the NSSA runtime's signature verification already ensures the signer controls the account.
