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
    #[error("Invalid Snowflake ID")]
    InvalidSnowflake(i64),
    #[error("OAuth 2.0 error")]
    Oauth2(#[from] rocket_oauth2::Error),
    #[error("Authorization error")]
    Authz(#[from] crate::authz::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        match self {
            Error::InvalidSnowflake(_) => Status::NotFound.respond_to(req),
            _ => Status::InternalServerError.respond_to(req),
        }
    }
}
