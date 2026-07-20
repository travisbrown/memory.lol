//! A [`rusqlite`]-backed implementation of the [`memory_lol_auth::AuthDb`] trait.
//!
//! This crate is the `next`-generation replacement for `memory-lol-auth-sqlx`. It uses
//! [`tokio_rusqlite::Connection`], which runs each query on a dedicated background thread,
//! so the async runtime is never blocked by SQLite work. The connection handle is cheaply
//! cloneable and can be shared across request handlers.

use egg_mode::{KeyPair, Token};
use memory_lol_auth::AuthDb;
use tokio_rusqlite::Connection;
use tokio_rusqlite::rusqlite::{self, OptionalExtension, Params};

/// The authorization database schema, applied idempotently by [`initialize`].
///
/// This mirrors the SQL in the top-level `migrations/` directory, so an existing database
/// created for the legacy `sqlx` stack can be used as-is.
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS github_names(
    id UNSIGNED BIGINT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS google_names(
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS google_names_value ON google_names (value);

CREATE TABLE IF NOT EXISTS twitter_names(
    id UNSIGNED BIGINT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS github_tokens(
    value VARCHAR(255) NOT NULL PRIMARY KEY,
    id UNSIGNED BIGINT NOT NULL,
    gist BOOLEAN NOT NULL,
    FOREIGN KEY (id) REFERENCES github_names (id)
);

CREATE TABLE IF NOT EXISTS google_tokens(
    value VARCHAR(255) NOT NULL PRIMARY KEY,
    id VARCHAR(255) NOT NULL,
    FOREIGN KEY (id) REFERENCES google_names (id)
);

CREATE TABLE IF NOT EXISTS twitter_tokens(
    value VARCHAR(255) NOT NULL PRIMARY KEY,
    id UNSIGNED BIGINT NOT NULL,
    consumer_secret VARCHAR(255) NOT NULL,
    access_key VARCHAR(255) NOT NULL,
    access_secret VARCHAR(255) NOT NULL,
    FOREIGN KEY (id) REFERENCES twitter_names (id)
);
";

/// Open a connection to the authorization database and apply the schema if needed.
///
/// # Arguments
///
/// * `path` - Path to the SQLite database file (created if missing)
///
/// # Errors
///
/// Returns [`Error::Sqlite`] if the database cannot be opened or the schema cannot be applied.
pub async fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Connection, Error> {
    let connection = Connection::open(path)
        .await
        .map_err(tokio_rusqlite::Error::from)?;
    initialize(&connection).await?;

    Ok(connection)
}

/// Apply the authorization database schema and pragmas to an open connection.
///
/// Safe to call on a database that already has the schema (all statements are idempotent).
///
/// # Errors
///
/// Returns [`Error::Sqlite`] if any statement fails.
pub async fn initialize(connection: &Connection) -> Result<(), Error> {
    connection
        .call(|connection| {
            // WAL allows concurrent readers while a write is in progress.
            connection.pragma_update(None, "journal_mode", "WAL")?;
            connection.pragma_update(None, "foreign_keys", "ON")?;
            connection.execute_batch(SCHEMA)
        })
        .await?;

    Ok(())
}

/// Run a single-column, single-row query, returning `None` if there is no matching row.
async fn query_optional<T, P>(
    connection: &Connection,
    sql: &'static str,
    params: P,
) -> Result<Option<T>, Error>
where
    // `FromSql` is rusqlite's deserialization trait; the `Send + 'static` bounds are
    // required because the closure runs on the connection's background thread.
    T: rusqlite::types::FromSql + Send + 'static,
    P: Params + Send + 'static,
{
    Ok(connection
        .call(move |connection| {
            connection
                .query_row(sql, params, |row| row.get(0))
                .optional()
        })
        .await?)
}

/// Run a statement that returns no rows.
async fn execute<P: Params + Send + 'static>(
    connection: &Connection,
    sql: &'static str,
    params: P,
) -> Result<(), Error> {
    connection
        .call(move |connection| connection.execute(sql, params).map(|_| ()))
        .await?;

    Ok(())
}

/// An implementation of [`AuthDb`] backed by SQLite via [`rusqlite`].
///
/// This is a stateless marker type: the trait passes the [`Connection`] into every method.
pub struct RusqliteAuthDb;

#[async_trait::async_trait]
impl AuthDb for RusqliteAuthDb {
    type Connection = Connection;
    type Error = Error;

    async fn get_github_name(
        connection: &mut Self::Connection,
        id: u64,
    ) -> Result<Option<String>, Self::Error> {
        let id = u64_to_i64(id)?;
        query_optional(
            connection,
            "SELECT value FROM github_names WHERE id = ?1",
            [id],
        )
        .await
    }

    async fn get_google_email(
        connection: &mut Self::Connection,
        sub: &str,
    ) -> Result<Option<String>, Self::Error> {
        query_optional(
            connection,
            "SELECT value FROM google_names WHERE id = ?1",
            [sub.to_string()],
        )
        .await
    }

    async fn get_google_sub(
        connection: &mut Self::Connection,
        email: &str,
    ) -> Result<Option<String>, Self::Error> {
        query_optional(
            connection,
            "SELECT id FROM google_names WHERE value = ?1",
            [email.to_string()],
        )
        .await
    }

    async fn get_twitter_name(
        connection: &mut Self::Connection,
        id: u64,
    ) -> Result<Option<String>, Self::Error> {
        let id = u64_to_i64(id)?;
        query_optional(
            connection,
            "SELECT value FROM twitter_names WHERE id = ?1",
            [id],
        )
        .await
    }

    async fn put_github_name(
        connection: &mut Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        execute(
            connection,
            "REPLACE INTO github_names (id, value) VALUES (?1, ?2)",
            (id, value.to_string()),
        )
        .await
    }

    async fn put_google_email(
        connection: &mut Self::Connection,
        sub: &str,
        value: &str,
    ) -> Result<(), Self::Error> {
        execute(
            connection,
            "REPLACE INTO google_names (id, value) VALUES (?1, ?2)",
            (sub.to_string(), value.to_string()),
        )
        .await
    }

    async fn put_twitter_name(
        connection: &mut Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        execute(
            connection,
            "REPLACE INTO twitter_names (id, value) VALUES (?1, ?2)",
            (id, value.to_string()),
        )
        .await
    }

    async fn lookup_github_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<(u64, bool)>, Self::Error> {
        let token = token.to_string();

        Ok(connection
            .call(move |connection| {
                connection
                    .query_row(
                        "SELECT id, gist FROM github_tokens WHERE value = ?1",
                        [token],
                        |row| Ok((row.get::<_, i64>(0)? as u64, row.get(1)?)),
                    )
                    .optional()
            })
            .await?)
    }

    async fn lookup_google_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<(String, String)>, Self::Error> {
        let token = token.to_string();

        Ok(connection
            .call(move |connection| {
                connection
                    .query_row(
                        "SELECT google_tokens.id, google_names.value
                            FROM google_tokens
                            JOIN google_names ON google_names.id = google_tokens.id
                            WHERE google_tokens.value = ?1",
                        [token],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
            })
            .await?)
    }

    async fn lookup_twitter_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<u64>, Self::Error> {
        let result: Option<i64> = query_optional(
            connection,
            "SELECT id FROM twitter_tokens WHERE value = ?1",
            [token.to_string()],
        )
        .await?;

        Ok(result.map(|id| id as u64))
    }

    async fn get_twitter_access_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<Token>, Self::Error> {
        let token = token.to_string();

        Ok(connection
            .call(move |connection| {
                connection
                    .query_row(
                        "SELECT consumer_secret, access_key, access_secret
                            FROM twitter_tokens
                            WHERE value = ?1",
                        [token.clone()],
                        |row| {
                            Ok(Token::Access {
                                consumer: KeyPair::new(token.clone(), row.get::<_, String>(0)?),
                                access: KeyPair::new(
                                    row.get::<_, String>(1)?,
                                    row.get::<_, String>(2)?,
                                ),
                            })
                        },
                    )
                    .optional()
            })
            .await?)
    }

    async fn put_github_token(
        connection: &mut Self::Connection,
        token: &str,
        id: u64,
        gist: bool,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        execute(
            connection,
            // `REPLACE` (rather than the legacy stack's plain `INSERT`) makes repeated
            // logins idempotent: some providers re-issue the same access token.
            "REPLACE INTO github_tokens (value, id, gist) VALUES (?1, ?2, ?3)",
            (token.to_string(), id, gist),
        )
        .await
    }

    async fn put_google_token(
        connection: &mut Self::Connection,
        token: &str,
        sub: &str,
    ) -> Result<(), Self::Error> {
        execute(
            connection,
            "REPLACE INTO google_tokens (value, id) VALUES (?1, ?2)",
            (token.to_string(), sub.to_string()),
        )
        .await
    }

    async fn put_twitter_token(
        connection: &mut Self::Connection,
        token: &str,
        id: u64,
        consumer_secret: &str,
        access_key: &str,
        access_secret: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        execute(
            connection,
            "REPLACE INTO twitter_tokens (value, id, consumer_secret, access_key, access_secret)
                VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                token.to_string(),
                id,
                consumer_secret.to_string(),
                access_key.to_string(),
                access_secret.to_string(),
            ),
        )
        .await
    }
}

/// Convert an unsigned ID to the signed representation SQLite stores.
fn u64_to_i64(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::InvalidId(value))
}

/// Errors that can occur when reading or writing the authorization database.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An underlying SQLite error.
    #[error("SQLite error")]
    Sqlite(#[from] tokio_rusqlite::Error),
    /// An ID too large to store as a SQLite integer.
    #[error("Invalid ID")]
    InvalidId(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn open_test_db() -> Connection {
        let connection = Connection::open_in_memory()
            .await
            .expect("failed to open in-memory database");
        initialize(&connection)
            .await
            .expect("failed to initialize schema");
        connection
    }

    #[tokio::test]
    async fn github_name_and_token_round_trip() {
        let mut connection = open_test_db().await;

        RusqliteAuthDb::put_github_name(&mut connection, 123, "octocat")
            .await
            .expect("put name");
        RusqliteAuthDb::put_github_token(&mut connection, "token-1", 123, true)
            .await
            .expect("put token");

        assert_eq!(
            RusqliteAuthDb::get_github_name(&mut connection, 123)
                .await
                .expect("get name"),
            Some("octocat".to_string())
        );
        assert_eq!(
            RusqliteAuthDb::lookup_github_token(&mut connection, "token-1")
                .await
                .expect("lookup token"),
            Some((123, true))
        );
        assert_eq!(
            RusqliteAuthDb::lookup_github_token(&mut connection, "missing")
                .await
                .expect("lookup missing token"),
            None
        );
    }

    #[tokio::test]
    async fn put_github_token_is_idempotent() {
        let mut connection = open_test_db().await;

        RusqliteAuthDb::put_github_name(&mut connection, 123, "octocat")
            .await
            .expect("put name");

        for _ in 0..2 {
            RusqliteAuthDb::put_github_token(&mut connection, "token-1", 123, false)
                .await
                .expect("put token");
        }

        assert_eq!(
            RusqliteAuthDb::lookup_github_token(&mut connection, "token-1")
                .await
                .expect("lookup token"),
            Some((123, false))
        );
    }

    #[tokio::test]
    async fn google_token_lookup_joins_names() {
        let mut connection = open_test_db().await;

        RusqliteAuthDb::put_google_email(&mut connection, "sub-1", "user@example.com")
            .await
            .expect("put email");
        RusqliteAuthDb::put_google_token(&mut connection, "token-1", "sub-1")
            .await
            .expect("put token");

        assert_eq!(
            RusqliteAuthDb::lookup_google_token(&mut connection, "token-1")
                .await
                .expect("lookup token"),
            Some(("sub-1".to_string(), "user@example.com".to_string()))
        );
        assert_eq!(
            RusqliteAuthDb::get_google_sub(&mut connection, "user@example.com")
                .await
                .expect("get sub"),
            Some("sub-1".to_string())
        );
    }

    #[tokio::test]
    async fn twitter_access_token_round_trip() {
        let mut connection = open_test_db().await;

        RusqliteAuthDb::put_twitter_name(&mut connection, 456, "twitterdev")
            .await
            .expect("put name");
        RusqliteAuthDb::put_twitter_token(
            &mut connection,
            "consumer-key",
            456,
            "consumer-secret",
            "access-key",
            "access-secret",
        )
        .await
        .expect("put token");

        assert_eq!(
            RusqliteAuthDb::lookup_twitter_token(&mut connection, "consumer-key")
                .await
                .expect("lookup token"),
            Some(456)
        );

        let token = RusqliteAuthDb::get_twitter_access_token(&mut connection, "consumer-key")
            .await
            .expect("get access token")
            .expect("token should exist");

        match token {
            Token::Access { consumer, access } => {
                assert_eq!(consumer.key.as_ref(), "consumer-key");
                assert_eq!(consumer.secret.as_ref(), "consumer-secret");
                assert_eq!(access.key.as_ref(), "access-key");
                assert_eq!(access.secret.as_ref(), "access-secret");
            }
            Token::Bearer(_) => panic!("expected an access token"),
        }
    }

    #[tokio::test]
    async fn out_of_range_id_is_rejected() {
        let mut connection = open_test_db().await;

        assert!(matches!(
            RusqliteAuthDb::put_github_name(&mut connection, u64::MAX, "octocat").await,
            Err(Error::InvalidId(_))
        ));
    }
}
