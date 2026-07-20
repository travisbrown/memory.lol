use crate::{
    error::Error,
    inclusions::Inclusions,
    model::{ExtendedAccount, ExtendedScreenNameResult},
};
use chrono::{Duration, NaiveDate, Utc};
use memory_lol::{
    db::{Database, table::ReadOnly},
    model::Account,
};
use serde_json::{Map, Value};

/// How far back unauthorized users can see, in days.
const UNAUTHORIZED_DAY_LIMIT: i64 = 60;

/// Maximum number of screen names returned for a prefix (wildcard) query.
const LOOKUP_BY_PREFIX_LIMIT: usize = 100;

/// The earliest observation date visible to unauthorized users.
fn get_unauthorized_first_date(limit: i64) -> NaiveDate {
    Utc::now().naive_utc().date() - Duration::days(limit)
}

/// Look up full or date-limited histories for a set of user IDs, dropping empty results.
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
                Ok(result) => (!result.is_empty())
                    .then(|| Ok(Account::from_raw_result(*user_id, result).into())),
                Err(error) => Some(Err(Error::from(error))),
            }
        })
        .collect()
}

/// Look up an account's screen name history by user ID.
///
/// # Arguments
///
/// * `db` - The account history database
/// * `user_id` - The Twitter user ID to look up
/// * `is_trusted` - Whether the caller may see the full (unlimited) history
///
/// # Errors
///
/// Returns [`Error::Db`] if the database read fails.
pub fn by_user_id(
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

/// Look up accounts by screen name query.
///
/// The query may be a single screen name, a comma-separated list of screen names
/// (returning a map keyed by name), or a prefix ending in `*` (returning a map of
/// up to [`LOOKUP_BY_PREFIX_LIMIT`] matches).
///
/// # Arguments
///
/// * `db` - The account history database
/// * `screen_name` - The query string
/// * `inclusions` - User IDs exempt from result limiting
/// * `is_trusted` - Whether the caller may see full (unlimited) histories
///
/// # Errors
///
/// Returns [`Error::Db`] if a database read fails, or [`Error::Json`] on
/// serialization failure.
pub fn by_screen_name(
    db: &Database<ReadOnly>,
    screen_name: &str,
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

        for screen_name in screen_name.split(',').filter(|value| !value.is_empty()) {
            let user_ids = db.lookup_by_screen_name(screen_name)?;
            let accounts = lookup_ids(db, &user_ids, inclusions, earliest)?;
            let result = ExtendedScreenNameResult { accounts };

            if result.includes_screen_name(screen_name) {
                map.insert(screen_name.to_string(), serde_json::to_value(result)?);
            }
        }

        Ok(Value::Object(map))
    } else if let Some(prefix) = screen_name.strip_suffix('*') {
        let mut map = Map::new();
        let results = db.lookup_by_screen_name_prefix(prefix, LOOKUP_BY_PREFIX_LIMIT)?;

        for (screen_name, user_ids) in results {
            let accounts = lookup_ids(db, &user_ids, inclusions, earliest)?;
            let result = ExtendedScreenNameResult { accounts };

            if result.includes_screen_name(&screen_name) {
                map.insert(screen_name, serde_json::to_value(result)?);
            }
        }

        Ok(Value::Object(map))
    } else {
        let user_ids = db.lookup_by_screen_name(screen_name)?;
        let accounts = lookup_ids(db, &user_ids, inclusions, earliest)?;
        let result = ExtendedScreenNameResult { accounts };

        let result = if result.includes_screen_name(screen_name) {
            result
        } else {
            ExtendedScreenNameResult::default()
        };

        Ok(serde_json::to_value(result)?)
    }
}
