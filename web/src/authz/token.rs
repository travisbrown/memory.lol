use super::{Error, Identity};
use futures_locks::RwLock;
use reqwest::{Client, StatusCode};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

const GITHUB_USER_URL: &str = "https://api.github.com/user";
const GITHUB_ACCEPT_HEADER: &str = "application/vnd.github+json";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Token {
    GitHub(String),
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("gho_") {
            Ok(Token::GitHub(s.to_string()))
        } else {
            Err(Error::InvalidToken(s.to_string()))
        }
    }
}

#[derive(Default)]
pub struct TokenDb {
    client: Client,
    mapping: RwLock<HashMap<Token, Option<Identity>>>,
}

#[derive(Deserialize)]
struct GitHubResponse {
    id: u64,
}

impl TokenDb {
    pub async fn lookup(&self, token: &Token) -> Result<Option<Identity>, Error> {
        let mapping = self.mapping.read().await;

        if let Some(maybe_identity) = mapping.get(token) {
            return Ok(maybe_identity.clone());
        }

        std::mem::drop(mapping);

        match token {
            Token::GitHub(token_str) => {
                let id = self.get_github_id(token_str).await?;
                let maybe_identity = id.map(|id| Identity::GitHub { id });
                let mut mapping = self.mapping.write().await;

                mapping.insert(token.clone(), maybe_identity.clone());
                Ok(maybe_identity)
            }
        }
    }

    async fn get_github_id(&self, token_str: &str) -> Result<Option<u64>, Error> {
        let response = self
            .client
            .get(GITHUB_USER_URL)
            .header("User-Agent", "memory.lol")
            .header("Accept", GITHUB_ACCEPT_HEADER)
            .header("Authorization", format!("token {}", token_str))
            .send()
            .await
            .map_err(Error::GitHubApi)?;

        if response.status() == StatusCode::UNAUTHORIZED {
            Ok(None)
        } else {
            let github_response = response
                .json::<GitHubResponse>()
                .await
                .map_err(Error::GitHubApi)?;

            Ok(Some(github_response.id))
        }
    }
}

