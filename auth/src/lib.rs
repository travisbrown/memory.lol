use egg_mode::Token;
use serde_json::Value;
use std::marker::PhantomData;
use std::path::Path;

use authorization::Authorizations;
use github::GitHubClient;
use google::GoogleClient;
use twitter::TwitterClient;

mod authorization;
pub mod db;
pub mod github;
pub mod google;
pub mod model;
pub mod twitter;

pub use db::AuthDb;
pub use model::{Access, Authorization, Identity, Provider};

pub struct Authorizer<A> {
    _auth_db: PhantomData<A>,
    authorizations: Authorizations,
    github_client: GitHubClient,
    google_client: GoogleClient,
    twitter_client: TwitterClient,
}

impl<A: AuthDb> Authorizer<A> {
    pub async fn open<P: AsRef<Path>>(
        authorizations_path: P,
        user_agent: &str,
        google_client_id: &str,
        google_client_secret: &str,
        twitter_client_id: &str,
        twitter_client_secret: &str,
        twitter_redirect_uri: &str,
    ) -> Result<Self, Error<A::Error>> {
        Ok(Self {
            _auth_db: PhantomData,
            authorizations: Authorizations::open(authorizations_path)?,
            github_client: GitHubClient::new(user_agent),
            google_client: GoogleClient::new(google_client_id, google_client_secret).await?,
            twitter_client: TwitterClient::new(
                twitter_client_id,
                twitter_client_secret,
                twitter_redirect_uri,
            ),
        })
    }

    pub async fn authorize_github(
        &self,
        connection: &mut A::Connection,
        token: &str,
    ) -> Result<Authorization, Error<A::Error>> {
        if let Some((id, _)) = A::lookup_github_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            let identity = Identity::GitHub { id };
            Ok(match self.authorizations.lookup(&identity) {
                Some(access) => Authorization::Authorized { identity, access },
                None => Authorization::LoggedIn { identity },
            })
        } else {
            Ok(Authorization::LoggedOut)
        }
    }

    pub async fn authorize_google(
        &self,
        connection: &mut A::Connection,
        token: &str,
    ) -> Result<Authorization, Error<A::Error>> {
        if let Some((sub, email)) = A::lookup_google_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            let sub_identity = Identity::Google { sub };
            Ok(match self.authorizations.lookup(&sub_identity) {
                Some(access) => Authorization::Authorized {
                    identity: sub_identity,
                    access,
                },
                None => {
                    let email_identity = Identity::GoogleEmail { email };
                    match self.authorizations.lookup(&email_identity) {
                        Some(access) => Authorization::Authorized {
                            identity: email_identity,
                            access,
                        },
                        None => Authorization::LoggedIn {
                            identity: email_identity,
                        },
                    }
                }
            })
        } else {
            Ok(Authorization::LoggedOut)
        }
    }

    pub async fn authorize_twitter(
        &self,
        connection: &mut A::Connection,
        token: &str,
    ) -> Result<Authorization, Error<A::Error>> {
        if let Some(id) = A::lookup_twitter_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            let identity = Identity::Twitter { id };
            Ok(match self.authorizations.lookup(&identity) {
                Some(access) => Authorization::Authorized { identity, access },
                None => Authorization::LoggedIn { identity },
            })
        } else {
            Ok(Authorization::LoggedOut)
        }
    }

    pub async fn save_github_token(
        &self,
        connection: &mut A::Connection,
        token: &str,
    ) -> Result<bool, Error<A::Error>> {
        match self.github_client.get_user_info(token).await? {
            Some((user_id, username, gist)) => {
                A::put_github_name(connection, user_id, &username)
                    .await
                    .map_err(Error::AuthDb)?;
                A::put_github_token(connection, token, user_id, gist)
                    .await
                    .map_err(Error::AuthDb)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn save_google_token(
        &self,
        connection: &mut A::Connection,
        token: &str,
        value: &Value,
    ) -> Result<bool, Error<A::Error>> {
        let claims = self.google_client.extract_id_token(value)?;
        match claims.userinfo.email {
            Some(email) => {
                A::put_google_email(connection, &claims.sub, &email)
                    .await
                    .map_err(Error::AuthDb)?;
                A::put_google_token(connection, token, &claims.sub)
                    .await
                    .map_err(Error::AuthDb)?;

                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn create_twitter_request_token(&self) -> Result<String, Error<A::Error>> {
        Ok(self.twitter_client.create_request_token().await?)
    }

    pub async fn save_twitter_token(
        &self,
        connection: &mut A::Connection,
        oauth_token: &str,
        oauth_verifier: &str,
    ) -> Result<Option<String>, Error<A::Error>> {
        match self
            .twitter_client
            .get_access_token(oauth_token, oauth_verifier)
            .await?
        {
            Some((Token::Access { consumer, access }, user_id, screen_name)) => {
                A::put_twitter_name(connection, user_id, &screen_name)
                    .await
                    .map_err(Error::AuthDb)?;
                A::put_twitter_token(
                    connection,
                    &consumer.key,
                    user_id,
                    &consumer.secret,
                    &access.key,
                    &access.secret,
                )
                .await
                .map_err(Error::AuthDb)?;

                Ok(Some(consumer.key.to_string()))
            }
            _ => Ok(None),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<E: std::error::Error> {
    #[error("Invalid token")]
    InvalidToken(String),
    #[error("GitHub client error")]
    GitHubClient(#[from] github::Error),
    #[error("Google client error")]
    GoogleClient(#[from] google::Error),
    #[error("Twitter client error")]
    TwitterClient(#[from] twitter::Error),
    #[error("Authorizations file error")]
    Authorizations(#[from] authorization::Error),
    #[error("Auth DB error")]
    AuthDb(E),
}
