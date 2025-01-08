use reqwest::{header::HeaderName, Client, StatusCode};
use serde::Deserialize;

const GITHUB_USER_URL: &str = "https://api.github.com/user";
const GITHUB_ACCEPT_HEADER: &str = "application/vnd.github+json";

pub struct GitHubClient {
    client: Client,
    user_agent: String,
}

impl GitHubClient {
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: Client::default(),
            user_agent: user_agent.to_string(),
        }
    }

    pub async fn get_user_info(&self, token: &str) -> Result<Option<(u64, String, bool)>, Error> {
        let response = self
            .client
            .get(GITHUB_USER_URL)
            .header("User-Agent", &self.user_agent)
            .header("Accept", GITHUB_ACCEPT_HEADER)
            .header("Authorization", format!("token {token}"))
            .send()
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            Ok(None)
        } else {
            let scopes_header_name = HeaderName::from_static("x-oauth-scopes");
            let gist_scope = match response.headers().get(scopes_header_name) {
                Some(scopes) => scopes
                    .to_str()?
                    .split(',')
                    .any(|scope| scope.trim() == "gist"),
                None => false,
            };

            let github_response = response.json::<GitHubResponse>().await?;

            Ok(Some((
                github_response.id,
                github_response.login,
                gist_scope,
            )))
        }
    }
}

#[derive(Deserialize)]
struct GitHubResponse {
    id: u64,
    login: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP client error")]
    Http(#[from] reqwest::Error),
    #[error("HTTP client header error")]
    HttpHeader(#[from] reqwest::header::ToStrError),
}
