//! See [`EmailVerificationSessionHandler`].

use crate::{
    middleware::ApiDocSecurityAddon,
    res::{AppJson, AppResult},
};
use axum::{Extension, Router, extract::State, middleware, routing::post};
use service::{AppState, dto, handlers::EmailVerificationSessionHandler, id::AccountId};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "signup/retry",
    security(("email_verification_session_jwt" = [])),
    request_body = dto::request::Email,
    responses(
        (status = OK)
    )
)]
async fn retry_signup(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        EmailVerificationSessionHandler(state)
            .retry_signup(id, email_dto)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/signup/retry", post(retry_signup))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_email_verification_session,
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(retry_signup),
    components(schemas(dto::request::Email)),
    modifiers(&ApiDocSecurityAddon),
    security(("email_verification_session_jwt" = []))
)]
pub struct ApiDoc;
