#[macro_use]
extern crate rocket;

use crate::error::Error;
use memory_lol::db::{table::ReadOnly, Database};
use memory_lol::model::{Account, ScreenNameResult};
use rocket::{fairing::AdHoc, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

mod error;
mod util;

const LOOKUP_BY_PREFIX_LIMIT: usize = 100;

#[derive(Deserialize)]
struct AppConfig {
    db: String,
}

#[get("/tw/id/<user_id>")]
fn by_user_id(user_id: u64, state: &State<Database<ReadOnly>>) -> Result<Json<Account>, Error> {
    let result = state.lookup_by_user_id(user_id)?;

    Ok(Json(Account::from_raw_result(user_id, result)))
}

#[get("/tw/<screen_name>")]
fn by_screen_name(
    screen_name: String,
    state: &State<Database<ReadOnly>>,
) -> Result<Json<Value>, Error> {
    if screen_name.contains(',') {
        let mut map = Map::new();

        for screen_name in screen_name.split(',') {
            if !screen_name.is_empty() {
                let user_ids = state.lookup_by_screen_name(screen_name)?;

                let accounts = user_ids
                    .iter()
                    .map(|user_id| {
                        let result = state.lookup_by_user_id(*user_id)?;

                        Ok(Account::from_raw_result(*user_id, result))
                    })
                    .collect::<Result<Vec<_>, Error>>()?;

                map.insert(
                    screen_name.to_string(),
                    serde_json::to_value(ScreenNameResult { accounts })?,
                );
            }
        }

        Ok(Json(serde_json::to_value(map)?))
    } else if screen_name.ends_with('*') {
        let mut map = Map::new();
        let results = state.lookup_by_screen_name_prefix(
            &screen_name[0..screen_name.len() - 1],
            LOOKUP_BY_PREFIX_LIMIT,
        )?;

        for (screen_name, user_ids) in results {
            let accounts = user_ids
                .iter()
                .map(|user_id| {
                    let result = state.lookup_by_user_id(*user_id)?;

                    Ok(Account::from_raw_result(*user_id, result))
                })
                .collect::<Result<Vec<_>, Error>>()?;

            map.insert(
                screen_name.to_string(),
                serde_json::to_value(ScreenNameResult { accounts })?,
            );
        }

        Ok(Json(serde_json::to_value(map)?))
    } else {
        let user_ids = state.lookup_by_screen_name(&screen_name)?;

        let accounts = user_ids
            .iter()
            .map(|user_id| {
                let result = state.lookup_by_user_id(*user_id)?;

                Ok(Account::from_raw_result(*user_id, result))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(Json(serde_json::to_value(ScreenNameResult { accounts })?))
    }
}

#[derive(Serialize)]
struct SnowflakeInfo {
    #[serde(rename = "epoch-second")]
    epoch_second: i64,
    #[serde(rename = "ufc-rfc2822")]
    utc_rfc2822: String,
}

#[get("/tw/util/snowflake/<id>")]
fn snowflake_info(id: i64) -> Result<Json<Value>, Error> {
    let timestamp =
        crate::util::snowflake_to_date_time(id).ok_or_else(|| Error::InvalidSnowflake(id))?;

    Ok(Json(serde_json::to_value(SnowflakeInfo {
        epoch_second: timestamp.timestamp(),
        utc_rfc2822: timestamp.to_rfc2822(),
    })?))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::config::<AppConfig>())
        .attach(AdHoc::try_on_ignite("Open database", |rocket| async {
            match rocket
                .state::<AppConfig>()
                .and_then(|config| Database::<ReadOnly>::open(&config.db).ok())
            {
                Some(db) => Ok(rocket.manage(db)),
                None => Err(rocket),
            }
        }))
        .mount("/", routes![by_user_id, by_screen_name, snowflake_info])
}
