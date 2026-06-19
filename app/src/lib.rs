//! # metw-accounts-center app
//!
//! This crate contains the application binary, bootstrapping all handlers to
//! the axum routers.

use axum::{Json, Router, routing::get};
use serde::Serialize;
use service::AppState;
use std::time::Instant;

use crate::routes::{
    authentication_routes, authorization_routes, pending_activation_session_routes, session_routes,
};

/// API routes.
pub mod routes;

/// Application status
#[derive(Serialize)]
pub struct AppStatus {
    /// Message to clients.
    pub message: String,
    /// JavaScript that should be executed by clients if present.
    pub patch: Option<String>,
    /// Uptime of the API server, in seconds.
    pub uptime: u64,
}

/// Constructs the web API.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get({
                let startup_time = Instant::now();

                async move || {
                    Json(AppStatus {
                        message: "OK".to_string(),
                        patch: None,
                        uptime: Instant::now().duration_since(startup_time).as_secs(),
                    })
                }
            }),
        )
        .with_state(state.clone())
        .merge(authentication_routes(state.clone()))
        .merge(authorization_routes(state.clone()))
        .merge(pending_activation_session_routes(state.clone()))
        .merge(session_routes(state))
}
