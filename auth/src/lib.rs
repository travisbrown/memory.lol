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
pub use model::{Access, Authorization, Identity, Provider, UserInfo};

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
    ) -> Result<Option<Authorization>, Error<A::Error>> {
        if let Some((id, gist)) = A::lookup_github_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            let identity = Identity::GitHub { id };
            let mut access = self.authorizations.lookup(&identity);

            if gist {
                access |= Access::Gist
            }

            Ok(Some(Authorization::new(identity, access)))
        } else {
            Ok(None)
        }
    }

    pub async fn authorize_google(
        &self,
        connection: &mut A::Connection,
        token: &str,
    ) -> Result<Option<Authorization>, Error<A::Error>> {
        if let Some((sub, email)) = A::lookup_google_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            let sub_identity = Identity::Google { sub };
            let sub_access = self.authorizations.lookup(&sub_identity);

            Ok(Some(if sub_access.is_empty() {
                let email_identity = Identity::GoogleEmail { email };
                let email_access = self.authorizations.lookup(&email_identity);

                if email_access.is_empty() {
                    Authorization::new(sub_identity, sub_access)
                } else {
                    Authorization::new(email_identity, email_access)
                }
            } else {
                Authorization::new(sub_identity, sub_access)
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn authorize_twitter(
        &self,
        connection: &mut A::Connection,
        token: &str,
    ) -> Result<Option<Authorization>, Error<A::Error>> {
        if let Some(id) = A::lookup_twitter_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            let identity = Identity::Twitter { id };
            let access = self.authorizations.lookup(&identity);

            Ok(Some(Authorization::new(identity, access)))
        } else {
            Ok(None)
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

    pub async fn get_user_info(
        &self,
        connection: &mut A::Connection,
        identity: &Identity,
    ) -> Result<Option<UserInfo>, Error<A::Error>> {
        Ok(match identity {
            Identity::GitHub { id } => A::get_github_name(connection, *id)
                .await
                .map_err(Error::AuthDb)?
                .map(|username| UserInfo::GitHub { id: *id, username }),
            Identity::Google { sub } => A::get_google_email(connection, sub)
                .await
                .map_err(Error::AuthDb)?
                .map(|email| UserInfo::Google {
                    sub: sub.to_string(),
                    email,
                }),
            Identity::GoogleEmail { email } => A::get_google_sub(connection, email)
                .await
                .map_err(Error::AuthDb)?
                .map(|sub| UserInfo::Google {
                    sub,
                    email: email.to_string(),
                }),
            Identity::Twitter { id } => A::get_twitter_name(connection, *id)
                .await
                .map_err(Error::AuthDb)?
                .map(|screen_name| UserInfo::Twitter {
                    id: *id,
                    screen_name,
                }),
        })
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
