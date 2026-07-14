//! See [`AuthorizationHandler`].

use crate::{
    middleware::{auth::GovernorAccountIdKeyExtractor, limiter},
    res::{AppJson, AppResult},
};
use axum::{Extension, Router, extract::State, routing::post};
use service::{AppState, dto, handlers::AuthorizationHandler};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "auth",
    request_body = dto::request::Token,
    responses(
        (status = OK)
    )
)]
async fn auth(
    State(state): State<AppState>,
    Extension(real_ip): Extension<IpAddr>,
    AppJson(token_dto): AppJson<dto::request::Token>,
) -> AppResult<()> {
    Ok(AppJson(
        AuthorizationHandler(state).auth(token_dto, real_ip).await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/auth", post(auth))
        .layer(limiter::basic::<GovernorAccountIdKeyExtractor>(
            2,
            Duration::from_secs(5),
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(paths(auth), components(schemas(dto::request::Token)))]
pub struct ApiDoc;
