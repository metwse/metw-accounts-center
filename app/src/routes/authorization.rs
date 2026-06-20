//! See [`AuthorizationHandler`].

use crate::res::{AppJson, AppResult};
use axum::{
    Router,
    extract::{Query, State},
    routing::post,
};
use serde::Deserialize;
use service::{AppState, handlers::AuthorizationHandler};
use utoipa::{IntoParams, OpenApi, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
struct Token {
    pub token: String,
}

#[utoipa::path(
    post, path = "auth",
    params(Token),
    responses(
        (status = OK)
    )
)]
async fn auth(State(state): State<AppState>, Query(token): Query<Token>) -> AppResult<()> {
    Ok(AppJson(
        AuthorizationHandler(state).auth(token.token).await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new().route("/auth", post(auth)).with_state(state)
}

#[derive(OpenApi)]
#[openapi(paths(auth), components(schemas(Token)))]
pub struct ApiDoc;
