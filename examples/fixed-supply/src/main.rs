//! Fixed-supply token example.
//!
//! Demonstrates: create a token with authority, mint the total supply, then
//! permanently revoke the authority so no further minting is possible.

use lez_approval::Authority;
use nssa_core::account::AccountId;
use token_authority_core::TokenDefinition;

fn main() {
    let admin = AccountId::new([1; 32]);

    println!("=== Fixed-Supply Token Example ===\n");

    // Step 1: Create token with authority
    let definition = TokenDefinition::FungibleWithAuthority {
        name: "FIXED".into(),
        total_supply: 1_000_000,
        metadata_id: None,
        authority: Authority::new(admin),
    };
    println!("[1] Created token 'FIXED' with supply 1,000,000");
    println!(
        "    Authority: {:?}",
        definition.authority().unwrap().admin()
    );

    // Step 2: Authority gates minting
    let authority = definition.authority().unwrap();
    authority.gate(admin);
    println!("[2] Authority gate check passed for admin");

    // Step 3: Revoke authority
    let mut definition = definition;
    let auth = definition.authority_mut().unwrap();
    auth.revoke(admin);
    println!("[3] Authority revoked — supply permanently fixed at 1,000,000");

    // Step 4: Verify minting is blocked
    let authority = definition.authority().unwrap();
    let result = std::panic::catch_unwind(|| {
        authority.gate(admin);
    });
    assert!(result.is_err());
    println!("[4] Mint attempt after revocation correctly rejected");

    println!("\n=== Fixed-Supply Token Example Complete ===");
}
