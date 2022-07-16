use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

mod authorization;
mod token;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Provider {
    GitHub,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Identity {
    GitHub { id: u64 },
}

impl Identity {
    pub fn provider(&self) -> Provider {
        match self {
            Identity::GitHub { .. } => Provider::GitHub,
        }
    }

    fn from_pair(provider: &str, id: &str) -> Result<Identity, Error> {
        match provider {
            "github" => {
                let id = id
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidIdentifier(id.to_string()))?;
                Ok(Identity::GitHub { id })
            }
            other => Err(Error::InvalidProvider(other.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Access {
    Admin,
    Full,
}

impl FromStr for Access {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Self::Admin),
            "full" => Ok(Self::Full),
            other => Err(Error::InvalidAccess(other.to_string())),
        }
    }
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Authorization {
    LoggedOut,
    LoggedIn { provider: Provider },
    Authorized { provider: Provider, access: Access },
}

impl Authorization {
    pub fn provider(&self) -> Option<Provider> {
        match self {
            Authorization::LoggedOut => None,
            Authorization::LoggedIn { provider } => Some(*provider),
            Authorization::Authorized { provider, .. } => Some(*provider),
        }
    }

    pub fn access(&self) -> Option<Access> {
        match self {
            Authorization::Authorized { access, .. } => Some(*access),
            _ => None,
        }
    }
}

impl Default for Authorization {
    fn default() -> Self {
        Self::LoggedOut
    }
}

pub struct Authorizer {
    authorization_db: authorization::AuthorizationDb,
    token_db: token::TokenDb,
}

impl Authorizer {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            authorization_db: authorization::AuthorizationDb::open(path)?,
            token_db: token::TokenDb::default(),
        })
    }

    pub async fn authorize(&self, token_value: &str) -> Result<Authorization, Error> {
        let token = token_value.parse::<token::Token>()?;
        Ok(match self.token_db.lookup(&token).await? {
            Some(identity) => match self.authorization_db.lookup(&identity) {
                Some(access) => Authorization::Authorized {
                    provider: identity.provider(),
                    access,
                },
                None => Authorization::LoggedIn {
                    provider: identity.provider(),
                },
            },
            None => Authorization::LoggedOut,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Invalid Access")]
    InvalidAccess(String),
    #[error("Invalid provider")]
    InvalidProvider(String),
    #[error("Invalid identifier")]
    InvalidIdentifier(String),
    #[error("Invalid authorization database file")]
    InvalidAuthorizationDb,
    #[error("GitHub API error")]
    GitHubApi(reqwest::Error),
    #[error("Invalid token")]
    InvalidToken(String),
}
