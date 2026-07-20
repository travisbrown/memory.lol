use memory_lol::model::Account;
use serde::{Deserialize, Serialize};

/// A set of accounts matching a screen name query.
#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub struct ExtendedScreenNameResult {
    /// The matching accounts, oldest first.
    pub accounts: Vec<ExtendedAccount>,
}

impl ExtendedScreenNameResult {
    /// Check (case-insensitively) whether any account has used the given screen name.
    pub fn includes_screen_name(&self, screen_name: &str) -> bool {
        let target_screen_name = screen_name.to_lowercase();
        self.accounts.iter().any(|account| {
            account
                .screen_names
                .keys()
                .any(|screen_name| screen_name.to_lowercase() == target_screen_name)
        })
    }
}

/// An account's screen name history, with the ID duplicated as a string.
///
/// The string form exists because JavaScript clients cannot represent 64-bit IDs exactly.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ExtendedAccount {
    /// The account's numeric Twitter ID.
    pub id: u64,
    /// The account's Twitter ID as a decimal string.
    pub id_str: String,
    /// Observed screen names mapped to their first and last observation dates.
    pub screen_names: indexmap::IndexMap<String, Option<Vec<chrono::NaiveDate>>>,
}

impl From<Account> for ExtendedAccount {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            id_str: account.id.to_string(),
            screen_names: account.screen_names,
        }
    }
}

/// The form body accepted by the POST lookup endpoints for token-based access.
#[derive(Debug, Deserialize)]
pub struct WithToken {
    /// A GitHub personal or OAuth access token.
    pub token: String,
}
