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
    Db(#[from] super::db::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        Status::InternalServerError.respond_to(req)
    }
}
