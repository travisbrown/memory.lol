use rocket::{
    http::Status,
    request::Request,
    response::{Responder, Result},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Database error")]
    Db(#[from] memory_lol::db::Error),
    #[error("Profile database error")]
    ProfileDb(#[from] hst_tw_db::Error),
    #[error("Invalid Snowflake ID")]
    InvalidSnowflake(i64),
    #[error("OAuth 2.0 error")]
    Oauth2(#[from] rocket_oauth2::Error),
    #[error("Authorization error")]
    Authorization(#[from] memory_lol_auth::Error<memory_lol_auth_sqlx::Error>),
    #[error("Google OpenID error")]
    GoogleOpenId(#[from] memory_lol_auth::google::Error),
    #[error("Twitter OAuth error")]
    TwitterOAuth(#[from] memory_lol_auth::twitter::Error),
    #[error("Invalid inclusion file line")]
    InvalidInclusionFileLine(String),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        match self {
            Error::InvalidSnowflake(_) => Status::NotFound.respond_to(req),
            _ => Status::InternalServerError.respond_to(req),
        }
    }
}
