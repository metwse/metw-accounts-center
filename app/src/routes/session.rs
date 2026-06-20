use crate::res::{AppJson, AppResult};
use axum::{
    Extension, Router,
    extract::State,
    middleware,
    routing::{delete, get, post},
};
use service::{AppState, dto, handlers::SessionHandler, id::AccountId};

async fn me(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
) -> AppResult<dto::response::Account> {
    Ok(AppJson(SessionHandler(state).me(id).await?))
}

async fn add_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    AppJson(email): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(SessionHandler(state).add_email(id, email).await?))
}

async fn delete_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    AppJson(email): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state).delete_email(id, email).await?,
    ))
}

async fn set_primary_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    AppJson(email): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state).set_primary_mail(id, email).await?,
    ))
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
