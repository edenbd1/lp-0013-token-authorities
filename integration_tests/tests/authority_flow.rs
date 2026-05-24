//! Integration tests for the token authority flow.
//!
//! These tests exercise the full lifecycle: create, mint, rotate, revoke.

use nssa_core::account::{Account, AccountId, AccountWithMetadata};
use token_authority_core::{TokenDefinition, TokenHolding};
use token_authority_program::authority::{
    mint_with_authority, new_fungible_definition_with_authority, revoke_authority, rotate_authority,
};

fn admin_id() -> AccountId {
    AccountId::new([10; 32])
}

fn new_admin_id() -> AccountId {
    AccountId::new([11; 32])
}

fn definition_id() -> AccountId {
    AccountId::new([20; 32])
}

fn holding_id() -> AccountId {
    AccountId::new([30; 32])
}

fn uninit_definition() -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: false,
        account_id: definition_id(),
    }
}

fn uninit_holding() -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: false,
        account_id: holding_id(),
    }
}

fn signer(id: AccountId) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: true,
        account_id: id,
    }
}

#[test]
fn full_lifecycle_create_mint_rotate_revoke() {
    // 1. Create definition with authority
    let post = new_fungible_definition_with_authority(
        uninit_definition(),
        uninit_holding(),
        "LIFECYCLE".into(),
        0,
        admin_id(),
    );
    let [def_post, _holding_post] = post.try_into().unwrap();

    // 2. Mint with authority
    let def_account = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: definition_id(),
    };
    let post = mint_with_authority(def_account, uninit_holding(), signer(admin_id()), 1000);
    let [def_post, holding_post] = post.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { total_supply, .. } => {
            assert_eq!(*total_supply, 1000);
        }
        _ => panic!("Expected FungibleWithAuthority"),
    }
    let holding: TokenHolding = TokenHolding::try_from(&holding_post.account().data).unwrap();
    match holding {
        TokenHolding::Fungible { balance, .. } => assert_eq!(balance, 1000),
        _ => panic!("Expected Fungible"),
    }

    // 3. Rotate authority
    let def_account = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: definition_id(),
    };
    let post = rotate_authority(def_account, signer(admin_id()), new_admin_id());
    let [def_post] = post.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { authority, .. } => {
            assert_eq!(authority.admin(), Some(new_admin_id()));
        }
        _ => panic!("Expected FungibleWithAuthority"),
    }

    // 4. Old admin cannot mint
    let def_account = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: definition_id(),
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = mint_with_authority(def_account, uninit_holding(), signer(admin_id()), 100);
    }));
    assert!(result.is_err());

    // 5. New admin can mint
    let def_account = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: definition_id(),
    };
    let post = mint_with_authority(def_account, uninit_holding(), signer(new_admin_id()), 500);
    let [def_post, _] = post.try_into().unwrap();

    // 6. Revoke authority
    let def_account = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: definition_id(),
    };
    let post = revoke_authority(def_account, signer(new_admin_id()));
    let [def_post] = post.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { authority, .. } => {
            assert!(authority.is_renounced());
        }
        _ => panic!("Expected FungibleWithAuthority"),
    }

    // 7. Nobody can mint after revocation
    let def_account = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: definition_id(),
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = mint_with_authority(def_account, uninit_holding(), signer(new_admin_id()), 100);
    }));
    assert!(result.is_err());
}
