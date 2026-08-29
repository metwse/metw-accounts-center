//! See [`TokenActionHandler`].

use crate::{
    middleware::{extract_real_ip::GovernorIpKeyExtractor, limiter},
    res::{AppJson, AppResult},
};
use axum::{Extension, Router, extract::State, routing::post};
use service::{AppState, dto, handlers::TokenActionHandler};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "token-actions",
    request_body = dto::request::Token,
    responses(
        (status = OK)
    )
)]
async fn execute_token_action(
    State(state): State<AppState>,
    Extension(real_ip): Extension<IpAddr>,
    AppJson(token_dto): AppJson<dto::request::Token>,
) -> AppResult<()> {
    Ok(AppJson(
        TokenActionHandler(state)
            .execute_token_action(token_dto, real_ip)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/token-actions", post(execute_token_action))
        .route_layer(limiter::basic::<GovernorIpKeyExtractor>(
            2,
            Duration::from_secs(5),
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(paths(execute_token_action), components(schemas(dto::request::Token)))]
pub struct ApiDoc;
