use crate::{
    auth,
    error::Error,
    logic,
    model::{ExtendedAccount, WithToken},
    state::AppState,
};
use axum::{
    Form, Json,
    extract::{Path, State},
};
use axum_extra::extract::cookie::PrivateCookieJar;
use serde_json::Value;

/// Handle `GET /tw/id/{user_id}`: look up a screen name history by user ID.
///
/// Cookie-based authorization determines whether results are date-limited.
pub async fn by_user_id(
    State(state): State<AppState>,
    Path(user_id): Path<u64>,
    jar: PrivateCookieJar,
) -> Result<Json<ExtendedAccount>, Error> {
    let full_results =
        state.inclusions.contains(user_id) || auth::lookup_is_trusted(&state, &jar).await?;

    let account = state
        .lookup(move |db, _| logic::by_user_id(db, user_id, full_results))
        .await?;

    Ok(Json(account))
}

/// Handle `POST /tw/id/{user_id}`: as [`by_user_id`], but authorized by a GitHub
/// token in the form body instead of cookies.
pub async fn by_user_id_post(
    State(state): State<AppState>,
    Path(user_id): Path<u64>,
    Form(with_token): Form<WithToken>,
) -> Result<Json<ExtendedAccount>, Error> {
    let full_results = state.inclusions.contains(user_id)
        || auth::github_token_is_trusted(&state, &with_token.token).await?;

    let account = state
        .lookup(move |db, _| logic::by_user_id(db, user_id, full_results))
        .await?;

    Ok(Json(account))
}

/// Handle `GET /tw/{screen_name_query}`: look up accounts by screen name, list, or prefix.
pub async fn by_screen_name(
    State(state): State<AppState>,
    Path(screen_name_query): Path<String>,
    jar: PrivateCookieJar,
) -> Result<Json<Value>, Error> {
    let is_trusted = auth::lookup_is_trusted(&state, &jar).await?;

    let result = state
        .lookup(move |db, inclusions| {
            logic::by_screen_name(db, &screen_name_query, inclusions, is_trusted)
        })
        .await?;

    Ok(Json(result))
}

/// Handle `POST /tw/{screen_name_query}`: as [`by_screen_name`], but authorized by
/// a GitHub token in the form body instead of cookies.
pub async fn by_screen_name_post(
    State(state): State<AppState>,
    Path(screen_name_query): Path<String>,
    Form(with_token): Form<WithToken>,
) -> Result<Json<Value>, Error> {
    let is_trusted = auth::github_token_is_trusted(&state, &with_token.token).await?;

    let result = state
        .lookup(move |db, inclusions| {
            logic::by_screen_name(db, &screen_name_query, inclusions, is_trusted)
        })
        .await?;

    Ok(Json(result))
}
