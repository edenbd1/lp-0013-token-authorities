//! Agnostic single-admin approval library for LEZ programs.
//!
//! Provides a reusable `Authority` primitive with `gate`, `rotate`, and `revoke`
//! operations. Satisfies [RFP-001](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-001-admin-authority-lib.md).
//!
//! # Usage
//!
//! ```
//! use lez_approval::Authority;
//!
//! let admin = [1u8; 32];
//! let mut auth = Authority::new(admin);
//!
//! // Only the admin can call gated operations.
//! auth.gate(admin); // OK
//!
//! // Rotate to a new admin.
//! let new_admin = [2u8; 32];
//! auth.rotate(admin, new_admin);
//!
//! // Permanently revoke — terminal, cannot be reversed.
//! auth.revoke(new_admin);
//! assert!(auth.is_renounced());
//! ```

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// A 32-byte account identifier, compatible with `nssa_core::account::AccountId`.
pub type AccountId = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Authority(Option<AccountId>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    Unauthorized,
    Renounced,
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "Unauthorized: signer is not the current authority"),
            Self::Renounced => write!(f, "Renounced: authority has been permanently revoked"),
        }
    }
}

impl Authority {
    #[must_use]
    pub fn new(admin: AccountId) -> Self {
        Self(Some(admin))
    }

    #[must_use]
    pub fn renounced() -> Self {
        Self(None)
    }

    #[must_use]
    pub fn is_renounced(&self) -> bool {
        self.0.is_none()
    }

    #[must_use]
    pub fn admin(&self) -> Option<AccountId> {
        self.0
    }

    pub fn gate(&self, signer: AccountId) {
        match self.0 {
            None => panic!("{}", ApprovalError::Renounced),
            Some(admin) => {
                assert!(admin == signer, "{}", ApprovalError::Unauthorized);
            }
        }
    }

    pub fn rotate(&mut self, signer: AccountId, new_admin: AccountId) {
        self.gate(signer);
        self.0 = Some(new_admin);
    }

    pub fn revoke(&mut self, signer: AccountId) {
        self.gate(signer);
        self.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_id() -> AccountId {
        [1; 32]
    }

    fn other_id() -> AccountId {
        [2; 32]
    }

    fn new_admin_id() -> AccountId {
        [3; 32]
    }

    #[test]
    fn new_authority_is_active() {
        let auth = Authority::new(admin_id());
        assert!(!auth.is_renounced());
        assert_eq!(auth.admin(), Some(admin_id()));
    }

    #[test]
    fn renounced_authority() {
        let auth = Authority::renounced();
        assert!(auth.is_renounced());
        assert_eq!(auth.admin(), None);
    }

    #[test]
    fn gate_succeeds_for_admin() {
        let auth = Authority::new(admin_id());
        auth.gate(admin_id());
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn gate_fails_for_wrong_signer() {
        let auth = Authority::new(admin_id());
        auth.gate(other_id());
    }

    #[test]
    #[should_panic(expected = "Renounced")]
    fn gate_fails_when_renounced() {
        let auth = Authority::renounced();
        auth.gate(admin_id());
    }

    #[test]
    fn rotate_succeeds() {
        let mut auth = Authority::new(admin_id());
        auth.rotate(admin_id(), new_admin_id());
        assert_eq!(auth.admin(), Some(new_admin_id()));
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn rotate_fails_for_wrong_signer() {
        let mut auth = Authority::new(admin_id());
        auth.rotate(other_id(), new_admin_id());
    }

    #[test]
    #[should_panic(expected = "Renounced")]
    fn rotate_fails_when_renounced() {
        let mut auth = Authority::renounced();
        auth.rotate(admin_id(), new_admin_id());
    }

    #[test]
    fn revoke_succeeds() {
        let mut auth = Authority::new(admin_id());
        auth.revoke(admin_id());
        assert!(auth.is_renounced());
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn revoke_fails_for_wrong_signer() {
        let mut auth = Authority::new(admin_id());
        auth.revoke(other_id());
    }

    #[test]
    #[should_panic(expected = "Renounced")]
    fn revoke_fails_when_already_renounced() {
        let mut auth = Authority::renounced();
        auth.revoke(admin_id());
    }

    #[test]
    fn rotate_then_old_admin_rejected() {
        let mut auth = Authority::new(admin_id());
        auth.rotate(admin_id(), new_admin_id());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            auth.gate(admin_id());
        }));
        assert!(result.is_err());
    }

    #[test]
    fn rotate_then_new_admin_accepted() {
        let mut auth = Authority::new(admin_id());
        auth.rotate(admin_id(), new_admin_id());
        auth.gate(new_admin_id());
    }
}
