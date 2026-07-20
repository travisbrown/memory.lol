//! The memory.lol JSON API service.
//!
//! An Axum port of the legacy Rocket service in `web/`, exposing the same routes:
//! account history lookups (`/tw/...`), a Snowflake timestamp utility, and OAuth
//! login flows for GitHub, Google, and Twitter.

use anyhow::Context;
use axum::{Router, routing::get};
use axum_extra::extract::cookie::Key;
use base64::Engine;
use memory_lol::db::{Database, table::ReadOnly};
use memory_lol_auth::Authorizer;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

mod auth;
mod config;
mod error;
mod inclusions;
mod logic;
mod model;
mod routes;
mod snowflake;
mod state;

use config::Config;
use inclusions::Inclusions;
use state::AppState;

/// The default configuration file path, relative to the working directory.
const DEFAULT_CONFIG_PATH: &str = "Api.toml";

/// How long a request may run before being answered with a timeout status.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let config = Config::load(&config_path)
        .with_context(|| format!("loading configuration from {config_path}"))?;

    let state = init_state(config).await?;
    let address = (state.config.address, state.config.port);

    let app = Router::new()
        .route(
            "/tw/id/{user_id}",
            get(routes::by_user_id).post(routes::by_user_id_post),
        )
        .route(
            "/tw/{screen_name_query}",
            get(routes::by_screen_name).post(routes::by_screen_name_post),
        )
        .route("/tw/util/snowflake/{id}", get(snowflake::info))
        .route("/login/status", get(auth::login::status))
        .route("/login/github", get(auth::login::github))
        .route("/login/google", get(auth::login::google))
        .route("/login/twitter", get(auth::login::twitter))
        .route("/logout", get(auth::login::logout))
        .route("/auth/github", get(auth::callback::github))
        .route("/auth/google", get(auth::callback::google))
        .route("/auth/twitter", get(auth::callback::twitter))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(CompressionLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("binding {}:{}", address.0, address.1))?;
    tracing::info!(address = %listener.local_addr()?, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Open all databases and clients and assemble the shared application state.
async fn init_state(config: Config) -> anyhow::Result<AppState> {
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&config.secret_key)
        .context("decoding secret_key as base64")?;
    let key = Key::try_from(key_bytes.as_slice())
        .map_err(|_| anyhow::anyhow!("secret_key must decode to at least 64 bytes"))?;

    let db = Database::<ReadOnly>::open(&config.db)
        .with_context(|| format!("opening account database at {}", config.db))?;

    let inclusions = match &config.inclusions {
        Some(path) => Inclusions::read_file(path)
            .with_context(|| format!("reading inclusions from {path}"))?,
        None => Inclusions::default(),
    };

    let auth_db = memory_lol_auth_rusqlite::open(&config.auth_db)
        .await
        .with_context(|| format!("opening authorization database at {}", config.auth_db))?;

    let authorizer = Authorizer::open(
        &config.authorization,
        "memory.lol",
        &config.google.client_id,
        &config.google.client_secret,
        &config.twitter.client_id,
        &config.twitter.client_secret,
        &config.twitter.redirect_uri,
    )
    .await
    .context("initializing authorizer")?;

    Ok(AppState {
        db: Arc::new(db),
        inclusions: Arc::new(inclusions),
        authorizer: Arc::new(authorizer),
        auth_db,
        http: reqwest::Client::new(),
        config: Arc::new(config),
        key,
    })
}

/// Resolve when the process receives Ctrl-C or (on Unix) SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to install Ctrl-C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => tracing::error!(%error, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
