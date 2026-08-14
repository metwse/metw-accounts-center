//! See [`EmailVerificationSessionHandler`].

use crate::{
    middleware::{
        auth::{ApiDocAuthAddon, GovernorAccountIdKeyExtractor, auth_email_verification_session},
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
    request_body = dto::request::Email,
    params(("captcha" = dto::request::Captcha, Query)),
    responses(
        (status = OK)
    )
)]
async fn retry_signup(
    State(state): State<AppState>,
    Extension(id): Extension<AccountId>,
    Extension(real_ip): Extension<IpAddr>,
    AppQuery(captcha): AppQuery<dto::request::Captcha>,
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
        .layer(limiter::basic::<GovernorAccountIdKeyExtractor>(
            2,
            Duration::from_secs(5),
        ))
        .layer(limiter::basic::<GovernorIpKeyExtractor>(
            5,
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
