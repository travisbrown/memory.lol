use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use memory_lol::db::{table::Writeable, Database};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, Read};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("CSV error")]
    Csv(#[from] csv::Error),
    #[error("CSV record encoding error")]
    InvalidCsvRecord(csv::StringRecord),
    #[error("JSON encoding error")]
    InvalidJson(serde_json::Value),
    #[error("Database error")]
    Db(#[from] memory_lol::db::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenNameEntry {
    pub id: u64,
    pub screen_name: String,
    pub snapshots: Vec<DateTime<Utc>>,
}

impl ScreenNameEntry {
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        Self::from_json_opt(value).ok_or_else(|| Error::InvalidJson(value.clone()))
    }

    pub fn from_record(record: &csv::StringRecord) -> Result<Self, Error> {
        Self::from_record_opt(record).ok_or_else(|| Error::InvalidCsvRecord(record.clone()))
    }

    fn from_json_opt(value: &Value) -> Option<Self> {
        let id_str_value = value.get("id_str")?;
        let id_str_string = id_str_value.as_str()?;
        let id = id_str_string.parse::<u64>().ok()?;
        let screen_name_value = value.get("screen_name")?;
        let screen_name = screen_name_value.as_str()?.to_string();
        let snapshot_value = value.get("snapshot")?;
        let snapshot = Utc.timestamp_opt(snapshot_value.as_i64()?, 0).single()?;
        let snapshots = vec![snapshot];

        Some(Self {
            id,
            screen_name,
            snapshots,
        })
    }

    pub fn from_record_opt(record: &csv::StringRecord) -> Option<Self> {
        let id = record.get(0).and_then(|value| value.parse::<u64>().ok())?;
        let screen_name = record.get(1)?.to_string();
        let snapshot_value = record.get(2).and_then(|value| value.parse::<i64>().ok())?;
        let snapshot = Utc.timestamp_opt(snapshot_value, 0).single()?;
        let snapshots = vec![snapshot];

        Some(Self {
            id,
            screen_name,
            snapshots,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateMode {
    All,
    Range,
}

#[derive(Default)]
pub struct Session {
    data: HashMap<(u64, String), Vec<DateTime<Utc>>>,
}

impl Session {
    pub fn load_json<R: BufRead>(source: R) -> Result<Self, Error> {
        let mut session = Session::default();

        for line in source.lines() {
            let line = line?;
            match serde_json::from_str(&line) {
                Ok(value) => {
                    if let Some(entry) = ScreenNameEntry::from_json_opt(&value) {
                        session.add_entry(&entry);
                    }
                }
                Err(error) => {
                    log::warn!("JSON error: {}", error);
                }
            }
        }

        Ok(session)
    }

    pub fn load_mentions<R: Read>(source: R) -> Result<Self, Error> {
        let mut session = Session::default();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(source);

        for record in reader.records() {
            let record = record?;
            let entry = ScreenNameEntry::from_record(&record)?;
            session.add_entry(&entry);
        }

        Ok(session)
    }

    pub fn add_entry(&mut self, entry: &ScreenNameEntry) {
        let snapshots = self
            .data
            .entry((entry.id, entry.screen_name.to_string()))
            .or_default();
        snapshots.extend(&entry.snapshots);
    }

    fn insert(
        db: &Database<Writeable>,
        id: u64,
        screen_name: &str,
        dates: Vec<NaiveDate>,
    ) -> Result<(), Error> {
        match db.insert(id, screen_name, dates) {
            Err(memory_lol::db::Error::InvalidScreenName(screen_name)) => {
                log::warn!("Invalid screen name: {}", screen_name);
                Ok(())
            }
            other => Ok(other?),
        }
    }

    pub fn update(&self, db: &Database<Writeable>, mode: UpdateMode) -> Result<usize, Error> {
        let mut count = 0;

        for ((id, screen_name), snapshots) in &self.data {
            let mut dates = to_dates(snapshots);
            dates.sort();
            dates.dedup();

            match mode {
                UpdateMode::All => {
                    Self::insert(db, *id, screen_name, dates)?;
                }

                UpdateMode::Range => {
                    let range = if dates.len() <= 2 {
                        dates
                    } else {
                        let mut range = Vec::with_capacity(2);

                        if let Some(first) = dates.first() {
                            range.push(*first);
                        }
                        if let Some(last) = dates.last() {
                            range.push(*last);
                        }

                        range
                    };

                    Self::insert(db, *id, screen_name, range)?;
                }
            }

            count += 1;
        }

        Ok(count)
    }
}

fn to_dates(timestamps: &[DateTime<Utc>]) -> Vec<NaiveDate> {
    timestamps
        .iter()
        .map(|timestamp| timestamp.naive_utc().date())
        .collect()
}
