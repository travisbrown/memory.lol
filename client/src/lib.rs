use memory_lol::model::Account;

pub mod client;

pub use client::Client;

use chrono::NaiveDate;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Observation {
    pub screen_name: String,
    pub range: Option<(NaiveDate, NaiveDate)>,
}

impl Observation {
    pub fn new(screen_name: String, range: Option<(NaiveDate, NaiveDate)>) -> Self {
        Self { screen_name, range }
    }

    pub fn from_account(account: &Account) -> Vec<Self> {
        account
            .screen_names
            .iter()
            .map(|(screen_name, dates)| {
                let range = if let Some(dates) = dates {
                    if dates.is_empty() {
                        None
                    } else {
                        Some((dates[0], dates[dates.len() - 1]))
                    }
                } else {
                    None
                };

                Self {
                    screen_name: screen_name.to_string(),
                    range,
                }
            })
            .collect()
    }
}
