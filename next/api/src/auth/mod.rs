use crate::{error::Error, state::AppState};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use memory_lol_auth::model::Provider;

pub mod callback;
pub mod login;
mod oauth;

/// The names of all provider token cookies, for logout.
const TOKEN_COOKIE_NAMES: [&str; 3] = [
    get_token_cookie_name(Provider::GitHub),
    get_token_cookie_name(Provider::Google),
    get_token_cookie_name(Provider::Twitter),
];

/// The cookie name used to store a provider's access token.
const fn get_token_cookie_name(provider: Provider) -> &'static str {
    match provider {
        Provider::GitHub => "github_token",
        Provider::Google => "google_token",
        Provider::Twitter => "twitter_token",
    }
}

/// Read a provider's access token from the (encrypted) cookie jar.
fn get_token_cookie(jar: &PrivateCookieJar, provider: Provider) -> Option<String> {
    jar.get(get_token_cookie_name(provider))
        .map(|cookie| cookie.value().to_string())
}

/// Build a provider token cookie, scoped to the configured domain if there is one.
fn make_token_cookie(provider: Provider, value: &str, domain: &Option<String>) -> Cookie<'static> {
    let cookie = Cookie::build((get_token_cookie_name(provider), value.to_string()))
        .same_site(SameSite::Lax)
        .path("/");

    match domain {
        Some(domain) => cookie.domain(domain.clone()),
        None => cookie,
    }
    .into()
}

/// Check whether any provider cookie in the jar belongs to a trusted user.
///
/// Providers are checked in order (GitHub, Google, Twitter), short-circuiting on
/// the first trusted authorization.
///
/// # Errors
///
/// Returns [`Error::Authorization`] if a token lookup fails.
pub async fn lookup_is_trusted(state: &AppState, jar: &PrivateCookieJar) -> Result<bool, Error> {
    let mut connection = state.auth_db.clone();

    for provider in [Provider::GitHub, Provider::Google, Provider::Twitter] {
        let Some(token) = get_token_cookie(jar, provider) else {
            continue;
        };

        let authorization = match provider {
            Provider::GitHub => {
                state
                    .authorizer
                    .authorize_github(&mut connection, &token)
                    .await?
            }
            Provider::Google => {
                state
                    .authorizer
                    .authorize_google(&mut connection, &token)
                    .await?
            }
            Provider::Twitter => {
                state
                    .authorizer
                    .authorize_twitter(&mut connection, &token)
                    .await?
            }
        };

        if authorization.is_some_and(|authorization| authorization.is_trusted()) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check whether a GitHub token provided directly (not via cookie) is trusted.
///
/// Unknown tokens are validated against the GitHub API and persisted before the
/// authorization check, so a valid token works on its first use.
///
/// # Errors
///
/// Returns [`Error::Authorization`] if token validation or persistence fails.
pub async fn github_token_is_trusted(state: &AppState, token: &str) -> Result<bool, Error> {
    let mut connection = state.auth_db.clone();

    let authorization = match state
        .authorizer
        .authorize_github(&mut connection, token)
        .await?
    {
        Some(authorization) => Some(authorization),
        None => {
            state
                .authorizer
                .save_github_token(&mut connection, token)
                .await?;

            state
                .authorizer
                .authorize_github(&mut connection, token)
                .await?
        }
    };

    Ok(authorization.is_some_and(|authorization| authorization.is_trusted()))
}
