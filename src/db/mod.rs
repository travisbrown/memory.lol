pub mod accounts;
pub mod screen_names;
pub mod table;
pub mod util;

use accounts::AccountTable;
use chrono::NaiveDate;
use screen_names::ScreenNameTable;
use std::collections::HashMap;
use std::path::Path;
use table::Table;

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

pub struct Database {
    accounts: AccountTable,
    screen_names: ScreenNameTable,
}

impl Database {
    pub fn open<P: AsRef<Path>>(base: P) -> Result<Self, Error> {
        Self::open_from_tables(
            base.as_ref().join("accounts"),
            base.as_ref().join("screen-names"),
        )
    }

    pub fn open_from_tables<P: AsRef<Path>>(
        accounts_path: P,
        screen_names_path: P,
    ) -> Result<Self, Error> {
        Ok(Self {
            accounts: AccountTable::open(accounts_path)?,
            screen_names: ScreenNameTable::open(screen_names_path)?,
        })
    }

    pub fn lookup_by_user_id(
        &self,
        user_id: u64,
    ) -> Result<HashMap<String, Vec<NaiveDate>>, Error> {
        self.accounts.lookup(user_id)
    }

    pub fn lookup_by_screen_name(&self, screen_name: &str) -> Result<Vec<u64>, Error> {
        self.screen_names.lookup(screen_name)
    }

    pub fn lookup_by_screen_name_prefix(
        &self,
        screen_name_prefix: &str,
        limit: usize,
    ) -> Result<Vec<(String, Vec<u64>)>, Error> {
        self.screen_names
            .lookup_by_prefix(screen_name_prefix, limit)
    }

    pub fn insert(&self, id: u64, screen_name: &str, dates: Vec<NaiveDate>) -> Result<(), Error> {
        self.accounts.insert(id, screen_name, dates)?;
        self.screen_names.insert(screen_name, id)?;
        Ok(())
    }

    pub fn remove(&self, id: u64, screen_name: &str) -> Result<(), Error> {
        self.accounts.remove(id, screen_name)
    }

    pub fn compact_ranges(&self) -> Result<(), Error> {
        self.accounts.compact_ranges()
    }
}
