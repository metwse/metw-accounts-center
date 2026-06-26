//! See [`EmailVerificationSessionHandler`].

use crate::{
    middleware::{
        auth::{ApiDocAuthAddon, auth_email_verification_session},
        extract_real_ip::GovernorIpKeyExtractor,
        limiter,
    },
    res::{AppJson, AppResult},
};
use axum::{
    Extension, Router,
    extract::{Query, State},
    middleware,
    routing::post,
};
use service::{AppState, dto, handlers::EmailVerificationSessionHandler, id::AccountId};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "signup/retry",
    security(("email_verification_session_jwt" = [])),
    request_body = dto::request::Email,
    params(dto::request::Captcha),
    responses(
        (status = OK)
    )
)]
async fn retry_signup(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Extension(real_ip): Extension<IpAddr>,
    Query(captcha): Query<dto::request::Captcha>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        EmailVerificationSessionHandler(state)
            .retry_signup(id, email_dto, real_ip, captcha)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/signup/retry", post(retry_signup))
        .layer(limiter::basic::<GovernorIpKeyExtractor>(
            2,
            Duration::from_secs(5),
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_email_verification_session,
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(retry_signup),
    components(schemas(dto::request::Email)),
    modifiers(&ApiDocAuthAddon),
    security(("email_verification_session_jwt" = []))
)]
pub struct ApiDoc;
