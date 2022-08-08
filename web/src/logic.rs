use super::{error::Error, inclusions::Inclusions, ExtendedAccount, ExtendedScreenNameResult};
use chrono::{Duration, NaiveDate, Utc};
use memory_lol::{
    db::{table::ReadOnly, Database},
    model::Account,
};
use serde_json::{Map, Value};

const UNAUTHORIZED_DAY_LIMIT: i64 = 60;
const LOOKUP_BY_PREFIX_LIMIT: usize = 100;

fn get_unauthorized_first_date(limit: i64) -> NaiveDate {
    Utc::now().naive_utc().date() - Duration::days(limit)
}

fn lookup_ids(
    db: &Database<ReadOnly>,
    user_ids: &[u64],
    inclusions: &Inclusions,
    earliest: Option<NaiveDate>,
) -> Result<Vec<ExtendedAccount>, Error> {
    user_ids
        .iter()
        .filter_map(|user_id| {
            let result = if inclusions.contains(*user_id) {
                db.lookup_by_user_id(*user_id)
            } else {
                db.limited_lookup_by_user_id(*user_id, earliest)
            };

            match result {
                Ok(result) => {
                    if result.is_empty() {
                        None
                    } else {
                        Some(Ok(Account::from_raw_result(*user_id, result).into()))
                    }
                }
                Err(error) => Some(Err(Error::from(error))),
            }
        })
        .collect::<Result<Vec<_>, Error>>()
}

pub(crate) fn by_user_id(
    db: &Database<ReadOnly>,
    user_id: u64,
    is_trusted: bool,
) -> Result<ExtendedAccount, Error> {
    let result = if is_trusted {
        db.lookup_by_user_id(user_id)?
    } else {
        db.limited_lookup_by_user_id(
            user_id,
            Some(get_unauthorized_first_date(UNAUTHORIZED_DAY_LIMIT)),
        )?
    };

    Ok(Account::from_raw_result(user_id, result).into())
}

pub(crate) fn by_screen_name(
    db: &Database<ReadOnly>,
    screen_name: String,
    inclusions: &Inclusions,
    is_trusted: bool,
) -> Result<Value, Error> {
    let earliest = if is_trusted {
        None
    } else {
        Some(get_unauthorized_first_date(UNAUTHORIZED_DAY_LIMIT))
    };

    if screen_name.contains(',') {
        let mut map = Map::new();

        for screen_name in screen_name.split(',') {
            if !screen_name.is_empty() {
                let user_ids = db.lookup_by_screen_name(screen_name)?;
                let accounts = lookup_ids(db, &user_ids, inclusions, earliest)?;
                let result = ExtendedScreenNameResult { accounts };

                if result.includes_screen_name(screen_name) {
                    map.insert(screen_name.to_string(), serde_json::to_value(result)?);
                }
            }
        }

        Ok(serde_json::to_value(map)?)
    } else if screen_name.ends_with('*') {
        let mut map = Map::new();
        let results = db.lookup_by_screen_name_prefix(
            &screen_name[0..screen_name.len() - 1],
            LOOKUP_BY_PREFIX_LIMIT,
        )?;

        for (screen_name, user_ids) in results {
            let accounts = lookup_ids(db, &user_ids, inclusions, earliest)?;
            let result = ExtendedScreenNameResult { accounts };

            if result.includes_screen_name(&screen_name) {
                map.insert(screen_name.to_string(), serde_json::to_value(result)?);
            }
        }

        Ok(serde_json::to_value(map)?)
    } else {
        let user_ids = db.lookup_by_screen_name(&screen_name)?;
        let accounts = lookup_ids(db, &user_ids, inclusions, earliest)?;
        let result = ExtendedScreenNameResult { accounts };

        let result = if result.includes_screen_name(&screen_name) {
            result
        } else {
            ExtendedScreenNameResult::default()
        };

        Ok(serde_json::to_value(result)?)
    }
}
