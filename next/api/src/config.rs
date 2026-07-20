use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;
use std::net::IpAddr;
use std::path::Path;

/// OAuth client credentials for a single provider.
#[derive(Clone, Debug, Deserialize)]
pub struct ProviderConfig {
    /// The OAuth client (or consumer) ID.
    pub client_id: String,
    /// The OAuth client (or consumer) secret.
    pub client_secret: String,
    /// The callback URI registered with the provider (e.g. `https://memory.lol/auth/github`).
    pub redirect_uri: String,
}

/// Application configuration, loaded from a TOML file with environment overrides.
///
/// Any field can be overridden with a `MEMORY_LOL_`-prefixed environment variable
/// (nested fields use `__` as a separator, e.g. `MEMORY_LOL_GITHUB__CLIENT_SECRET`).
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Path to the account history RocksDB database directory.
    pub db: String,
    /// Path to the authorizations CSV file mapping identities to access levels.
    pub authorization: String,
    /// Path to the SQLite authorization database (created if missing).
    pub auth_db: String,
    /// Optional path to a file of user IDs (one per line) exempt from result limiting.
    pub inclusions: Option<String>,
    /// Optional cookie domain (e.g. `memory.lol`).
    pub domain: Option<String>,
    /// Where to send the browser after a login or logout round trip.
    pub default_login_redirect_uri: String,
    /// Base64-encoded key for private (encrypted) cookies; must decode to at least 64 bytes.
    pub secret_key: String,
    /// Address to bind (defaults to 127.0.0.1).
    #[serde(default = "default_address")]
    pub address: IpAddr,
    /// Port to bind (defaults to 8000).
    #[serde(default = "default_port")]
    pub port: u16,
    /// GitHub OAuth 2.0 credentials.
    pub github: ProviderConfig,
    /// Google OAuth 2.0 credentials.
    pub google: ProviderConfig,
    /// Twitter OAuth 1.0a credentials.
    pub twitter: ProviderConfig,
}

fn default_address() -> IpAddr {
    IpAddr::from([127, 0, 0, 1])
}

const fn default_port() -> u16 {
    8000
}

impl Config {
    /// Load configuration from the given TOML file, then apply environment overrides.
    ///
    /// # Errors
    ///
    /// Fails if the file is invalid or required fields are missing.
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Ok(Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("MEMORY_LOL_").split("__"))
            .extract()?)
    }
}
