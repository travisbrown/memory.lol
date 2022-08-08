use super::Observation;
use memory_lol::model::{Account, ScreenNameResult};
use reqwest::Url;
use std::collections::HashMap;

const MEMORY_LOL_BASE: &str = "https://api.memory.lol/v1/";

lazy_static::lazy_static! {
    pub static ref MEMORY_LOL_BASE_URL: Url = Url::parse(MEMORY_LOL_BASE).unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP client error")]
    HttpClient(#[from] reqwest::Error),
    #[error("Invalid URL error")]
    Url(#[from] url::ParseError),
    #[error("Invalid date range")]
    InvalidDateRange(Vec<String>),
}

pub struct Client {
    base: Url,
}

impl Client {
    pub fn new(base: &Url) -> Self {
        Self { base: base.clone() }
    }

    pub async fn lookup_tw_user_id(&self, user_id: u64) -> Result<Vec<Observation>, Error> {
        let url = self.base.join(&format!("tw/id/{}", user_id))?;
        let account = reqwest::get(url).await?.json::<Account>().await?;

        Ok(Observation::from_account(&account))
    }

    pub async fn lookup_tw_screen_name(
        &self,
        screen_name: &str,
    ) -> Result<HashMap<u64, Vec<Observation>>, Error> {
        let url = self.base.join(&format!("tw/{}", screen_name))?;
        let accounts = reqwest::get(url).await?.json::<ScreenNameResult>().await?;

        Ok(accounts
            .accounts
            .into_iter()
            .map(|account| (account.id, Observation::from_account(&account)))
            .collect())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new(&MEMORY_LOL_BASE_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Observation;
    use super::*;
    use chrono::NaiveDate;
    use serial_test::serial;
    use tokio::time::Duration;

    #[tokio::test]
    #[serial]
    async fn lookup_tw_user_id() {
        let client = Client::default();

        let result = client.lookup_tw_user_id(1015295486612291585).await.unwrap();

        assert_eq!(result.len(), 19);
    }

    #[tokio::test]
    #[serial]
    #[cfg(not(tarpaulin))]
    async fn lookup_tw_screen_name() {
        tokio::time::sleep(Duration::from_millis(2000)).await;

        let client = Client::default();

        let result = client
            .lookup_tw_screen_name("ConceptualJames")
            .await
            .unwrap();
        let expected = vec![(
            826261914,
            vec![
                Observation::new(
                    "GodDoesnt".to_string(),
                    Some((
                        NaiveDate::from_ymd(2013, 01, 04),
                        NaiveDate::from_ymd(2018, 07, 28),
                    )),
                ),
                Observation::new(
                    "ConceptualJames".to_string(),
                    Some((
                        NaiveDate::from_ymd(2018, 07, 29),
                        NaiveDate::from_ymd(2022, 08, 05),
                    )),
                ),
            ],
        )]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }
}
