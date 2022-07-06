use super::error::Error;
use chrono::{Duration, NaiveDate};
use rocksdb::{DBIterator, IteratorMode, MergeOperands, Options, DB};
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;

pub struct Lookup {
    db: DB,
}

impl Lookup {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Lookup, Error> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.set_merge_operator_associative("merge", Self::merge);
        let db = DB::open(&options, path)?;

        Ok(Lookup { db })
    }

    pub fn get_estimated_key_count(&self) -> Result<Option<u64>, Error> {
        Ok(self.db.property_int_value("rocksdb.estimate-num-keys")?)
    }

    fn date_to_day_id(date: &NaiveDate) -> Result<u16, Error> {
        let twitter_epoch = NaiveDate::from_ymd(2006, 3, 21);
        let day = (*date - twitter_epoch).num_days();
        day.try_into().map_err(|_| Error::InvalidDay(day))
    }

    fn day_id_to_date(day_id: u16) -> NaiveDate {
        let twitter_epoch = NaiveDate::from_ymd(2006, 3, 21);
        twitter_epoch + Duration::days(day_id.into())
    }

    pub fn pairs(&self) -> PairIterator<DBIterator> {
        PairIterator {
            underlying: self.db.iterator(IteratorMode::Start),
        }
    }

    pub fn get_counts(&self) -> Result<(u64, u64, u64), Error> {
        let mut pair_count = 0;
        let mut user_id_count = 0;
        let mut screen_name_count = 0;

        let iter = self.db.iterator(IteratorMode::Start);
        let mut last_user_id = 0;

        for (key, _) in iter {
            if key[0] == 0 {
                pair_count += 1;
                let user_id = u64::from_be_bytes(
                    key[1..9]
                        .try_into()
                        .map_err(|_| Error::InvalidKey(key.to_vec()))?,
                );
                if user_id != last_user_id {
                    user_id_count += 1;
                    last_user_id = user_id;
                }
            } else if key[0] == 1 {
                screen_name_count += 1;
            } else if key[0] != 2 {
                // We allow 2 as a prefix because it was previously used to track imported files
                return Err(Error::InvalidKey(key.to_vec()));
            }
        }

        Ok((pair_count, user_id_count, screen_name_count))
    }

    pub fn get_date_counts(&self) -> Result<Vec<(NaiveDate, u64)>, Error> {
        let mut map = HashMap::new();
        let iter = self.db.iterator(IteratorMode::Start);

        for (key, value) in iter {
            if key[0] == 0 {
                let dates = Self::value_to_dates(&value)?;

                for date in dates {
                    let count = map.entry(date).or_default();
                    *count += 1;
                }
            } else {
                break;
            }
        }

        let mut result = map.into_iter().collect::<Vec<_>>();
        result.sort();

        Ok(result)
    }

    pub fn get_most_screen_names(&self, k: usize) -> Result<Vec<(u64, Vec<String>)>, Error> {
        let mut queue = priority_queue::DoublePriorityQueue::with_capacity(k);
        let iter = self.db.iterator(IteratorMode::Start);
        let mut last_user_id = 0;
        let mut current: Vec<String> = vec![];

        for (key, _) in iter {
            if let Some((user_id, screen_name)) = Self::key_to_pair(&key)? {
                if user_id != last_user_id {
                    let min = queue.peek_min().map(|(_, count)| *count).unwrap_or(0);
                    let len = current.len();

                    if len >= min {
                        queue.push((last_user_id, current.drain(..).collect()), len);

                        if queue.len() > k {
                            queue.pop_min();
                        }
                    } else {
                        current.clear();
                    }

                    last_user_id = user_id;
                }
                current.push(screen_name.to_string());
            } else {
                break;
            }
        }

        Ok(queue.into_descending_sorted_vec())
    }

    pub fn compact_ranges(&self) -> Result<(), Error> {
        let iter = self.db.iterator(IteratorMode::Start);

        for (key, value) in iter {
            if key[0] == 0 {
                let mut dates = Self::value_to_dates(&value)?;
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

                    let mut new_value = Vec::with_capacity(4 * compacted_dates.len());

                    for date in compacted_dates {
                        new_value.extend_from_slice(&Self::date_to_day_id(&date)?.to_be_bytes());
                    }

                    self.db.put(key, new_value)?;
                }
            }
        }

        Ok(())
    }

    fn user_id_to_prefix(user_id: u64) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(9);
        prefix.push(0);
        prefix.extend_from_slice(&user_id.to_be_bytes());
        prefix
    }

    fn pair_to_key(user_id: u64, screen_name: &str) -> Vec<u8> {
        let screen_name_bytes = screen_name.as_bytes();
        let mut prefix = Vec::with_capacity(screen_name_bytes.len() + 9);
        prefix.push(0);
        prefix.extend_from_slice(&user_id.to_be_bytes());
        prefix.extend_from_slice(screen_name_bytes);
        prefix
    }

    fn key_to_pair(key: &[u8]) -> Result<Option<(u64, &str)>, Error> {
        let pair = if key[0] == 0 {
            let user_id = u64::from_be_bytes(
                key[1..9]
                    .try_into()
                    .map_err(|_| Error::InvalidKey(key.to_vec()))?,
            );
            let screen_name = std::str::from_utf8(&key[9..])?;

            Some((user_id, screen_name))
        } else {
            None
        };

        Ok(pair)
    }

    fn value_to_dates(value: &[u8]) -> Result<Vec<NaiveDate>, Error> {
        let mut result = Vec::with_capacity(1);

        for i in 0..(value.len() / 2) {
            let day_id = u16::from_be_bytes(
                value[i * 2..(i * 2 + 2)]
                    .try_into()
                    .map_err(|_| Error::InvalidValue(value.to_vec()))?,
            );
            result.push(Self::day_id_to_date(day_id));
        }

        result.sort();
        Ok(result)
    }

    pub fn lookup_by_user_id(
        &self,
        user_id: u64,
    ) -> Result<HashMap<String, Vec<NaiveDate>>, Error> {
        let prefix = Self::user_id_to_prefix(user_id);
        let iter = self.db.prefix_iterator(prefix);
        let mut result = HashMap::new();

        for (key, value) in iter {
            if let Some((next_user_id, next_screen_name)) = Self::key_to_pair(&key)? {
                if next_user_id == user_id {
                    let dates = Self::value_to_dates(&value)?;
                    result.insert(next_screen_name.to_string(), dates);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn lookup_by_screen_name(&self, screen_name: &str) -> Result<Vec<u64>, Error> {
        let value = self.db.get_pinned(Self::screen_name_to_key(screen_name))?;

        if let Some(value) = value {
            let mut result = Vec::with_capacity(1);
            let mut i = 0;

            while i < value.len() {
                let next = u64::from_be_bytes(
                    value[i..i + 8]
                        .try_into()
                        .map_err(|_| Error::InvalidValue(value.to_vec()))?,
                );

                result.push(next);
                i += 8;
            }

            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    fn screen_name_to_key(screen_name: &str) -> Vec<u8> {
        let form = screen_name.to_lowercase();
        let as_bytes = form.as_bytes();
        let mut key = Vec::with_capacity(as_bytes.len() + 1);
        key.push(1);
        key.extend_from_slice(as_bytes);
        key
    }

    fn key_to_screen_name(key: &[u8]) -> Result<Option<&str>, Error> {
        let screen_name = if key[0] == 1 {
            Some(std::str::from_utf8(&key[1..])?)
        } else {
            None
        };

        Ok(screen_name)
    }

    pub fn insert_pair(
        &self,
        id: u64,
        screen_name: &str,
        dates: Vec<NaiveDate>,
    ) -> Result<(), Error> {
        let mut value = Vec::with_capacity(4 * dates.len());

        for date in dates {
            value.extend_from_slice(&Self::date_to_day_id(&date)?.to_be_bytes());
        }

        self.db.merge(Self::pair_to_key(id, screen_name), value)?;
        self.db
            .merge(Self::screen_name_to_key(screen_name), id.to_be_bytes())?;

        Ok(())
    }

    pub fn remove_pair(&self, id: u64, screen_name: &str) -> Result<(), Error> {
        let key = Self::pair_to_key(id, screen_name);

        Ok(self.db.delete(key)?)
    }

    fn is_valid_screen_name(value: &str) -> bool {
        value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    }

    pub fn validate_screen_names(&self) -> Result<Vec<(Option<u64>, String)>, Error> {
        let iter = self.db.iterator(IteratorMode::Start);
        let mut errors = vec![];

        for (key, _) in iter {
            if key[0] == 0 {
                if let Some((id, screen_name)) = Self::key_to_pair(&key)? {
                    if !Self::is_valid_screen_name(screen_name) {
                        errors.push((Some(id), screen_name.to_string()));
                    }
                }
            } else if key[0] == 1 {
                if let Some(screen_name) = Self::key_to_screen_name(&key)? {
                    if !Self::is_valid_screen_name(screen_name) {
                        errors.push((None, screen_name.to_string()));
                    }
                }
            }
        }

        Ok(errors)
    }

    fn merge(
        new_key: &[u8],
        existing_val: Option<&[u8]>,
        operands: &MergeOperands,
    ) -> Option<Vec<u8>> {
        let mut new_val = match existing_val {
            Some(bytes) => bytes.to_vec(),
            None => Vec::with_capacity(operands.len() * 10 * 8),
        };

        if new_key[0] == 0 {
            for operand in operands.iter() {
                Self::merge_for_pair(&mut new_val, operand);
            }
        } else {
            for operand in operands.iter() {
                Self::merge_for_screen_name(&mut new_val, operand);
            }
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
pub struct PairIterator<I> {
    underlying: I,
}

impl<I: Iterator<Item = (Box<[u8]>, Box<[u8]>)>> Iterator for PairIterator<I> {
    type Item = Result<(u64, String, Vec<NaiveDate>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.underlying.next()?;

        Lookup::key_to_pair(&key).map_or_else(
            |error| Some(Err(error)),
            |pair| {
                pair.map(|(id, screen_name)| {
                    Lookup::value_to_dates(&value).map(|dates| (id, screen_name.to_string(), dates))
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn insert() {
        let dir = tempfile::tempdir().unwrap();
        let db = Lookup::new(dir).unwrap();
        db.insert_pair(123, "foo", vec![]).unwrap();
        db.insert_pair(123, "bar", vec![]).unwrap();
        db.insert_pair(456, "foo", vec![]).unwrap();
        db.insert_pair(123, "foo", vec![]).unwrap();

        let mut expected_by_id = HashMap::new();
        expected_by_id.insert("foo".to_string(), vec![]);
        expected_by_id.insert("bar".to_string(), vec![]);

        let expected_pairs = vec![
            (123, "bar".to_string(), vec![]),
            (123, "foo".to_string(), vec![]),
            (456, "foo".to_string(), vec![]),
        ];

        assert_eq!(db.lookup_by_screen_name("foo").unwrap(), vec![123, 456]);
        assert_eq!(db.lookup_by_user_id(123).unwrap(), expected_by_id);
        assert_eq!(db.get_counts().unwrap(), (3, 2, 2));
        assert_eq!(
            db.pairs().collect::<Result<Vec<_>, _>>().unwrap(),
            expected_pairs
        );

        db.compact_ranges().unwrap();

        assert_eq!(db.lookup_by_screen_name("foo").unwrap(), vec![123, 456]);
        assert_eq!(db.lookup_by_user_id(123).unwrap(), expected_by_id);
        assert_eq!(db.get_counts().unwrap(), (3, 2, 2));
        assert_eq!(
            db.pairs().collect::<Result<Vec<_>, _>>().unwrap(),
            expected_pairs
        );
    }
}
