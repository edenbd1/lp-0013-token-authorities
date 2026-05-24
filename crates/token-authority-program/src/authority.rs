use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, Data},
    program::{AccountPostState, Claim},
};
use token_authority_core::{Authority, TokenDefinition, TokenHolding};

#[must_use]
pub fn new_fungible_definition_with_authority(
    definition_target_account: AccountWithMetadata,
    holding_target_account: AccountWithMetadata,
    name: String,
    initial_supply: u128,
    authority: AccountId,
) -> Vec<AccountPostState> {
    assert_eq!(
        definition_target_account.account,
        Account::default(),
        "Definition target account must have default values"
    );

    assert_eq!(
        holding_target_account.account,
        Account::default(),
        "Holding target account must have default values"
    );

    let token_definition = TokenDefinition::FungibleWithAuthority {
        name,
        total_supply: initial_supply,
        metadata_id: None,
        authority: Authority::new(authority.into_value()),
    };
    let token_holding = TokenHolding::Fungible {
        definition_id: definition_target_account.account_id,
        balance: initial_supply,
    };

    let mut definition_post = definition_target_account.account;
    definition_post.data = Data::from(&token_definition);

    let mut holding_post = holding_target_account.account;
    holding_post.data = Data::from(&token_holding);

    vec![
        AccountPostState::new_claimed(definition_post, Claim::Authorized),
        AccountPostState::new_claimed(holding_post, Claim::Authorized),
    ]
}

#[must_use]
pub fn mint_with_authority(
    definition_account: AccountWithMetadata,
    user_holding_account: AccountWithMetadata,
    amount_to_mint: u128,
) -> Vec<AccountPostState> {
    assert!(
        definition_account.is_authorized,
        "Definition authorization is missing"
    );

    let mut definition = TokenDefinition::try_from(&definition_account.account.data)
        .expect("Token Definition account must be valid");

    let authority = definition
        .authority()
        .expect("Token definition must have an authority");
    authority.gate(definition_account.account_id.into_value());

    let mut holding = if user_holding_account.account == Account::default() {
        TokenHolding::zeroized_from_definition(definition_account.account_id, &definition)
    } else {
        TokenHolding::try_from(&user_holding_account.account.data)
            .expect("Token Holding account must be valid")
    };

    assert_eq!(
        definition_account.account_id,
        holding.definition_id(),
        "Mismatch Token Definition and Token Holding"
    );

    match (&mut definition, &mut holding) {
        (
            TokenDefinition::FungibleWithAuthority { total_supply, .. },
            TokenHolding::Fungible { balance, .. },
        ) => {
            *balance = balance
                .checked_add(amount_to_mint)
                .expect("Balance overflow on minting");

            *total_supply = total_supply
                .checked_add(amount_to_mint)
                .expect("Total supply overflow");
        }
        _ => panic!("MintWithAuthority requires FungibleWithAuthority definition"),
    }

    let mut definition_post = definition_account.account;
    definition_post.data = Data::from(&definition);

    let mut holding_post = user_holding_account.account;
    holding_post.data = Data::from(&holding);

    vec![
        AccountPostState::new(definition_post),
        AccountPostState::new_claimed_if_default(holding_post, Claim::Authorized),
    ]
}

#[must_use]
pub fn rotate_authority(
    definition_account: AccountWithMetadata,
    new_authority: AccountId,
) -> Vec<AccountPostState> {
    assert!(
        definition_account.is_authorized,
        "Definition authorization is missing"
    );

    let mut definition = TokenDefinition::try_from(&definition_account.account.data)
        .expect("Token Definition account must be valid");

    let authority = definition
        .authority_mut()
        .expect("Token definition must have an authority");
    authority.rotate(
        definition_account.account_id.into_value(),
        new_authority.into_value(),
    );

    let mut definition_post = definition_account.account;
    definition_post.data = Data::from(&definition);

    vec![AccountPostState::new(definition_post)]
}

#[must_use]
pub fn revoke_authority(definition_account: AccountWithMetadata) -> Vec<AccountPostState> {
    assert!(
        definition_account.is_authorized,
        "Definition authorization is missing"
    );

    let mut definition = TokenDefinition::try_from(&definition_account.account.data)
        .expect("Token Definition account must be valid");

    let authority = definition
        .authority_mut()
        .expect("Token definition must have an authority");
    authority.revoke(definition_account.account_id.into_value());

    let mut definition_post = definition_account.account;
    definition_post.data = Data::from(&definition);

    vec![AccountPostState::new(definition_post)]
}
