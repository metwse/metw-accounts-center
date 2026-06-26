//! # metw-accounts-center app
//!
//! This crate contains the application binary, bootstrapping all handlers to
//! the axum routers.
//!
//! *Note*: The Axum application is intended to run under a reverse proxy. The
//! reverse proxy must provide the remote address via the `X-Real-IP` header.

use axum::{Json, Router, routing::get};
use serde::Serialize;
use service::AppState;
use std::{sync::LazyLock, time::Instant};
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};

/// API routes.
pub mod routes;

/// API middleware.
pub mod middleware;

/// API results.
pub mod res;

/// OpenAPI documentation.
#[derive(OpenApi)]
#[openapi(
    info(description = "metw-accounts-center API"),
    servers((url = "http://localhost:3781", description = "Local server")),
    external_docs(url = "https://metwse.github.io/metw-accounts-center/", description = "Crate documentation"),
    paths(get_status),
    nest(
        (path = "/", api = routes::authentication::ApiDoc, tags = ["authentication"]),
        (path = "/", api = routes::authorization::ApiDoc, tags = ["authorization"]),
        (path = "/me", api = routes::session::ApiDoc, tags = ["session"]),
        (path = "/", api = routes::email_verification_session::ApiDoc, tags = ["email_verification_session"])
    )
)]
pub struct ApiDoc;

/// Application status
#[derive(Serialize, ToSchema)]
struct AppStatus {
    /// Status message.
    pub message: String,
    /// JavaScript that should be executed by clients if present.
    pub patch: Option<String>,
    /// Uptime of the API server, in seconds.
    pub uptime: u64,
}

static STARTUP_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[utoipa::path(
    get, path = "/",
    responses(
        (status = OK, body = AppStatus)
    )
)]
async fn get_status() -> Json<AppStatus> {
    Json(AppStatus {
        message: "OK".to_string(),
        patch: None,
        uptime: Instant::now().duration_since(*STARTUP_TIME).as_secs(),
    })
}

/// Constructs the web API.
pub fn app(state: AppState) -> Router {
    let _ = *STARTUP_TIME;

    Router::new()
        .route("/", get(get_status))
        .route("/openapi.json", get(async || Json(ApiDoc::openapi())))
        .with_state(state.clone())
        .merge(routes::authentication::routes(state.clone()))
        .merge(routes::authorization::routes(state.clone()))
        .merge(routes::email_verification_session::routes(state.clone()))
        .merge(routes::session::routes(state))
        .route_layer(axum::middleware::from_fn(
            middleware::extract_real_ip::extract_real_ip,
        ))
        .layer(TraceLayer::new_for_http())
}
