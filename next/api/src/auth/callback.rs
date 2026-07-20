use super::oauth;
use crate::{error::Error, state::AppState};
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::cookie::PrivateCookieJar;
use memory_lol_auth::model::Provider;
use serde::Deserialize;

/// The query parameters of an OAuth 2.0 authorization code callback.
#[derive(Debug, Deserialize)]
pub struct CodeCallback {
    code: String,
    state: String,
}

/// The query parameters of a Twitter OAuth 1.0a callback.
#[derive(Debug, Deserialize)]
pub struct TwitterCallback {
    oauth_token: String,
    oauth_verifier: String,
}

/// The post-login redirect shared by all callbacks.
fn redirect(state: &AppState) -> Redirect {
    Redirect::to(&state.config.default_login_redirect_uri)
}

/// Handle `GET /auth/github`: exchange the code, persist the token, set the cookie.
pub async fn github(
    State(state): State<AppState>,
    Query(query): Query<CodeCallback>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), Error> {
    let jar = oauth::verify(jar, &query.state)?;
    let response = oauth::exchange_code(
        &state.http,
        oauth::GITHUB_TOKEN_URI,
        &state.config.github,
        &query.code,
    )
    .await?;
    let token = oauth::access_token(&response)?;

    let mut connection = state.auth_db.clone();
    let jar = if state
        .authorizer
        .save_github_token(&mut connection, token)
        .await?
    {
        jar.add(super::make_token_cookie(
            Provider::GitHub,
            token,
            &state.config.domain,
        ))
    } else {
        jar
    };

    Ok((jar, redirect(&state)))
}

/// Handle `GET /auth/google`: exchange the code, persist the token, set the cookie.
pub async fn google(
    State(state): State<AppState>,
    Query(query): Query<CodeCallback>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), Error> {
    let jar = oauth::verify(jar, &query.state)?;
    let response = oauth::exchange_code(
        &state.http,
        oauth::GOOGLE_TOKEN_URI,
        &state.config.google,
        &query.code,
    )
    .await?;
    let token = oauth::access_token(&response)?;

    let mut connection = state.auth_db.clone();
    // The full response is passed along because it carries the `id_token` claims.
    let jar = if state
        .authorizer
        .save_google_token(&mut connection, token, &response)
        .await?
    {
        jar.add(super::make_token_cookie(
            Provider::Google,
            token,
            &state.config.domain,
        ))
    } else {
        jar
    };

    Ok((jar, redirect(&state)))
}

/// Handle `GET /auth/twitter`: trade the request token and verifier for an access
/// token, persist it, and set the cookie.
pub async fn twitter(
    State(state): State<AppState>,
    Query(query): Query<TwitterCallback>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), Error> {
    let mut connection = state.auth_db.clone();
    let jar = if let Some(token) = state
        .authorizer
        .save_twitter_token(&mut connection, &query.oauth_token, &query.oauth_verifier)
        .await?
    {
        jar.add(super::make_token_cookie(
            Provider::Twitter,
            &token,
            &state.config.domain,
        ))
    } else {
        jar
    };

    Ok((jar, redirect(&state)))
}
