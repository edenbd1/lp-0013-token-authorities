//! Full authority lifecycle integration test.

use lez_approval::Authority;
use nssa_core::account::{Account, AccountId, AccountWithMetadata, Data};
use token_authority_core::TokenDefinition;
use token_authority_program::authority::{
    mint_with_authority, new_fungible_definition_with_authority, revoke_authority,
};

fn admin_id() -> AccountId {
    AccountId::new([1; 32])
}

fn holding_id() -> AccountId {
    AccountId::new([20; 32])
}

fn uninit(id: AccountId) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: true,
        account_id: id,
    }
}

#[test]
fn create_mint_revoke_reject() {
    let posts = new_fungible_definition_with_authority(
        uninit(admin_id()),
        uninit(holding_id()),
        "FLOW".into(),
        0,
        admin_id(),
    );

    let def_acct = AccountWithMetadata {
        account: posts[0].account().clone(),
        is_authorized: true,
        account_id: admin_id(),
    };
    let posts = mint_with_authority(def_acct, uninit(holding_id()), 1000);

    let def: TokenDefinition = TokenDefinition::try_from(&posts[0].account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { total_supply, .. } => {
            assert_eq!(*total_supply, 1000);
        }
        _ => panic!("expected FungibleWithAuthority"),
    }

    let def_acct = AccountWithMetadata {
        account: posts[0].account().clone(),
        is_authorized: true,
        account_id: admin_id(),
    };
    let posts = revoke_authority(def_acct);

    let def: TokenDefinition = TokenDefinition::try_from(&posts[0].account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { authority, .. } => {
            assert!(authority.is_renounced());
        }
        _ => panic!("expected FungibleWithAuthority"),
    }

    let def_acct = AccountWithMetadata {
        account: posts[0].account().clone(),
        is_authorized: true,
        account_id: admin_id(),
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mint_with_authority(def_acct, uninit(holding_id()), 100);
    }));
    assert!(result.is_err(), "mint after revoke should panic");
}
