#[macro_use]
extern crate rocket;

use memory_lol::db::{table::ReadOnly, Database};
use memory_lol::model::Account;
use memory_lol_auth::{
    model::{
        providers::{GitHub, Google, Twitter},
        IsProvider,
    },
    Authorizer,
};
use memory_lol_auth_sqlx::SqlxAuthDb;
use rocket::{
    fairing::{AdHoc, Fairing},
    form::Form,
    http::CookieJar,
    serde::json::Json,
    Build, Rocket, State,
};
use rocket_db_pools::{sqlx, Connection, Database as PoolDatabase};
use rocket_oauth2::{OAuth2, OAuthConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod auth;
mod error;
mod logic;
mod snowflake;
mod util;

use error::Error;

fn provider_fairing<P: IsProvider>() -> impl Fairing {
    OAuth2::<P>::fairing(P::provider().name())
}

#[derive(Deserialize)]
pub struct AppConfig {
    db: String,
    authorization: String,
    domain: Option<String>,
    default_login_redirect_uri: rocket::http::uri::Reference<'static>,
}

#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub struct ExtendedScreenNameResult {
    accounts: Vec<ExtendedAccount>,
}

impl ExtendedScreenNameResult {
    pub fn includes_screen_name(&self, screen_name: &str) -> bool {
        let target_screen_name = screen_name.to_lowercase();
        self.accounts.iter().any(|account| {
            account
                .screen_names
                .keys()
                .any(|screen_name| screen_name.to_lowercase() == target_screen_name)
        })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ExtendedAccount {
    pub id: u64,
    pub id_str: String,
    pub screen_names: indexmap::IndexMap<String, Option<Vec<chrono::NaiveDate>>>,
}

impl From<Account> for ExtendedAccount {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            id_str: account.id.to_string(),
            screen_names: account.screen_names,
        }
    }
}

#[derive(FromForm)]
struct WithToken<'a> {
    token: &'a str,
}

type SqliteAuthorizer = Authorizer<SqlxAuthDb>;

#[derive(PoolDatabase)]
#[database("sqlite_auth")]
pub struct Auth(sqlx::SqlitePool);

#[get("/tw/id/<user_id>")]
async fn by_user_id(
    user_id: u64,
    cookies: &CookieJar<'_>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<SqliteAuthorizer>,
    connection: Connection<Auth>,
) -> Result<Json<ExtendedAccount>, Error> {
    let is_trusted = auth::lookup_is_trusted(cookies, authorizer, connection).await?;
    let account = crate::logic::by_user_id(db, user_id, is_trusted)?;

    Ok(Json(account))
}

#[post("/tw/id/<user_id>", data = "<with_token>")]
async fn by_user_id_post(
    user_id: u64,
    with_token: Form<WithToken<'_>>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Json<ExtendedAccount>, Error> {
    let authorization = authorizer
        .authorize_github(&mut connection, with_token.token)
        .await?;

    let access = match authorization {
        None => {
            authorizer
                .save_github_token(&mut connection, with_token.token)
                .await?;

            authorizer
                .authorize_github(&mut connection, with_token.token)
                .await?
                .map(|authorization| authorization.is_trusted())
                .unwrap_or(false)
        }
        Some(authorization) => authorization.is_trusted(),
    };

    let account = crate::logic::by_user_id(db, user_id, access)?;

    Ok(Json(account))
}

#[get("/tw/<screen_name_query>")]
async fn by_screen_name(
    screen_name_query: String,
    cookies: &CookieJar<'_>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<SqliteAuthorizer>,
    connection: Connection<Auth>,
) -> Result<Json<Value>, Error> {
    let is_trusted = auth::lookup_is_trusted(cookies, authorizer, connection).await?;
    let result = crate::logic::by_screen_name(db, screen_name_query, is_trusted)?;

    Ok(Json(result))
}

#[post("/tw/<screen_name_query>", data = "<with_token>")]
async fn by_screen_name_post(
    screen_name_query: String,
    with_token: Form<WithToken<'_>>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Json<Value>, Error> {
    let authorization = authorizer
        .authorize_github(&mut connection, with_token.token)
        .await?;

    let access = match authorization {
        None => {
            authorizer
                .save_github_token(&mut connection, with_token.token)
                .await?;

            authorizer
                .authorize_github(&mut connection, with_token.token)
                .await?
                .map(|authorization| authorization.is_trusted())
                .unwrap_or(false)
        }
        Some(authorization) => authorization.is_trusted(),
    };
    let result = crate::logic::by_screen_name(db, screen_name_query, access)?;

    Ok(Json(result))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::config::<AppConfig>())
        .attach(AdHoc::try_on_ignite("Open database", |rocket| async {
            match init_db(&rocket) {
                Some(db) => Ok(rocket.manage(db)),
                None => Err(rocket),
            }
        }))
        .attach(AdHoc::try_on_ignite(
            "Open authorization databases",
            |rocket| async {
                match init_authorization(&rocket).await {
                    Some(authorizer) => Ok(rocket.manage(authorizer)),
                    None => Err(rocket),
                }
            },
        ))
        .attach(Auth::init())
        .attach(provider_fairing::<GitHub>())
        .attach(provider_fairing::<Google>())
        .attach(provider_fairing::<Twitter>())
        .mount(
            "/",
            routes![
                by_user_id,
                by_user_id_post,
                by_screen_name,
                by_screen_name_post,
                snowflake::info,
                auth::login::status,
                auth::login::logout,
                auth::login::github,
                auth::login::google,
                auth::login::twitter,
                auth::callback::github,
                auth::callback::google,
                auth::callback::twitter,
            ],
        )
}

fn init_db(rocket: &Rocket<Build>) -> Option<Database<ReadOnly>> {
    let config = rocket.state::<AppConfig>()?;
    Database::<ReadOnly>::open(&config.db).ok()
}

async fn init_authorization(rocket: &Rocket<Build>) -> Option<SqliteAuthorizer> {
    let google_config = OAuthConfig::from_figment(rocket.figment(), "google").ok()?;
    let twitter_config = OAuthConfig::from_figment(rocket.figment(), "twitter").ok()?;
    let config = rocket.state::<AppConfig>()?;

    Authorizer::open(
        &config.authorization,
        "memory.lol",
        google_config.client_id(),
        google_config.client_secret(),
        twitter_config.client_id(),
        twitter_config.client_secret(),
        twitter_config.redirect_uri()?,
    )
    .await
    .ok()
}
