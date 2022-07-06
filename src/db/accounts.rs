use super::{table::Table, util::is_valid_screen_name, Error};
use chrono::{Duration, NaiveDate};
use rocksdb::{DBIterator, IteratorMode, MergeOperands, Options, DB};
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;

pub struct AccountTableCounts {
    pub id_count: u64,
    pub pair_count: u64,
}

pub struct AccountTable {
    db: DB,
}

impl Table for AccountTable {
    type Counts = AccountTableCounts;

    fn underlying(&self) -> &DB {
        &self.db
    }

    fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.set_merge_operator_associative("merge", Self::merge);
        let db = DB::open(&options, path)?;

        Ok(Self { db })
    }

    fn get_counts(&self) -> Result<Self::Counts, Error> {
        let mut pair_count = 0;
        let mut id_count = 0;
        let mut last_id = 0;

        let iter = self.db.iterator(IteratorMode::Start);

        for (key, _) in iter {
            pair_count += 1;
            let id = key_prefix_to_id(&key)?;
            if id != last_id {
                id_count += 1;
                last_id = id;
            }
        }

        Ok(Self::Counts {
            id_count,
            pair_count,
        })
    }
}

impl AccountTable {
    pub fn pairs(&self) -> PairIterator<DBIterator> {
        PairIterator {
            underlying: self.db.iterator(IteratorMode::Start),
        }
    }

