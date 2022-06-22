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
    #[error("RocksDb error")]
    Db(#[from] rocksdb::Error),
    #[error("Invalid Twitter epoch day")]
    InvalidDay(i64),
    #[error("Invalid key")]
    InvalidKey(Vec<u8>),
    #[error("Invalid value")]
    InvalidValue(Vec<u8>),
    #[error("Invalid UTF-8 string")]
    InvalidString(#[from] std::str::Utf8Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        Status::InternalServerError.respond_to(req)
    }
}
