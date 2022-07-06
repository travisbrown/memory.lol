pub mod accounts;
pub mod screen_names;
pub mod table;
mod util;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("RocksDb error")]
    Db(#[from] rocksdb::Error),
    #[error("Invalid UTF-8 string")]
    InvalidString(#[from] std::str::Utf8Error),
    #[error("Invalid key")]
    InvalidKey(Vec<u8>),
    #[error("Invalid value")]
    InvalidValue(Vec<u8>),
    #[error("Invalid Twitter epoch day")]
    InvalidDay(i64),
    #[error("Invalid Twitter screen name")]
    InvalidScreenName(String),
}