    pub fn lookup(&self, id: u64) -> Result<HashMap<String, Vec<NaiveDate>>, Error> {
        let prefix = id_to_key_prefix(id);
        let iter = self.db.prefix_iterator(prefix);
        let mut result = HashMap::new();

        for (key, value) in iter {
            let (next_id, next_screen_name) = key_to_pair(&key)?;
            if next_id == id {
                let dates = value_to_dates(&value)?;
                result.insert(next_screen_name.to_string(), dates);
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn insert(&self, id: u64, screen_name: &str, dates: Vec<NaiveDate>) -> Result<(), Error> {
        if is_valid_screen_name(screen_name) {
            let mut value = Vec::with_capacity(2 * dates.len());

            for date in dates {
                value.extend_from_slice(&date_to_day_id(&date)?.to_be_bytes());
            }

            self.db.merge(pair_to_key(id, screen_name), value)?;

            Ok(())
        } else {
            Err(Error::InvalidScreenName(screen_name.to_string()))
        }
    }

    pub fn remove(&self, id: u64, screen_name: &str) -> Result<(), Error> {
        let key = pair_to_key(id, screen_name);

        Ok(self.db.delete(key)?)
    }

    pub fn get_date_counts(&self) -> Result<Vec<(NaiveDate, u64)>, Error> {
        let mut map = HashMap::new();
        let iter = self.db.iterator(IteratorMode::Start);

        for (_, value) in iter {
            let dates = value_to_dates(&value)?;

            for date in dates {
                let count = map.entry(date).or_default();
                *count += 1;
            }
        }

        let mut result = map.into_iter().collect::<Vec<_>>();
        result.sort();

        Ok(result)
    }

    pub fn get_most_screen_names(&self, k: usize) -> Result<Vec<(u64, Vec<String>)>, Error> {
        let mut queue = priority_queue::DoublePriorityQueue::with_capacity(k);
        let iter = self.db.iterator(IteratorMode::Start);
        let mut last_id = 0;
        let mut current: Vec<String> = vec![];

        for (key, _) in iter {
            let (id, screen_name) = key_to_pair(&key)?;
            if id != last_id {
                let min = queue.peek_min().map(|(_, count)| *count).unwrap_or(0);
                let len = current.len();

                if len >= min {
                    queue.push((last_id, current.drain(..).collect()), len);

                    if queue.len() > k {
                        queue.pop_min();
                    }
                } else {
                    current.clear();
                }

                last_id = id;
            }
            current.push(screen_name.to_string());
        }

        Ok(queue.into_descending_sorted_vec())
    }

    pub fn compact_ranges(&self) -> Result<(), Error> {
        let iter = self.db.iterator(IteratorMode::Start);

        for (key, value) in iter {
            let mut dates = value_to_dates(&value)?;
            // If we don't have more than a range we don't need to compact
            if dates.len() > 2 {
                dates.sort();
                dates.dedup();

                let compacted_dates = if dates.len() <= 2 {
                    dates
                } else {
                    let mut compacted_dates = Vec::with_capacity(2);

                    if let Some(first) = dates.first() {
                        compacted_dates.push(*first);
                    }

                    if let Some(last) = dates.last() {
                        compacted_dates.push(*last);
                    }

                    compacted_dates
                };

                let mut new_value = Vec::with_capacity(2 * compacted_dates.len());

                for date in compacted_dates {
                    new_value.extend_from_slice(&date_to_day_id(&date)?.to_be_bytes());
                }

                self.db.put(key, new_value)?;
            }
        }

        Ok(())
    }

    pub fn validate_screen_names(&self) -> Result<Vec<(u64, String)>, Error> {
        let iter = self.db.iterator(IteratorMode::Start);
        let mut errors = vec![];

        for (key, _) in iter {
            let (id, screen_name) = key_to_pair(&key)?;

            if !is_valid_screen_name(screen_name) {
                errors.push((id, screen_name.to_string()));
            }
        }

        Ok(errors)
    }

    fn merge(
        _new_key: &[u8],
        existing_val: Option<&[u8]>,
        operands: &MergeOperands,
    ) -> Option<Vec<u8>> {
        let mut new_val = match existing_val {
            Some(bytes) => bytes.to_vec(),
            None => Vec::with_capacity(operands.len() * 2),
        };

        for operand in operands.iter() {
            Self::merge_for_pair(&mut new_val, operand);
        }

        Some(new_val)
    }

    fn merge_for_pair(a: &mut Vec<u8>, b: &[u8]) {
        let original_len = a.len();
        let mut i = 0;

        while i < b.len() {
            let bytes: [u8; 2] = match b[i..i + 2].try_into() {
                Ok(bytes) => bytes,
                Err(error) => {
                    log::error!("{}", error);
                    return;
                }
            };
            let next_b = u16::from_be_bytes(bytes);

            let mut found = false;
            let mut j = 0;

            while !found && j < original_len {
                let bytes = match a[j..j + 2].try_into() {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        log::error!("{}", error);
                        return;
                    }
                };
                let next_a = u16::from_be_bytes(bytes);
                found = next_a == next_b;
                j += 2;
            }

            if !found {
                a.extend_from_slice(&b[i..i + 2]);
            }
            i += 2;
        }
    }
}
pub struct PairIterator<I> {
    underlying: I,
}

impl<I: Iterator<Item = (Box<[u8]>, Box<[u8]>)>> Iterator for PairIterator<I> {
    type Item = Result<(u64, String, Vec<NaiveDate>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.underlying.next()?;

        Some(Self::to_item(&key, &value))
    }
}

impl<I: Iterator<Item = (Box<[u8]>, Box<[u8]>)>> PairIterator<I> {
    fn to_item(key: &[u8], value: &[u8]) -> <Self as Iterator>::Item {
        let (id, screen_name) = key_to_pair(key)?;
        let dates = value_to_dates(value)?;

        Ok((id, screen_name.to_string(), dates))
    }
}

fn id_to_key_prefix(id: u64) -> [u8; 8] {
    id.to_be_bytes()
}

fn key_prefix_to_id(key: &[u8]) -> Result<u64, Error> {
    Ok(u64::from_be_bytes(
        key[0..8]
            .try_into()
            .map_err(|_| Error::InvalidKey(key.to_vec()))?,
    ))
}

fn pair_to_key(id: u64, screen_name: &str) -> Vec<u8> {
    let screen_name_bytes = screen_name.as_bytes();
    let mut prefix = Vec::with_capacity(8 + screen_name_bytes.len());
    prefix.extend_from_slice(&id.to_be_bytes());
    prefix.extend_from_slice(screen_name_bytes);
    prefix
}

fn key_to_pair(key: &[u8]) -> Result<(u64, &str), Error> {
    let id = key_prefix_to_id(key)?;
    let screen_name = std::str::from_utf8(&key[8..])?;

    Ok((id, screen_name))
}

lazy_static::lazy_static! {
    /// Date of the first tweet
    static ref TWITTER_EPOCH: NaiveDate = NaiveDate::from_ymd(2006, 3, 21);
}

fn date_to_day_id(date: &NaiveDate) -> Result<u16, Error> {
    let day = (*date - *TWITTER_EPOCH).num_days();
    day.try_into().map_err(|_| Error::InvalidDay(day))
}

fn day_id_to_date(day_id: u16) -> NaiveDate {
    *TWITTER_EPOCH + Duration::days(day_id.into())
}

fn value_to_dates(value: &[u8]) -> Result<Vec<NaiveDate>, Error> {
    let count = value.len() / 2;
    let mut result = Vec::with_capacity(count);

    for i in 0..count {
        let day_id = u16::from_be_bytes(
            value[i * 2..(i * 2 + 2)]
                .try_into()
                .map_err(|_| Error::InvalidValue(value.to_vec()))?,
        );
        result.push(day_id_to_date(day_id));
    }

    result.sort();
    Ok(result)
}
