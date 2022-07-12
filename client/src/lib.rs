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
}
