use super::{
    super::{Auth, SqliteAuthorizer},
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

#[get("/auth/github")]
pub async fn github(
    token: TokenResponse<GitHub>,
    cookies: &CookieJar<'_>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Redirect, Error> {
    if authorizer
        .save_github_token(&mut connection, token.access_token())
        .await?
    {
        cookies.add_private(
            Cookie::build(
                get_token_cookie_name(Provider::GitHub),
                token.access_token().to_string(),
            )
            .same_site(SameSite::Lax)
            .finish(),
        );
    }

    Ok(Redirect::to("/login/status"))
}

#[get("/auth/google")]
pub async fn google(
    token: TokenResponse<Google>,
    cookies: &CookieJar<'_>,
    authorizer: &State<SqliteAuthorizer>,
    mut connection: Connection<Auth>,
) -> Result<Redirect, Error> {
    if authorizer
        .save_google_token(&mut connection, token.access_token(), token.as_value())
        .await?
    {
        cookies.add_private(
            Cookie::build(
                get_token_cookie_name(Provider::Google),
                token.access_token().to_string(),
            )
            .same_site(SameSite::Lax)
            .finish(),
        );
    }

    Ok(Redirect::to("/login/status"))
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
        cookies.add_private(
            Cookie::build(get_token_cookie_name(Provider::Twitter), token)
                .same_site(SameSite::Lax)
                .finish(),
        );
    }

    Ok(Redirect::to("/login/status"))
}
