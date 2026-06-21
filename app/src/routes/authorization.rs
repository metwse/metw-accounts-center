//! See [`AuthorizationHandler`].

use crate::res::{AppJson, AppResult};
use axum::{Router, extract::State, routing::post};
use service::{AppState, dto, handlers::AuthorizationHandler};
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
    AppJson(token_dto): AppJson<dto::request::Token>,
) -> AppResult<()> {
    Ok(AppJson(AuthorizationHandler(state).auth(token_dto).await?))
}

pub fn routes(state: AppState) -> Router {
    Router::new().route("/auth", post(auth)).with_state(state)
}

#[derive(OpenApi)]
#[openapi(paths(auth), components(schemas(dto::request::Token)))]
pub struct ApiDoc;
