//! See [`SessionHandler`].

use crate::{
    middleware::ApiDocSecurityAddon,
    res::{AppJson, AppResult},
};
use axum::{
    Extension, Router,
    extract::{Query, State},
    middleware,
    routing::{delete, get, post},
};
use service::{AppState, dto, handlers::SessionHandler, id::AccountId};
use std::net::IpAddr;
use utoipa::OpenApi;

#[utoipa::path(
    get, path = "",
    security(("session_jwt" = [])),
    responses(
        (status = OK, body = dto::response::Account)
    )
)]
async fn me(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
) -> AppResult<dto::response::Account> {
    Ok(AppJson(SessionHandler(state).me(id).await?))
}

#[utoipa::path(
    post, path = "/emails",
    security(("session_jwt" = [])),
    request_body = dto::request::Email,
    params(dto::request::Captcha),
    responses(
        (status = OK)
    )
)]
async fn add_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Extension(real_ip): Extension<IpAddr>,
    Query(captcha): Query<dto::request::Captcha>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .add_email(id, email_dto, real_ip, captcha)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "/emails",
    security(("session_jwt" = [])),
    request_body = dto::request::Email,
    responses(
        (status = OK)
    )
)]
async fn delete_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state).delete_email(id, email_dto).await?,
    ))
}

#[utoipa::path(
    post, path = "/emails/set-primary",
    security(("session_jwt" = [])),
    request_body = dto::request::Email,
    params(dto::request::Captcha),
    responses(
        (status = OK)
    )
)]
async fn set_primary_email(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Query(captcha): Query<dto::request::Captcha>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .set_primary_email(id, email_dto, captcha)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
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

#[derive(OpenApi)]
#[openapi(
    paths(me, add_email, delete_email, set_primary_email),
    components(schemas(dto::response::Account, dto::request::Email)),
    modifiers(&ApiDocSecurityAddon),
    security(("session_jwt" = []))
)]
pub struct ApiDoc;
