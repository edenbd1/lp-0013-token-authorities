//! SDK for constructing token authority transactions on the Logos Execution Zone.
//!
//! This crate re-exports the key types and documents the transaction construction
//! pattern for each of the four authority instructions.
//!
//! # Instructions
//!
//! ## `NewFungibleDefinitionWithAuthority`
//!
//! Creates a new fungible token with a mint authority.
//!
//! - **Accounts:** definition (authorized, uninitialized), holding (authorized, uninitialized)
//! - **Instruction data:** `name: String`, `initial_supply: u128`, `authority: AccountId`
//! - **Signers:** definition owner, holding owner
//!
//! ## `MintWithAuthority`
//!
//! Mints tokens gated by the recorded mint authority.
//!
//! - **Accounts:** definition (authorized), holding, authority_signer (authorized)
//! - **Instruction data:** `amount_to_mint: u128`
//! - **Signers:** definition owner, authority signer
//!
//! ## `RotateAuthority`
//!
//! Atomically replaces the current authority with a new admin.
//!
//! - **Accounts:** definition (authorized), authority_signer (authorized)
//! - **Instruction data:** `new_authority: AccountId`
//! - **Signers:** definition owner, current authority signer
//!
//! ## `RevokeAuthority`
//!
//! Permanently revokes the mint authority. Terminal — cannot be reversed.
//!
//! - **Accounts:** definition (authorized), authority_signer (authorized)
//! - **Instruction data:** _(none)_
//! - **Signers:** definition owner, current authority signer

pub use lez_approval::{ApprovalError, Authority};
pub use token_authority_core::{Instruction, TokenDefinition, TokenHolding};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_roundtrip() {
        let admin = nssa_core::account::AccountId::new([42; 32]);
        let auth = Authority::new(admin.into_value());
        assert_eq!(auth.admin(), Some(admin.into_value()));
        assert!(!auth.is_renounced());
    }

    #[test]
    fn instruction_variants_exist() {
        let _create = Instruction::NewFungibleDefinitionWithAuthority {
            name: "test".into(),
            initial_supply: 1000,
            authority: nssa_core::account::AccountId::new([1; 32]),
        };
        let _mint = Instruction::MintWithAuthority {
            amount_to_mint: 500,
        };
        let _rotate = Instruction::RotateAuthority {
            new_authority: nssa_core::account::AccountId::new([2; 32]),
        };
        let _revoke = Instruction::RevokeAuthority;
    }
}
