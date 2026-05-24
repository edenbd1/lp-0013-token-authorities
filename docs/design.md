# Design Decisions

## Why a New Enum Variant Instead of Modifying Fungible

We chose `TokenDefinition::FungibleWithAuthority` as a new variant rather than adding an `authority: Option<Authority>` field to the existing `Fungible` variant.

**Rationale:**
- Adding a field to `Fungible` is a **Borsh-layout breaking change** that would invalidate every existing `Fungible` definition account on the network.
- A new variant preserves complete binary compatibility — existing accounts deserialize unchanged.
- The Borsh discriminant naturally routes deserialization to the correct variant.

**Trade-off:** Callers must use a separate code path for authority tokens. This is intentional — it prevents accidental use of the ungated `Mint` instruction on authority-gated tokens.

## Why Additive Instruction Variants

The four authority instructions (`NewFungibleDefinitionWithAuthority`, `MintWithAuthority`, `RotateAuthority`, `RevokeAuthority`) are new variants of the existing `Instruction` enum, not modifications of existing variants.

**Rationale:**
- Modifying `Mint` to add optional authority fields would be a wire-format breaking change.
- Existing callers of `Mint` would need to handle the new fields, even if they never use authority.
- The serde encoding of the enum naturally version-gates the new instructions.

## Why Authority in pre_states, Not Instruction Data

Early versions passed `authority_signer: AccountId` as instruction data. This was a **security vulnerability** — anyone who could read the authority's AccountId (visible on-chain) could forge the instruction data and perform unauthorized operations.

The fix: the authority signer is now a pre_state account whose `is_authorized` flag is checked by the NSSA runtime. The handler verifies both:
1. `authority_account.is_authorized` — the signer cryptographically proved they own this account.
2. `authority_account.account_id == stored_authority` — the proved signer matches the recorded admin.

## Why panic-on-failure

LEZ guest programs use panic-on-failure as the canonical rejection mechanism. The prover catches the panic and rejects the transaction. Every other handler in the Token program follows this convention. We do the same.

**Considered alternative:** `Result`-returning handlers. Rejected because mixing `Result` and panic in the same codebase is a maintenance trap — callers would need to know which error style each handler uses.

## Why Single-Admin (Not Multi-Sig)

The prize spec targets a single mint authority model. Multi-sig is a separate concern (LP-0002). The `Authority(Option<AccountId>)` design can be extended to `Authority(Option<MultiSigConfig>)` in the future without changing the wire format of the existing `gate`/`rotate`/`revoke` primitives.

## Why Rejecting Regular Mint on Authority Tokens

When a `FungibleWithAuthority` token is created, the regular `Mint` instruction (which only checks `definition_account.is_authorized`) is explicitly rejected. This prevents the definition owner from bypassing the authority gate.

If we allowed `Mint` on authority tokens, it would mean there are two ways to mint — one gated by the authority, one gated by the definition owner. This would undermine the authority model and confuse users about who can actually mint.
