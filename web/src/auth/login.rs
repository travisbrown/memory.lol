use super::super::{AppConfig, Auth, SqliteAuthorizer};
use crate::error::Error;
use memory_lol_auth::{
    model::{
        providers::{GitHub, Google, Twitter},
        Provider, UserInfo,
    },
    Authorization,
};
use rocket::{http::CookieJar, response::Redirect, serde::json::Json, State};
use rocket_db_pools::Connection;
use rocket_oauth2::OAuth2;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Serialize)]
struct LoginStatus {
    github: Option<ProviderStatus>,
    google: Option<ProviderStatus>,
    twitter: Option<ProviderStatus>,
}

#[derive(Serialize)]
struct ProviderStatus {
    id: String,
    name: String,
    access: Vec<&'static str>,
}

impl ProviderStatus {
    fn new(authorization: &Authorization, user_info: &UserInfo) -> Self {
        let mut access = Vec::with_capacity(1);

        if authorization.is_admin() {
            access.push("admin");
        }

        if authorization.is_trusted() {
            access.push("trusted");
        }

        if authorization.can_write_gists() {
            access.push("gist");
        }

        Self {
            id: user_info.id_str(),
            name: user_info.name(),
            access,
        }
    }
}

#[get("/login/status")]
pub async fn status(
    cookies: &CookieJar<'_>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Json<Value>, Error> {
    let mut status = LoginStatus::default();
    if let Some(token) = super::get_token_cookie(cookies, Provider::GitHub) {
        if let Some(authorization) = authorizer.authorize_github(&mut connection, &token).await? {
            if let Some(user_info) = authorizer
                .get_user_info(&mut connection, &authorization.identity)
                .await?
            {
                status.github = Some(ProviderStatus::new(&authorization, &user_info));
            }
        }
    }

    if let Some(token) = super::get_token_cookie(cookies, Provider::Google) {
        if let Some(authorization) = authorizer.authorize_google(&mut connection, &token).await? {
            if let Some(user_info) = authorizer
                .get_user_info(&mut connection, &authorization.identity)
                .await?
            {
                status.google = Some(ProviderStatus::new(&authorization, &user_info));
            }
        }
    }

    if let Some(token) = super::get_token_cookie(cookies, Provider::Twitter) {
        if let Some(authorization) = authorizer
            .authorize_twitter(&mut connection, &token)
            .await?
        {
            if let Some(user_info) = authorizer
                .get_user_info(&mut connection, &authorization.identity)
                .await?
            {
                status.twitter = Some(ProviderStatus::new(&authorization, &user_info));
            }
        }
    }

    Ok(Json(serde_json::json!(status)))
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>, app_config: &State<AppConfig>) -> Redirect {
    for token_cookie_name in super::TOKEN_COOKIE_NAMES {
        if let Some(cookie) = cookies.get_private(token_cookie_name) {
            cookies.remove_private(cookie);
        }
    }

    Redirect::to(app_config.default_login_redirect_uri.clone())
}

#[get("/login/github?<scope>")]
pub fn github(
    oauth2: OAuth2<GitHub>,
    cookies: &CookieJar<'_>,
    scope: Option<&str>,
) -> Result<Redirect, Error> {
    if scope == Some("gist") {
        Ok(oauth2.get_redirect(cookies, &["gist"])?)
    } else {
        Ok(oauth2.get_redirect(cookies, &[])?)
    }
}

#[get("/login/google")]
pub fn google(oauth2: OAuth2<Google>, cookies: &CookieJar<'_>) -> Result<Redirect, Error> {
    Ok(oauth2.get_redirect(cookies, &["https://www.googleapis.com/auth/userinfo.email"])?)
}

#[get("/login/twitter")]
pub async fn twitter(
    oauth2: OAuth2<Twitter>,
    cookies: &CookieJar<'_>,
    authorizer: &State<SqliteAuthorizer>,
) -> Result<Redirect, Error> {
    let request_token_key = authorizer.create_twitter_request_token().await?;

    Ok(oauth2.get_redirect_extras(cookies, &[], &[("oauth_token", &request_token_key)])?)
}
