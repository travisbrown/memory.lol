use super::{
    super::{AppConfig, Auth, SqliteAuthorizer},
    get_token_cookie_name,
};
use crate::error::Error;
use memory_lol_auth::model::{
    providers::{GitHub, Google},
    Provider,
};
use rocket::{
    http::{Cookie, CookieJar, SameSite},
    response::Redirect,
    State,
};
use rocket_db_pools::Connection;
use rocket_oauth2::TokenResponse;

fn make_cookie(name: &'static str, value: String, domain: &Option<String>) -> Cookie<'static> {
    let cookie = Cookie::build((name, value)).same_site(SameSite::Lax);

    let cookie = match domain {
        Some(domain) => cookie.domain(domain.to_string()),
        None => cookie,
    };

    cookie.into()
}

#[get("/auth/github")]
pub async fn github(
    token: TokenResponse<GitHub>,
    cookies: &CookieJar<'_>,
    app_config: &State<AppConfig>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Redirect, Error> {
    if authorizer
        .save_github_token(&mut connection, token.access_token())
        .await?
    {
        cookies.add_private(make_cookie(
            get_token_cookie_name(Provider::GitHub),
            token.access_token().to_string(),
            &app_config.domain,
        ));
    }

    let redirect = Redirect::to(app_config.default_login_redirect_uri.clone());

    Ok(redirect)
}

#[get("/auth/google")]
pub async fn google(
    token: TokenResponse<Google>,
    cookies: &CookieJar<'_>,
    app_config: &State<AppConfig>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Redirect, Error> {
    if authorizer
        .save_google_token(&mut connection, token.access_token(), token.as_value())
        .await?
    {
        cookies.add_private(make_cookie(
            get_token_cookie_name(Provider::Google),
            token.access_token().to_string(),
            &app_config.domain,
        ));
    }

    let redirect = Redirect::to(app_config.default_login_redirect_uri.clone());

    Ok(redirect)
}

#[derive(FromForm, Debug)]
pub struct TwitterTokenResponse<'r> {
    oauth_token: &'r str,
    oauth_verifier: &'r str,
}

#[get("/auth/twitter?<token_response..>")]
pub async fn twitter(
    token_response: TwitterTokenResponse<'_>,
    cookies: &CookieJar<'_>,
    app_config: &State<AppConfig>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Redirect, Error> {
    if let Some(token) = authorizer
        .save_twitter_token(
            &mut connection,
            token_response.oauth_token,
            token_response.oauth_verifier,
        )
        .await?
    {
        cookies.add_private(make_cookie(
            get_token_cookie_name(Provider::Twitter),
            token,
            &app_config.domain,
        ));
    }

    let redirect = Redirect::to(app_config.default_login_redirect_uri.clone());

    Ok(redirect)
}
