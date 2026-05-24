//! Integration tests for the token authority lifecycle.
//!
//! These tests exercise the full handler pipeline against in-memory accounts,
//! simulating what the LEZ sequencer does at the pre_state / post_state level.

use lez_approval::Authority;
use nssa_core::account::{Account, AccountId, AccountWithMetadata, Data};
use token_authority_core::{TokenDefinition, TokenHolding};
use token_authority_program::authority::{
    mint_with_authority, new_fungible_definition_with_authority, revoke_authority, rotate_authority,
};
use token_authority_program::burn::burn;

fn admin_id() -> AccountId {
    AccountId::new([1; 32])
}

fn new_admin_id() -> AccountId {
    AccountId::new([2; 32])
}

fn def_id() -> AccountId {
    AccountId::new([10; 32])
}

fn holding_id() -> AccountId {
    AccountId::new([20; 32])
}

fn uninit_account(id: AccountId) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: true,
        account_id: id,
    }
}

fn signer(id: AccountId) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: true,
        account_id: id,
    }
}

fn signer_unauthorized(id: AccountId) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: false,
        account_id: id,
    }
}

fn make_def(total_supply: u128, authority: Authority) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account {
            program_owner: [5_u32; 8],
            balance: 0,
            data: Data::from(&TokenDefinition::FungibleWithAuthority {
                name: "TEST".into(),
                total_supply,
                metadata_id: None,
                authority,
            }),
            nonce: 0_u128.into(),
        },
        is_authorized: true,
        account_id: def_id(),
    }
}

fn make_holding(balance: u128) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account {
            program_owner: [5_u32; 8],
            balance: 0,
            data: Data::from(&TokenHolding::Fungible {
                definition_id: def_id(),
                balance,
            }),
            nonce: 0_u128.into(),
        },
        is_authorized: true,
        account_id: holding_id(),
    }
}

#[test]
fn full_lifecycle_create_mint_rotate_revoke() {
    // Create
    let posts = new_fungible_definition_with_authority(
        uninit_account(def_id()),
        uninit_account(holding_id()),
        "LIFECYCLE".into(),
        1000,
        admin_id(),
    );
    let [def_post, _holding_post] = posts.try_into().unwrap();
    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    assert!(matches!(def, TokenDefinition::FungibleWithAuthority { .. }));

    // Mint 500
    let def_acct = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: def_id(),
    };
    let posts = mint_with_authority(
        def_acct,
        uninit_account(holding_id()),
        signer(admin_id()),
        500,
    );
    let [def_post, holding_post] = posts.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { total_supply, .. } => {
            assert_eq!(*total_supply, 1500);
        }
        _ => panic!("wrong variant"),
    }
    let holding: TokenHolding = TokenHolding::try_from(&holding_post.account().data).unwrap();
    match holding {
        TokenHolding::Fungible { balance, .. } => assert_eq!(balance, 500),
        _ => panic!("wrong variant"),
    }

    // Rotate to new admin
    let def_acct = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: def_id(),
    };
    let posts = rotate_authority(def_acct, signer(admin_id()), new_admin_id());
    let [def_post] = posts.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { authority, .. } => {
            assert_eq!(authority.admin(), Some(new_admin_id()));
        }
        _ => panic!("wrong variant"),
    }

    // New admin mints 300
    let def_acct = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: def_id(),
    };
    let posts = mint_with_authority(def_acct, make_holding(500), signer(new_admin_id()), 300);
    let [def_post, holding_post] = posts.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { total_supply, .. } => {
            assert_eq!(*total_supply, 1800);
        }
        _ => panic!("wrong variant"),
    }
    let holding: TokenHolding = TokenHolding::try_from(&holding_post.account().data).unwrap();
    match holding {
        TokenHolding::Fungible { balance, .. } => assert_eq!(balance, 800),
        _ => panic!("wrong variant"),
    }

    // Revoke
    let def_acct = AccountWithMetadata {
        account: def_post.account().clone(),
        is_authorized: true,
        account_id: def_id(),
    };
    let posts = revoke_authority(def_acct, signer(new_admin_id()));
    let [def_post] = posts.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { authority, .. } => {
            assert!(authority.is_renounced());
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
#[should_panic(expected = "Renounced")]
fn mint_after_revoke_is_rejected() {
    let def = make_def(1000, Authority::renounced());
    let _posts = mint_with_authority(def, uninit_account(holding_id()), signer(admin_id()), 100);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn wrong_signer_is_rejected() {
    let def = make_def(1000, Authority::new(admin_id()));
    let _posts = mint_with_authority(
        def,
        uninit_account(holding_id()),
        signer(new_admin_id()),
        100,
    );
}

#[test]
#[should_panic(expected = "Authority signer authorization is missing")]
fn unsigned_authority_is_rejected() {
    let def = make_def(1000, Authority::new(admin_id()));
    let _posts = mint_with_authority(
        def,
        uninit_account(holding_id()),
        signer_unauthorized(admin_id()),
        100,
    );
}

#[test]
fn burn_works_on_authority_tokens() {
    let def = make_def(1000, Authority::new(admin_id()));
    let holding = make_holding(500);

    let posts = burn(def, holding, 200);
    let [def_post, holding_post] = posts.try_into().unwrap();

    let def: TokenDefinition = TokenDefinition::try_from(&def_post.account().data).unwrap();
    match &def {
        TokenDefinition::FungibleWithAuthority { total_supply, .. } => {
            assert_eq!(*total_supply, 800);
        }
        _ => panic!("wrong variant"),
    }
    let holding: TokenHolding = TokenHolding::try_from(&holding_post.account().data).unwrap();
    match holding {
        TokenHolding::Fungible { balance, .. } => assert_eq!(balance, 300),
        _ => panic!("wrong variant"),
    }
}
