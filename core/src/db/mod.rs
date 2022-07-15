pub mod accounts;
pub mod screen_names;
pub mod table;
pub mod util;

use accounts::AccountTable;
use chrono::NaiveDate;
use screen_names::ScreenNameTable;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
pub use table::{Mode, ReadOnly, Table, Writeable};

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
    #[error("Channel send error")]
    ChannelSend,
    #[error("Channel receive error")]
    ChannelRecv(#[from] std::sync::mpsc::RecvError),
}

pub struct Database<M> {
    pub accounts: Arc<AccountTable<M>>,
    pub screen_names: ScreenNameTable<M>,
}

impl<M: Sync + Send + 'static> Database<M> {
    pub fn get_counts(
        &self,
    ) -> Result<
        (
            accounts::AccountTableCounts,
            screen_names::ScreenNameTableCounts,
        ),
        Error,
    > {
        let (tx, rx) = std::sync::mpsc::channel();
        let accounts = self.accounts.clone();

        std::thread::spawn(move || {
            tx.send(accounts.get_counts())
                .map_err(|_| Error::ChannelSend)
        });

        let screen_name_counts = self.screen_names.get_counts()?;
        let account_counts = rx.recv()??;

        Ok((account_counts, screen_name_counts))
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

    pub fn limited_lookup_by_user_id(
        &self,
        user_id: u64,
        earliest: Option<NaiveDate>,
    ) -> Result<HashMap<String, Vec<NaiveDate>>, Error> {
        match earliest {
            Some(earliest) => self.accounts.limited_lookup(user_id, earliest),
            None => self.accounts.lookup(user_id),
        }
    }
}

impl<M: Mode> Database<M> {
    pub fn open<P: AsRef<Path>>(base: P) -> Result<Self, Error> {
        Self::open_from_tables(
            base.as_ref().join("accounts"),
            base.as_ref().join("screen-names"),
        )
    }

    fn open_from_tables<P: AsRef<Path>>(
        accounts_path: P,
        screen_names_path: P,
    ) -> Result<Self, Error> {
        Ok(Self {
            accounts: Arc::new(AccountTable::open(accounts_path)?),
            screen_names: ScreenNameTable::open(screen_names_path)?,
        })
    }
}

impl Database<Writeable> {
    pub fn insert(&self, id: u64, screen_name: &str, dates: Vec<NaiveDate>) -> Result<(), Error> {
        self.accounts.insert(id, screen_name, dates)?;
        self.screen_names.insert(screen_name, id)?;
        Ok(())
    }

    pub fn rebuild_index(&mut self) -> Result<(), Error> {
        self.screen_names.rebuild(&self.accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn insert() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir).unwrap();
        db.insert(123, "foo", vec![]).unwrap();
        db.insert(123, "bar", vec![]).unwrap();
        db.insert(456, "foo", vec![]).unwrap();
        db.insert(123, "foo", vec![]).unwrap();

        let mut expected_by_id = HashMap::new();
        expected_by_id.insert("foo".to_string(), vec![]);
        expected_by_id.insert("bar".to_string(), vec![]);

        let expected_pairs = vec![
            (123, "bar".to_string(), vec![]),
            (123, "foo".to_string(), vec![]),
            (456, "foo".to_string(), vec![]),
        ];

        let expected_counts = (
            accounts::AccountTableCounts {
                id_count: 2,
                pair_count: 3,
            },
            screen_names::ScreenNameTableCounts {
                screen_name_count: 2,
                mapping_count: 3,
            },
        );

        assert_eq!(db.lookup_by_screen_name("foo").unwrap(), vec![123, 456]);
        assert_eq!(db.lookup_by_user_id(123).unwrap(), expected_by_id);
        assert_eq!(db.get_counts().unwrap(), expected_counts);
        assert_eq!(
            db.accounts.pairs().collect::<Result<Vec<_>, _>>().unwrap(),
            expected_pairs
        );

        db.accounts.compact_ranges().unwrap();

        assert_eq!(db.lookup_by_screen_name("foo").unwrap(), vec![123, 456]);
        assert_eq!(db.lookup_by_user_id(123).unwrap(), expected_by_id);
        assert_eq!(db.get_counts().unwrap(), expected_counts);
        assert_eq!(
            db.accounts.pairs().collect::<Result<Vec<_>, _>>().unwrap(),
            expected_pairs
        );
    }

    #[test]
    fn lookup_by_screen_name_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir).unwrap();
        db.insert(123, "foo", vec![]).unwrap();
        db.insert(123, "bar", vec![]).unwrap();
        db.insert(1000, "for", vec![]).unwrap();
        db.insert(1001, "baz", vec![]).unwrap();
        db.insert(1002, "follow", vec![]).unwrap();
        db.insert(1003, "FOR", vec![]).unwrap();

        let expected = vec![
            ("follow".to_string(), vec![1002]),
            ("foo".to_string(), vec![123]),
            ("for".to_string(), vec![1000, 1003]),
        ];

        assert_eq!(
            db.lookup_by_screen_name_prefix("fo", 128).unwrap(),
            expected
        );
    }
}
