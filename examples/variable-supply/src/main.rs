//! Variable-supply token example.
//!
//! Demonstrates: create a token with authority, mint tokens, rotate authority
//! to a new admin, new admin mints more. Old admin is rejected.

use lez_approval::Authority;

use token_authority_core::TokenDefinition;

fn main() {
    let admin_a = [1u8; 32];
    let admin_b = [2u8; 32];

    println!("=== Variable-Supply Token Example ===\n");

    // Step 1: Create token with authority A
    let mut definition = TokenDefinition::FungibleWithAuthority {
        name: "VARIABLE".into(),
        total_supply: 1_000,
        metadata_id: None,
        authority: Authority::new(admin_a),
    };
    println!("[1] Created token 'VARIABLE' with supply 1,000");

    // Step 2: Admin A can mint
    definition.authority().unwrap().gate(admin_a);
    println!("[2] Admin A gate check passed — can mint");

    // Step 3: Rotate authority to Admin B
    definition.authority_mut().unwrap().rotate(admin_a, admin_b);
    println!("[3] Authority rotated from A to B");

    // Step 4: Admin B can now mint
    definition.authority().unwrap().gate(admin_b);
    println!("[4] Admin B gate check passed — can mint");

    // Step 5: Admin A is rejected
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        definition.authority().unwrap().gate(admin_a);
    }));
    assert!(result.is_err());
    println!("[5] Admin A correctly rejected after rotation");

    println!("\n=== Variable-Supply Token Example Complete ===");
    println!("Final authority: Admin B");
}
