use crate::res::AppResult;
use axum::{
    Extension, Json, Router,
    extract::State,
    middleware,
    routing::{delete, get, post},
};
use service::{AppState, dto, handlers::SessionHandler, id::AccountId};

async fn me(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
) -> AppResult<dto::response::Account> {
    SessionHandler(state).me(id).await.into()
}

async fn add_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Json(email): Json<dto::request::Email>,
) -> AppResult<()> {
    SessionHandler(state).add_email(id, email).await.into()
}

async fn delete_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Json(email): Json<dto::request::Email>,
) -> AppResult<()> {
    SessionHandler(state).delete_email(id, email).await.into()
}

async fn set_primary_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Json(email): Json<dto::request::Email>,
) -> AppResult<()> {
    SessionHandler(state)
        .set_primary_mail(id, email)
        .await
        .into()
}

/// See [`SessionHandler`].
pub fn session_routes(state: AppState) -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/me/emails", post(add_email))
        .route("/me/emails", delete(delete_email))
        .route("/me/emails/set-primary", post(set_primary_email))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_session,
        ))
        .with_state(state)
}
