use crate::res::{AppJson, AppResult};
use axum::{Router, extract::State, routing::post};
use service::{AppState, handlers::AuthorizationHandler};

async fn auth(State(state): State<AppState>, AppJson(token): AppJson<String>) -> AppResult<()> {
    Ok(AppJson(AuthorizationHandler(state).auth(token).await?))
}

/// See [`AuthorizationHandler`].
pub fn authorization_routes(state: AppState) -> Router {
    Router::new().route("/auth", post(auth)).with_state(state)
}
