pub mod accounts;
pub mod screen_names;
pub mod table;
pub mod util;

use accounts::AccountTable;
use chrono::NaiveDate;
use screen_names::ScreenNameTable;
use std::collections::HashMap;
use std::path::Path;
pub use table::Table;

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
    pub accounts: AccountTable,
    pub screen_names: ScreenNameTable,
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

    pub fn get_counts(
        &self,
    ) -> Result<
        (
            accounts::AccountTableCounts,
            screen_names::ScreenNameTableCounts,
        ),
        Error,
    > {
        let account_counts = self.accounts.get_counts()?;
        let screen_name_counts = self.screen_names.get_counts()?;

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

    pub fn insert(&self, id: u64, screen_name: &str, dates: Vec<NaiveDate>) -> Result<(), Error> {
        self.accounts.insert(id, screen_name, dates)?;
        self.screen_names.insert(screen_name, id)?;
        Ok(())
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
