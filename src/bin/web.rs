#[macro_use]
extern crate rocket;

use chrono::NaiveDate;
use memory_lol::{db::Database, error::Error};
use rocket::{fairing::AdHoc, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

#[derive(Deserialize)]
struct AppConfig {
    db: String,
}

#[derive(Serialize, Deserialize)]
struct ScreenNameResult {
    accounts: Vec<Account>,
}

#[derive(Serialize, Deserialize)]
struct Account {
    id: u64,
    #[serde(rename = "screen-names")]
    screen_names: Value,
}

fn format_screen_names(result: HashMap<String, Vec<NaiveDate>>) -> Value {
    let mut sorted = result
        .into_iter()
        .map(|(screen_name, mut dates)| {
            dates.sort();

            let value = match dates.len() {
                0 => None,
                1 => Some(vec![dates[0]]),
                n => Some(vec![dates[0], dates[n - 1]]),
            };

            (screen_name, value)
        })
        .collect::<Vec<_>>();

    sorted.sort_by(|(screen_name_a, dates_a), (screen_name_b, dates_b)| {
        dates_a
            .as_ref()
            .and_then(|dates| dates.get(0))
            .cmp(&dates_b.as_ref().and_then(|dates| dates.get(0)))
            .then_with(|| screen_name_a.cmp(screen_name_b))
    });

    let mut screen_names = Map::new();

    for (screen_name, dates) in sorted {
        screen_names.insert(
            screen_name.to_string(),
            json!(dates.map(|dates| dates
                .iter()
                .map(|date| format!("{}", date))
                .collect::<Vec<_>>())),
        );
    }

    json!(screen_names)
}

#[get("/tw/id/<user_id>")]
fn by_user_id(user_id: u64, state: &State<Database>) -> Result<Json<Account>, Error> {
    let result = state.lookup_by_user_id(user_id)?;

    Ok(Json(Account {
        id: user_id,
        screen_names: format_screen_names(result),
    }))
}

#[get("/tw/<screen_name>")]
fn by_screen_name(screen_name: String, state: &State<Database>) -> Result<Json<Value>, Error> {
    if screen_name.contains(',') {
        let mut map = Map::new();

        for screen_name in screen_name.split(',') {
            if !screen_name.is_empty() {
                let user_ids = state.lookup_by_screen_name(screen_name)?;

                let accounts = user_ids
                    .iter()
                    .map(|user_id| {
                        let result = state.lookup_by_user_id(*user_id)?;

                        Ok(Account {
                            id: *user_id,
                            screen_names: format_screen_names(result),
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;

                map.insert(
                    screen_name.to_string(),
                    serde_json::to_value(ScreenNameResult { accounts })?,
                );
            }
        }

        Ok(Json(serde_json::to_value(map)?))
    } else {
        let user_ids = state.lookup_by_screen_name(&screen_name)?;

        let accounts = user_ids
            .iter()
            .map(|user_id| {
                let result = state.lookup_by_user_id(*user_id)?;
                Ok(Account {
                    id: *user_id,
                    screen_names: format_screen_names(result),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(Json(serde_json::to_value(ScreenNameResult { accounts })?))
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::config::<AppConfig>())
        .attach(AdHoc::try_on_ignite("Open database", |rocket| async {
            match rocket
                .state::<AppConfig>()
                .and_then(|config| Database::open(&config.db).ok())
            {
                Some(db) => Ok(rocket.manage(db)),
                None => Err(rocket),
            }
        }))
        .mount("/", routes![by_user_id, by_screen_name])
}
