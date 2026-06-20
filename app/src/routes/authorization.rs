use crate::res::AppResult;
use axum::{Json, Router, extract::State, routing::post};
use service::{AppState, handlers::AuthorizationHandler};

async fn auth(State(state): State<AppState>, Json(token): Json<String>) -> AppResult<()> {
    AuthorizationHandler(state).auth(token).await.into()
}

/// See [`AuthorizationHandler`].
pub fn authorization_routes(state: AppState) -> Router {
    Router::new().route("/auth", post(auth)).with_state(state)
}
