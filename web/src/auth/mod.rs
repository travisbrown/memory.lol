use super::{error::Error, Auth, SqliteAuthorizer};
use memory_lol_auth::{model::Provider, Access};
use rocket::http::CookieJar;
use rocket_db_pools::Connection;

pub mod callback;
pub mod login;

const TOKEN_COOKIE_NAMES: [&str; 3] = [
    get_token_cookie_name(Provider::GitHub),
    get_token_cookie_name(Provider::Google),
    get_token_cookie_name(Provider::Twitter),
];

const fn get_token_cookie_name(provider: Provider) -> &'static str {
    match provider {
        Provider::GitHub => "github_token",
        Provider::Google => "google_token",
        Provider::Twitter => "twitter_token",
    }
}

pub fn get_token_cookie(cookies: &CookieJar<'_>, provider: Provider) -> Option<String> {
    cookies
        .get_private(get_token_cookie_name(provider))
        .map(|cookie| cookie.value().to_string())
}

pub async fn lookup_access(
    cookies: &CookieJar<'_>,
    authorizer: &SqliteAuthorizer,
    mut connection: Connection<Auth>,
) -> Result<Option<Access>, Error> {
    let github_token = match get_token_cookie(cookies, Provider::GitHub) {
        Some(token) => authorizer
            .authorize_github(&mut connection, &token)
            .await?
            .access(),
        None => None,
    };

    Ok(match github_token {
        Some(access) => Some(access),
        None => {
            let google_token = match get_token_cookie(cookies, Provider::Google) {
                Some(token) => authorizer
                    .authorize_google(&mut connection, &token)
                    .await?
                    .access(),
                None => None,
            };

            match google_token {
                Some(access) => Some(access),
                None => match get_token_cookie(cookies, Provider::Twitter) {
                    Some(token) => authorizer
                        .authorize_twitter(&mut connection, &token)
                        .await?
                        .access(),
                    None => None,
                },
            }
        }
    })
}
