//! A minimal OAuth 2.0 authorization code flow: redirect construction, CSRF state
//! cookies, and token exchange. This replaces `rocket_oauth2` from the legacy stack.

use crate::{config::ProviderConfig, error::Error};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use rand::{Rng, distr::Alphanumeric};
use serde_json::Value;
use url::Url;

/// The private cookie holding the pending authorization request's state parameter.
const STATE_COOKIE_NAME: &str = "oauth_state";

/// The number of alphanumeric characters in a generated state parameter.
const STATE_LENGTH: usize = 32;

pub const GITHUB_AUTH_URI: &str = "https://github.com/login/oauth/authorize";
pub const GITHUB_TOKEN_URI: &str = "https://github.com/login/oauth/access_token";
pub const GOOGLE_AUTH_URI: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
pub const TWITTER_AUTHENTICATE_URI: &str = "https://api.twitter.com/oauth/authenticate";

/// Begin an authorization request: generate a state parameter and store it in a
/// private cookie for verification when the provider calls back.
pub fn begin(jar: PrivateCookieJar) -> (PrivateCookieJar, String) {
    let state: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(STATE_LENGTH)
        .map(char::from)
        .collect();

    let cookie: Cookie<'static> = Cookie::build((STATE_COOKIE_NAME, state.clone()))
        .same_site(SameSite::Lax)
        .path("/")
        .into();

    (jar.add(cookie), state)
}

/// Verify a callback's state parameter against the stored cookie, removing it on success.
///
/// # Errors
///
/// Returns [`Error::InvalidOAuthState`] on a missing or mismatched state (possible CSRF).
pub fn verify(jar: PrivateCookieJar, state: &str) -> Result<PrivateCookieJar, Error> {
    let expected = jar
        .get(STATE_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    if !state.is_empty() && expected.as_deref() == Some(state) {
        Ok(jar.remove(Cookie::build(STATE_COOKIE_NAME).path("/")))
    } else {
        Err(Error::InvalidOAuthState)
    }
}

/// Build a provider's authorization redirect URL.
///
/// # Errors
///
/// Returns [`Error::UrlParse`] if the endpoint URI is invalid (an internal invariant).
pub fn authorization_url(
    auth_uri: &str,
    provider: &ProviderConfig,
    scope: &str,
    state: &str,
) -> Result<String, Error> {
    let url = Url::parse_with_params(
        auth_uri,
        [
            ("response_type", "code"),
            ("client_id", provider.client_id.as_str()),
            ("redirect_uri", provider.redirect_uri.as_str()),
            ("scope", scope),
            ("state", state),
        ],
    )?;

    Ok(url.into())
}

/// Exchange an authorization code for the provider's token response.
///
/// The full JSON response is returned because Google's includes an `id_token`
/// needed downstream, not just the access token.
///
/// # Errors
///
/// Returns [`Error::OAuthExchange`] on transport failures or non-success statuses.
pub async fn exchange_code(
    http: &reqwest::Client,
    token_uri: &str,
    provider: &ProviderConfig,
    code: &str,
) -> Result<Value, Error> {
    Ok(http
        .post(token_uri)
        // GitHub responds with form-encoded data unless JSON is requested explicitly.
        .header("Accept", "application/json")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", provider.client_id.as_str()),
            ("client_secret", provider.client_secret.as_str()),
            ("redirect_uri", provider.redirect_uri.as_str()),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// Extract the access token from a token exchange response.
///
/// # Errors
///
/// Returns [`Error::MissingAccessToken`] if the field is absent or not a string.
pub fn access_token(response: &Value) -> Result<&str, Error> {
    response
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or(Error::MissingAccessToken)
}
