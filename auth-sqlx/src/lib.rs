use egg_mode::{KeyPair, Token};
use memory_lol_auth::AuthDb;
use sqlx::SqliteConnection;
use std::convert::TryFrom;

pub struct SqlxAuthDb;

#[async_trait::async_trait]
impl AuthDb for SqlxAuthDb {
    type Connection = SqliteConnection;
    type Error = Error;

    async fn get_github_name(
        connection: &mut Self::Connection,
        id: u64,
    ) -> Result<Option<String>, Self::Error> {
        let id = u64_to_i64(id)?;
        Ok(
            sqlx::query_scalar!("SELECT value FROM github_names WHERE id = ?", id)
                .fetch_optional(connection)
                .await?,
        )
    }

    async fn get_google_email(
        connection: &mut Self::Connection,
        sub: &str,
    ) -> Result<Option<String>, Self::Error> {
        Ok(
            sqlx::query_scalar!("SELECT value FROM google_names WHERE id = ?", sub)
                .fetch_optional(connection)
                .await?,
        )
    }

    async fn get_twitter_name(
        connection: &mut Self::Connection,
        id: u64,
    ) -> Result<Option<String>, Self::Error> {
        let id = u64_to_i64(id)?;
        Ok(
            sqlx::query_scalar!("SELECT value FROM twitter_names WHERE id = ?", id)
                .fetch_optional(connection)
                .await?,
        )
    }

    async fn put_github_name(
        connection: &mut Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        sqlx::query!(
            "REPLACE INTO github_names (id, value) VALUES (?, ?)",
            id,
            value
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn put_google_email(
        connection: &mut Self::Connection,
        sub: &str,
        value: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            "REPLACE INTO google_names (id, value) VALUES (?, ?)",
            sub,
            value
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn put_twitter_name(
        connection: &mut Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        sqlx::query!(
            "REPLACE INTO twitter_names (id, value) VALUES (?, ?)",
            id,
            value
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn lookup_github_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<(u64, bool)>, Self::Error> {
        Ok(
            sqlx::query!("SELECT id, gist FROM github_tokens WHERE value = ?", token)
                .fetch_optional(connection)
                .await?,
        )
        .map(|result| result.map(|row| (row.id as u64, row.gist)))
    }

    async fn lookup_google_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<(String, String)>, Self::Error> {
        Ok(sqlx::query!(
            "SELECT google_tokens.id AS sub, google_names.value AS email
                FROM google_tokens
                JOIN google_names ON google_names.id = google_tokens.id
                WHERE google_tokens.value = ?",
            token
        )
        .fetch_optional(connection)
        .await?)
        .map(|result| result.map(|row| (row.sub, row.email)))
    }

    async fn lookup_twitter_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<u64>, Self::Error> {
        Ok(
            sqlx::query_scalar!("SELECT id FROM twitter_tokens WHERE value = ?", token)
                .fetch_optional(connection)
                .await?,
        )
        .map(|result| result.map(|id| id as u64))
    }

    async fn get_twitter_access_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<Token>, Self::Error> {
        Ok(sqlx::query!(
            "SELECT id, consumer_secret, access_key, access_secret
                    FROM twitter_tokens
                    WHERE value = ?",
            token
        )
        .fetch_optional(connection)
        .await?)
        .map(|result| {
            result.map(|row| Token::Access {
                consumer: KeyPair::new(token.to_string(), row.consumer_secret),
                access: KeyPair::new(row.access_key, row.access_secret),
            })
        })
    }

    async fn put_github_token(
        connection: &mut Self::Connection,
        token: &str,
        id: u64,
        gist: bool,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        sqlx::query!(
            "INSERT INTO github_tokens (value, id, gist) VALUES (?, ?, ?)",
            token,
            id,
            gist
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn put_google_token(
        connection: &mut Self::Connection,
        token: &str,
        sub: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            "INSERT INTO google_tokens (value, id) VALUES (?, ?)",
            token,
            sub
        )
        .execute(connection)
        .await?;

        Ok(())
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
        sqlx::query!(
            "INSERT INTO twitter_tokens (value, id, consumer_secret, access_key, access_secret)
                VALUES (?, ?, ?, ?, ?)",
            token,
            id,
            consumer_secret,
            access_key,
            access_secret
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}

fn u64_to_i64(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::InvalidId(value))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SQL error")]
    Sqlx(#[from] sqlx::Error),
    #[error("Invalid ID")]
    InvalidId(u64),
}
