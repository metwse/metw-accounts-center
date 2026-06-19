use axum::Router;
use service::{AppState, handlers::SessionHandler};

/// See [`SessionHandler`].
pub fn session_routes(state: AppState) -> Router {
    let _ = SessionHandler(state.clone());

    Router::new().with_state(state)
}
