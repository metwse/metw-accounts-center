use crate::res::{AppJson, AppResult};
use axum::{Extension, Router, extract::State, middleware, routing::post};
use service::{AppState, dto, handlers::PendingActivationSessionHandler, id::AccountId};

#[axum::debug_handler]
async fn retry_signup(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    AppJson(email): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        PendingActivationSessionHandler(state)
            .retry_signup(id, email)
            .await?,
    ))
}

/// See [`PendingActivationSessionHandler`].
pub fn pending_activation_session_routes(state: AppState) -> Router {
    Router::new()
        .route("/retry-signup", post(retry_signup))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_pending_activation_session,
        ))
        .with_state(state)
}
