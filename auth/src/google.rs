use openid::{Client, IdToken, Jws, StandardClaims};
use serde_json::Value;
use url::Url;

const ISSUER_URL: &str = "https://accounts.google.com";
const ID_TOKEN_KEY: &str = "id_token";

pub struct GoogleClient {
    client: Client,
}

impl GoogleClient {
    pub async fn new(client_id: &str, client_secret: &str) -> Result<Self, Error> {
        let client = Client::discover(
            client_id.to_string(),
            client_secret.to_string(),
            None,
            Url::parse(ISSUER_URL).unwrap(),
        )
        .await?;

        Ok(Self { client })
    }

    pub fn decode_id_token(&self, value: &str) -> Result<StandardClaims, Error> {
        let mut id_token = IdToken::<StandardClaims>::new_encoded(value);
        self.client.decode_token(&mut id_token)?;

        match id_token {
            Jws::Decoded { header: _, payload } => Ok(payload),
            Jws::Encoded(_) => Err(Error::from(openid::error::Jose::UnsupportedOperation)),
        }
    }

    pub fn extract_id_token(&self, value: &Value) -> Result<StandardClaims, Error> {
        let id_token_str = value
            .get(ID_TOKEN_KEY)
            .and_then(|value| value.as_str())
            .ok_or_else(|| Error::MissingIdTokenField(value.clone()))?;

        self.decode_id_token(id_token_str)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OpenID error")]
    OpenId(#[from] openid::error::Error),
    #[error("OpenID JOSE error")]
    OpenIdJose(#[from] openid::error::Jose),
    #[error("Missing ID token field")]
    MissingIdTokenField(Value),
}
