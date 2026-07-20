use crate::{config::Config, error::Error, inclusions::Inclusions};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use memory_lol::db::{Database, table::ReadOnly};
use memory_lol_auth::Authorizer;
use memory_lol_auth_rusqlite::RusqliteAuthDb;
use std::sync::Arc;

/// Shared application state, cheap to clone (all fields are handles).
#[derive(Clone)]
pub struct AppState {
    /// The account history database (read-only RocksDB).
    pub db: Arc<Database<ReadOnly>>,
    /// User IDs exempt from result limiting.
    pub inclusions: Arc<Inclusions>,
    /// The OAuth token verifier and access level resolver.
    pub authorizer: Arc<Authorizer<RusqliteAuthDb>>,
    /// A handle to the SQLite authorization database.
    pub auth_db: tokio_rusqlite::Connection,
    /// A shared HTTP client for OAuth token exchanges.
    pub http: reqwest::Client,
    /// The application configuration.
    pub config: Arc<Config>,
    /// The private cookie encryption key.
    pub key: Key,
}

// Lets the `PrivateCookieJar` extractor obtain the encryption key from our state.
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl AppState {
    /// Run a read-only lookup against the account history database on a blocking thread.
    ///
    /// RocksDB reads are synchronous, so this keeps them off the async reactor.
    ///
    /// # Errors
    ///
    /// Propagates lookup errors, or [`Error::TaskJoin`] if the task panics.
    pub async fn lookup<T, F>(&self, lookup: F) -> Result<T, Error>
    where
        T: Send + 'static,
        F: FnOnce(&Database<ReadOnly>, &Inclusions) -> Result<T, Error> + Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let inclusions = Arc::clone(&self.inclusions);

        tokio::task::spawn_blocking(move || lookup(&db, &inclusions)).await?
    }
}
