use axum::Router;
use service::{AppState, handlers::AuthorizationHandler};

/// See [`AuthorizationHandler`].
pub fn authorization_routes(state: AppState) -> Router {
    let _ = AuthorizationHandler(state.clone());

    Router::new().with_state(state)
}
