use axum::Router;
use service::{AppState, handlers::AuthenticationHandler};

/// See [`AuthenticationHandler`].
pub fn authentication_routes(state: AppState) -> Router {
    let _ = AuthenticationHandler(state.clone());

    Router::new().with_state(state.clone())
}
