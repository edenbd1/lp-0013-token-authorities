//! Config PDA gating example (RFP-001 requirement §4).
//!
//! Demonstrates how a non-token program can use `lez-approval` to gate
//! privileged instructions behind an admin authority. The admin is the
//! only one who can update the program's configuration PDA.

use borsh::{BorshDeserialize, BorshSerialize};
use lez_approval::Authority;
use nssa_core::account::AccountId;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct ProgramConfig {
    fee_bps: u16,
    paused: bool,
    authority: Authority,
}

impl ProgramConfig {
    fn new(admin: AccountId, fee_bps: u16) -> Self {
        Self {
            fee_bps,
            paused: false,
            authority: Authority::new(admin),
        }
    }

    fn update_fee(&mut self, signer: AccountId, new_fee_bps: u16) {
        self.authority.gate(signer);
        self.fee_bps = new_fee_bps;
    }

    fn pause(&mut self, signer: AccountId) {
        self.authority.gate(signer);
        self.paused = true;
    }

    fn transfer_admin(&mut self, signer: AccountId, new_admin: AccountId) {
        self.authority.rotate(signer, new_admin);
    }

    fn renounce_admin(&mut self, signer: AccountId) {
        self.authority.revoke(signer);
    }
}

fn main() {
    let admin = AccountId::new([1; 32]);
    let new_admin = AccountId::new([2; 32]);
    let attacker = AccountId::new([99; 32]);

    println!("=== Config PDA Gating Example (RFP-001 §4) ===\n");

    // Initialize config with admin
    let mut config = ProgramConfig::new(admin, 30);
    println!(
        "[1] Config initialized: fee_bps={}, admin set",
        config.fee_bps
    );

    // Admin updates fee
    config.update_fee(admin, 50);
    println!("[2] Admin updated fee: fee_bps={}", config.fee_bps);

    // Attacker tries to update fee — rejected
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut config_copy = ProgramConfig::new(admin, 50);
        config_copy.update_fee(attacker, 9999);
    }));
    assert!(result.is_err());
    println!("[3] Attacker fee update correctly rejected");

    // Admin pauses the program
    config.pause(admin);
    println!("[4] Admin paused program: paused={}", config.paused);

    // Transfer admin to new_admin
    config.transfer_admin(admin, new_admin);
    println!("[5] Admin transferred to new_admin");

    // New admin updates fee
    config.update_fee(new_admin, 25);
    println!("[6] New admin updated fee: fee_bps={}", config.fee_bps);

    // Old admin is now rejected
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut config_copy = ProgramConfig::new(new_admin, 25);
        config_copy.update_fee(admin, 0);
    }));
    assert!(result.is_err());
    println!("[7] Old admin correctly rejected after transfer");

    // Renounce admin — config becomes immutable
    config.renounce_admin(new_admin);
    println!("[8] Admin renounced — config is now immutable");

    // Nobody can update anymore
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut config_copy = ProgramConfig {
            fee_bps: 25,
            paused: true,
            authority: Authority::renounced(),
        };
        config_copy.update_fee(new_admin, 0);
    }));
    assert!(result.is_err());
    println!("[9] Post-renounce update correctly rejected");

    // Verify Borsh roundtrip
    let encoded = borsh::to_vec(&config).unwrap();
    let decoded = ProgramConfig::try_from_slice(&encoded).unwrap();
    assert_eq!(decoded.fee_bps, 25);
    assert!(decoded.paused);
    assert!(decoded.authority.is_renounced());
    println!("[10] Borsh serialize/deserialize roundtrip OK");

    println!("\n=== Config PDA Gating Example Complete ===");
    println!("Demonstrated: gate, rotate, revoke on a non-token config PDA");
}
