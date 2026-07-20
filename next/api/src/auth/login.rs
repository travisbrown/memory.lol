use super::oauth;
use crate::{error::Error, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use memory_lol_auth::{Authorization, model::Provider, model::UserInfo};
use serde::{Deserialize, Serialize};
use url::Url;

/// The signed-in status for each supported provider.
#[derive(Debug, Default, Serialize)]
pub struct LoginStatus {
    github: Option<ProviderStatus>,
    google: Option<ProviderStatus>,
    twitter: Option<ProviderStatus>,
}

/// A signed-in user's identity and access levels for one provider.
#[derive(Debug, Serialize)]
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

/// Resolve one provider's status from its token cookie, if present and valid.
async fn provider_status(
    state: &AppState,
    jar: &PrivateCookieJar,
    provider: Provider,
) -> Result<Option<ProviderStatus>, Error> {
    let Some(token) = super::get_token_cookie(jar, provider) else {
        return Ok(None);
    };

    let mut connection = state.auth_db.clone();

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

    let Some(authorization) = authorization else {
        return Ok(None);
    };

    Ok(state
        .authorizer
        .get_user_info(&mut connection, &authorization.identity)
        .await?
        .map(|user_info| ProviderStatus::new(&authorization, &user_info)))
}

/// Handle `GET /login/status` by reporting the signed-in status for each provider.
pub async fn status(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<Json<LoginStatus>, Error> {
    Ok(Json(LoginStatus {
        github: provider_status(&state, &jar, Provider::GitHub).await?,
        google: provider_status(&state, &jar, Provider::Google).await?,
        twitter: provider_status(&state, &jar, Provider::Twitter).await?,
    }))
}

/// Handle `GET /logout` by removing all provider token cookies.
pub async fn logout(
    State(state): State<AppState>,
    mut jar: PrivateCookieJar,
) -> (PrivateCookieJar, Redirect) {
    for name in super::TOKEN_COOKIE_NAMES {
        let cookie = Cookie::build(name).path("/");

        let cookie = match &state.config.domain {
            Some(domain) => cookie.domain(domain.clone()),
            None => cookie,
        };

        jar = jar.remove(cookie);
    }

    (jar, Redirect::to(&state.config.default_login_redirect_uri))
}

/// The query parameters accepted by the GitHub login endpoint.
#[derive(Debug, Deserialize)]
pub struct GitHubLoginQuery {
    /// Pass `scope=gist` to additionally request gist write access.
    scope: Option<String>,
}

/// Handle `GET /login/github` by redirecting to GitHub's authorization page.
pub async fn github(
    State(state): State<AppState>,
    Query(query): Query<GitHubLoginQuery>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), Error> {
    let (jar, oauth_state) = oauth::begin(jar);
    let scope = if query.scope.as_deref() == Some("gist") {
        "gist"
    } else {
        ""
    };
    let url = oauth::authorization_url(
        oauth::GITHUB_AUTH_URI,
        &state.config.github,
        scope,
        &oauth_state,
    )?;

    Ok((jar, Redirect::to(&url)))
}

/// Handle `GET /login/google` by redirecting to Google's authorization page.
pub async fn google(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), Error> {
    let (jar, oauth_state) = oauth::begin(jar);
    // The `openid` scope makes Google include the `id_token` used to extract the
    // user's subject identifier and email address.
    let url = oauth::authorization_url(
        oauth::GOOGLE_AUTH_URI,
        &state.config.google,
        "openid email",
        &oauth_state,
    )?;

    Ok((jar, Redirect::to(&url)))
}

/// Handle `GET /login/twitter` by creating an OAuth 1.0a request token and
/// redirecting to Twitter's authentication page.
pub async fn twitter(State(state): State<AppState>) -> Result<Redirect, Error> {
    let request_token = state.authorizer.create_twitter_request_token().await?;
    let url = Url::parse_with_params(
        oauth::TWITTER_AUTHENTICATE_URI,
        [("oauth_token", request_token.as_str())],
    )?;

    Ok(Redirect::to(url.as_str()))
}
