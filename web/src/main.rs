#[macro_use]
extern crate rocket;

use crate::{
    authz::{Authorization, Authorizer},
    error::Error,
};
use memory_lol::db::{table::ReadOnly, Database};
use memory_lol::model::Account;
use rocket::{
    fairing::AdHoc,
    form::Form,
    http::{Cookie, CookieJar, SameSite},
    response::Redirect,
    serde::json::Json,
    State,
};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod authz;
mod error;
mod logic;
mod util;

const TOKEN_COOKIE_NAME: &str = "token";

struct GitHub;

#[derive(Deserialize)]
struct AppConfig {
    db: String,
    authorization: String,
}

#[derive(FromForm)]
struct WithToken<'a> {
    token: &'a str,
}

#[get("/tw/id/<user_id>")]
async fn by_user_id(
    user_id: u64,
    cookies: &CookieJar<'_>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<Authorizer>,
) -> Result<Json<Account>, Error> {
    let token_value = cookies
        .get_private(TOKEN_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());
    let account = crate::logic::by_user_id(user_id, token_value.as_deref(), db, authorizer).await?;

    Ok(Json(account))
}

#[post("/tw/id/<user_id>", data = "<with_token>")]
async fn by_user_id_post(
    user_id: u64,
    with_token: Form<WithToken<'_>>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<Authorizer>,
) -> Result<Json<Account>, Error> {
    let account = crate::logic::by_user_id(user_id, Some(with_token.token), db, authorizer).await?;

    Ok(Json(account))
}

#[get("/tw/<screen_name_query>")]
async fn by_screen_name(
    screen_name_query: String,
    cookies: &CookieJar<'_>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<Authorizer>,
) -> Result<Json<Value>, Error> {
    let token_value = cookies
        .get_private(TOKEN_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());
    let result =
        crate::logic::by_screen_name(screen_name_query, token_value.as_deref(), db, authorizer)
            .await?;

    Ok(Json(result))
}

#[post("/tw/<screen_name_query>", data = "<with_token>")]
async fn by_screen_name_post(
    screen_name_query: String,
    with_token: Form<WithToken<'_>>,
    db: &State<Database<ReadOnly>>,
    authorizer: &State<Authorizer>,
) -> Result<Json<Value>, Error> {
    let result =
        crate::logic::by_screen_name(screen_name_query, Some(with_token.token), db, authorizer)
            .await?;

    Ok(Json(result))
}

#[derive(Serialize)]
struct SnowflakeInfo {
    #[serde(rename = "epoch-second")]
    epoch_second: i64,
    #[serde(rename = "utc-rfc2822")]
    utc_rfc2822: String,
}

#[get("/tw/util/snowflake/<id>")]
fn snowflake_info(id: i64) -> Result<Json<Value>, Error> {
    let timestamp = crate::util::snowflake_to_date_time(id).ok_or(Error::InvalidSnowflake(id))?;

    Ok(Json(serde_json::to_value(SnowflakeInfo {
        epoch_second: timestamp.timestamp(),
        utc_rfc2822: timestamp.to_rfc2822(),
    })?))
}

#[get("/auth/github")]
fn github_callback(token: TokenResponse<GitHub>, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        Cookie::build(TOKEN_COOKIE_NAME, token.access_token().to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/login/status")
}

#[derive(Serialize)]
struct LoginStatus {
    provider: Option<String>,
    access: Option<String>,
}

impl LoginStatus {
    fn new(authorization: &Authorization) -> Self {
        Self {
            provider: authorization.provider().map(|value| format!("{}", value)),
            access: authorization.access().map(|value| format!("{}", value)),
        }
    }
}

#[get("/login/github")]
fn login_github(oauth2: OAuth2<GitHub>, cookies: &CookieJar<'_>) -> Result<Redirect, Error> {
    Ok(oauth2.get_redirect(cookies, &[])?)
}

#[get("/login/status")]
async fn login_status(
    cookies: &CookieJar<'_>,
    authorizer: &State<Authorizer>,
) -> Result<Json<Value>, Error> {
    let authorization = match cookies.get_private(TOKEN_COOKIE_NAME) {
        Some(cookie) => authorizer.authorize(cookie.value()).await?,
        None => Authorization::default(),
    };

    Ok(Json(serde_json::json!(LoginStatus::new(&authorization))))
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    if let Some(cookie) = cookies.get_private(TOKEN_COOKIE_NAME) {
        cookies.remove_private(cookie);
    }
    Redirect::to("/login/status")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::config::<AppConfig>())
        .attach(AdHoc::try_on_ignite("Open database", |rocket| async {
            match rocket.state::<AppConfig>().and_then(|config| {
                let db = Database::<ReadOnly>::open(&config.db).ok()?;
                let authorizer = Authorizer::open(&config.authorization).ok()?;

                Some((db, authorizer))
            }) {
                Some((db, authorizer)) => Ok(rocket.manage(db).manage(authorizer)),
                None => Err(rocket),
            }
        }))
        .attach(OAuth2::<GitHub>::fairing("github"))
        .mount(
            "/",
            routes![
                by_user_id,
                by_user_id_post,
                by_screen_name,
                by_screen_name_post,
                snowflake_info,
                github_callback,
                login_github,
                login_status,
                logout,
            ],
        )
}
