use super::authorization::Error;
use std::fmt::Display;
use std::str::FromStr;

pub mod providers {
    use super::{IsProvider, Provider};

    pub struct GitHub;
    pub struct Google;
    pub struct Twitter;

    impl IsProvider for GitHub {
        type Id = u64;

        fn provider() -> Provider {
            Provider::GitHub
        }
    }

    impl IsProvider for Google {
        type Id = String;

        fn provider() -> Provider {
            Provider::Google
        }
    }

    impl IsProvider for Twitter {
        type Id = u64;

        fn provider() -> Provider {
            Provider::Twitter
        }
    }
}

pub trait IsProvider: 'static {
    type Id;

    fn provider() -> Provider;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Provider {
    GitHub,
    Google,
    Twitter,
}

impl Provider {
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::GitHub => "gh",
            Self::Google => "gc",
            Self::Twitter => "tw",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
            Self::Twitter => "twitter",
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(Self::GitHub),
            "google" => Ok(Self::Google),
            "twitter" => Ok(Self::Twitter),
            other => Err(Error::InvalidProvider(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Identity {
    GitHub { id: u64 },
    Google { sub: String },
    GoogleEmail { email: String },
    Twitter { id: u64 },
}

impl Identity {
    pub fn provider(&self) -> Provider {
        match self {
            Identity::GitHub { .. } => Provider::GitHub,
            Identity::Google { .. } => Provider::Google,
            Identity::GoogleEmail { .. } => Provider::Google,
            Identity::Twitter { .. } => Provider::Twitter,
        }
    }

    pub fn for_provider(provider: Provider, id: &str, name: &str) -> Result<Identity, Error> {
        match provider {
            Provider::GitHub => {
                let id = id
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidIdentifier(id.to_string()))?;
                Ok(Identity::GitHub { id })
            }
            Provider::Google => {
                if id.is_empty() {
                    Ok(Identity::GoogleEmail {
                        email: name.to_string(),
                    })
                } else if id.len() <= 255 {
                    Ok(Identity::Google {
                        sub: id.to_string(),
                    })
                } else {
                    Err(Error::InvalidIdentifier(id.to_string()))
                }
            }
            Provider::Twitter => {
                let id = id
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidIdentifier(id.to_string()))?;
                Ok(Identity::Twitter { id })
            }
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Authorization {
    LoggedOut,
    LoggedIn { identity: Identity },
    Authorized { identity: Identity, access: Access },
}

impl Authorization {
    pub fn identity(&self) -> Option<&Identity> {
        match self {
            Authorization::LoggedOut => None,
            Authorization::LoggedIn { identity } => Some(identity),
            Authorization::Authorized { identity, .. } => Some(identity),
        }
    }

    pub fn provider(&self) -> Option<Provider> {
        self.identity().map(|identity| identity.provider())
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
