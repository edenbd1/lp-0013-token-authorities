//! SPEL sidecar for token authority IDL generation.
//!
//! This module mirrors the token authority program's instruction surface
//! using SPEL macros so that `spel -- generate-idl` can produce a
//! machine-generated IDL alongside the hand-authored one.
//!
//! This is NOT the real program — the real handlers live in
//! `crates/token-authority-program/src/authority.rs`. This sidecar exists
//! solely because integrating `spel-framework` into the LEZ workspace
//! causes a `nssa_core` v0.1.0 vs v0.2.0-rc3 dep-graph collision.
//! See `docs/SPEL_STATUS.md` for details.

use spel_framework::prelude::*;

#[lez_program]
mod token_authority {
    use super::*;

    /// Create a new fungible token definition with a mint authority.
    #[instruction]
    pub fn new_fungible_definition_with_authority(
        #[account(init, signer)] definition: AccountWithMetadata,
        #[account(init, signer)] holding: AccountWithMetadata,
        name: String,
        initial_supply: u128,
        authority: [u8; 32],
    ) -> SpelResult {
        Ok(SpelOutput::execute(vec![definition, holding], vec![]))
    }

    /// Mint tokens gated by the recorded mint authority.
    #[instruction]
    pub fn mint_with_authority(
        #[account(mut, signer)] definition: AccountWithMetadata,
        #[account(mut)] holding: AccountWithMetadata,
        amount_to_mint: u128,
    ) -> SpelResult {
        Ok(SpelOutput::execute(vec![definition, holding], vec![]))
    }

    /// Rotate the mint authority to a new admin.
    #[instruction]
    pub fn rotate_authority(
        #[account(mut, signer)] definition: AccountWithMetadata,
        new_authority: [u8; 32],
    ) -> SpelResult {
        Ok(SpelOutput::execute(vec![definition], vec![]))
    }

    /// Permanently revoke the mint authority.
    #[instruction]
    pub fn revoke_authority(
        #[account(mut, signer)] definition: AccountWithMetadata,
    ) -> SpelResult {
        Ok(SpelOutput::execute(vec![definition], vec![]))
    }
}
