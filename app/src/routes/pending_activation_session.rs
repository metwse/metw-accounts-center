use axum::Router;
use service::{AppState, handlers::PendingActivationSessionHandler};

/// See [`PendingActivationSessionHandler`].
pub fn pending_activation_session_routes(state: AppState) -> Router {
    let _ = PendingActivationSessionHandler(state.clone());

    Router::new().with_state(state)
}
