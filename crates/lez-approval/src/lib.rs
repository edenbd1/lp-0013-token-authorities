use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::account::AccountId;
use serde::{Deserialize, Serialize};

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
        AccountId::new([1; 32])
    }

    fn other_id() -> AccountId {
        AccountId::new([2; 32])
    }

    fn new_admin_id() -> AccountId {
        AccountId::new([3; 32])
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
