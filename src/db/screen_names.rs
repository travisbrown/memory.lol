use super::{accounts::AccountTable, table::Table, Error};
use rocksdb::{IteratorMode, MergeOperands, Options, DB};
use std::convert::TryInto;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenNameTableCounts {
    pub screen_name_count: u64,
    pub mapping_count: u64,
}

pub struct ScreenNameTable {
    db: Option<DB>,
}

impl Table for ScreenNameTable {
    type Counts = ScreenNameTableCounts;

    fn underlying(&self) -> &DB {
        &self.db.as_ref().unwrap()
    }

    fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let options = Self::make_options();
        let db = DB::open(&options, path)?;

        Ok(Self { db: Some(db) })
    }

    fn get_counts(&self) -> Result<Self::Counts, Error> {
        let mut screen_name_count = 0;
        let mut mapping_count = 0;

        let iter = self.db.as_ref().unwrap().iterator(IteratorMode::Start);

        for (_, value) in iter {
            screen_name_count += 1;
            let value_len = value.len();

            if value_len % 8 == 0 {
                mapping_count += value_len as u64 / 8;
            } else {
                return Err(Error::InvalidValue(value.to_vec()));
            }
        }

        Ok(Self::Counts {
            screen_name_count,
            mapping_count,
        })
    }
}

impl ScreenNameTable {
    fn make_options() -> Options {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.set_merge_operator_associative("merge", Self::merge);
        options
    }

    pub fn rebuild(&mut self, accounts: &AccountTable) -> Result<(), Error> {
        let path = self.db.as_ref().unwrap().path().to_path_buf();
        self.db.take().unwrap();

        let options = Self::make_options();

        DB::destroy(&options, &path)?;

        self.db = Some(DB::open(&options, &path)?);

        for pair in accounts.pairs() {
            let (id, screen_name, _) = pair?;

            self.insert(&screen_name, id)?;
        }

        Ok(())
    }

    pub fn lookup(&self, screen_name: &str) -> Result<Vec<u64>, Error> {
        let value = self
            .db
            .as_ref()
            .unwrap()
            .get_pinned(screen_name_to_key(screen_name))?;
        value
            .as_ref()
            .map(|value| value_to_ids(value))
            .unwrap_or_else(|| Ok(vec![]))
    }

    pub fn lookup_by_prefix(
        &self,
        screen_name: &str,
        limit: usize,
    ) -> Result<Vec<(String, Vec<u64>)>, Error> {
        let prefix = screen_name_to_key(screen_name);
        let iter = self.db.as_ref().unwrap().prefix_iterator(&prefix);
        let mut result = Vec::with_capacity(1);

        for (key, value) in iter.take(limit) {
            if key.starts_with(&prefix) {
                let screen_name = key_to_screen_name(&key)?;
                let ids = value_to_ids(&value)?;

                result.push((screen_name.to_string(), ids));
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn insert(&self, screen_name: &str, id: u64) -> Result<(), Error> {
        Ok(self
            .db
            .as_ref()
            .unwrap()
            .merge(screen_name_to_key(screen_name), id.to_be_bytes())?)
    }

    fn merge(
        _new_key: &[u8],
        existing_val: Option<&[u8]>,
        operands: &MergeOperands,
    ) -> Option<Vec<u8>> {
        let mut new_val = match existing_val {
            Some(bytes) => bytes.to_vec(),
            None => Vec::with_capacity(operands.len() * 8),
        };

        for operand in operands.iter() {
            Self::merge_for_screen_name(&mut new_val, operand);
        }

        Some(new_val)
    }

    fn merge_for_screen_name(a: &mut Vec<u8>, b: &[u8]) {
        let original_len = a.len();
        let mut i = 0;

        while i < b.len() {
            let bytes: [u8; 8] = match b[i..i + 8].try_into() {
                Ok(bytes) => bytes,
                Err(error) => {
                    log::error!("{}", error);
                    return;
                }
            };
            let next_b = u64::from_be_bytes(bytes);

            let mut found = false;
            let mut j = 0;

            while !found && j < original_len {
                let bytes = match a[j..j + 8].try_into() {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        log::error!("{}", error);
                        return;
                    }
                };
                let next_a = u64::from_be_bytes(bytes);
                found = next_a == next_b;
                j += 8;
            }

            if !found {
                a.extend_from_slice(&b[i..i + 8]);
            }
            i += 8;
        }
    }
}

fn screen_name_to_key(screen_name: &str) -> Vec<u8> {
    let form = screen_name.to_lowercase();
    form.as_bytes().to_vec()
}

fn key_to_screen_name(key: &[u8]) -> Result<&str, Error> {
    Ok(std::str::from_utf8(key)?)
}

fn value_to_ids(value: &[u8]) -> Result<Vec<u64>, Error> {
    let mut result = Vec::with_capacity(value.len() / 8);
    let mut i = 0;

    while i < value.len() {
        let id = u64::from_be_bytes(
            value[i..i + 8]
                .try_into()
                .map_err(|_| Error::InvalidValue(value.to_vec()))?,
        );
        result.push(id);
        i += 8;
    }

    Ok(result)
}
