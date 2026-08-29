//! See [`EmailVerificationSessionHandler`].

use crate::{
    middleware::{
        access_control::{
            ApiDocSecurityAddon, GovernorAccountIdKeyExtractor, require_email_verification_session,
        },
        extract_real_ip::GovernorIpKeyExtractor,
        limiter,
    },
    res::{AppJson, AppQuery, AppResult},
};
use axum::{Extension, Router, extract::State, middleware, routing::post};
use service::{AppState, dto, handlers::EmailVerificationSessionHandler, id::AccountId};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "signup/retry",
    security(("email_verification_session_jwt" = [])),
    request_body = dto::request::RetrySignup,
    params(("captcha" = dto::request::Captcha, Query)),
    responses(
        (status = OK)
    )
)]
async fn retry_signup(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Extension(real_ip): Extension<IpAddr>,
    AppQuery(captcha): AppQuery<dto::request::Captcha>,
    AppJson(retry_signup_dto): AppJson<dto::request::RetrySignup>,
) -> AppResult<()> {
    Ok(AppJson(
        EmailVerificationSessionHandler(state)
            .retry_signup(account_id, retry_signup_dto, real_ip, captcha)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/signup/retry", post(retry_signup))
        .route_layer(limiter::basic::<GovernorAccountIdKeyExtractor>(
            2,
            Duration::from_secs(5),
        ))
        .route_layer(limiter::basic::<GovernorIpKeyExtractor>(
            5,
            Duration::from_secs(5),
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_email_verification_session,
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(retry_signup),
    components(schemas(dto::request::RetrySignup)),
    modifiers(&ApiDocSecurityAddon),
    security(("email_verification_session_jwt" = []))
)]
pub struct ApiDoc;
