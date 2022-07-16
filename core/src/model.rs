use chrono::NaiveDate;
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScreenNameResult {
    pub accounts: Vec<Account>,
}

impl ScreenNameResult {
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub id: u64,
    #[serde(rename = "screen-names")]
    pub screen_names: IndexMap<String, Option<Vec<NaiveDate>>>,
}

impl Account {
    pub fn from_raw_result(id: u64, result: HashMap<String, Vec<NaiveDate>>) -> Self {
        let mut sorted = result
            .into_iter()
            .map(|(screen_name, mut dates)| {
                dates.sort();

                let value = match dates.len() {
                    0 => None,
                    1 => Some(vec![dates[0]]),
                    n => Some(vec![dates[0], dates[n - 1]]),
                };

                (screen_name, value)
            })
            .collect::<IndexMap<_, _>>();

        sorted.sort_by(|screen_name_a, dates_a, screen_name_b, dates_b| {
            dates_a
                .as_ref()
                .and_then(|dates| dates.get(0))
                .cmp(&dates_b.as_ref().and_then(|dates| dates.get(0)))
                .then_with(|| screen_name_a.cmp(screen_name_b))
        });

        Self {
            id,
            screen_names: sorted,
        }
    }
}
