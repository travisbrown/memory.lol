use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// All failure modes for request handling.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An I/O error (typically while reading configuration files).
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    /// A JSON serialization error.
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    /// An account history database error.
    #[error("Database error")]
    Db(#[from] memory_lol::db::Error),
    /// A value that is not a valid Twitter Snowflake ID.
    #[error("Invalid Snowflake ID")]
    InvalidSnowflake(i64),
    /// An authorization subsystem error.
    #[error("Authorization error")]
    Authorization(#[from] memory_lol_auth::Error<memory_lol_auth_rusqlite::Error>),
    /// A Twitter OAuth 1.0a error.
    #[error("Twitter OAuth error")]
    TwitterOAuth(#[from] memory_lol_auth::twitter::Error),
    /// A failure while exchanging an OAuth authorization code for a token.
    #[error("OAuth token exchange error")]
    OAuthExchange(#[from] reqwest::Error),
    /// A missing or mismatched OAuth state parameter (possible CSRF).
    #[error("Invalid OAuth state")]
    InvalidOAuthState,
    /// An invalid URL (an internal invariant violation for constant endpoints).
    #[error("URL error")]
    UrlParse(#[from] url::ParseError),
    /// A token exchange response without an access token.
    #[error("Missing access token in OAuth response")]
    MissingAccessToken,
    /// A malformed line in the inclusions file.
    #[error("Invalid inclusion file line")]
    InvalidInclusionFileLine(String),
    /// A panicked or cancelled blocking task.
    #[error("Task join error")]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::InvalidSnowflake(_) => StatusCode::NOT_FOUND,
            Error::InvalidOAuthState => StatusCode::BAD_REQUEST,
            ref error => {
                tracing::error!(%error, "request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}
