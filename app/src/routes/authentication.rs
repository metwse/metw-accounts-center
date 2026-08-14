//! See [`AuthenticationHandler`].

use crate::{
    middleware::{extract_real_ip::GovernorIpKeyExtractor, limiter::basic},
    res::{AppJson, AppQuery, AppResult},
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    routing::{get, post},
};
use service::{AppState, dto, handlers::AuthenticationHandler};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "signup",
    request_body = dto::request::Signup,
    params(("captcha" = dto::request::Captcha, Query)),
    responses(
        (status = OK, description = "JWT for email verification session",
            body = dto::response::Token)
    )
)]
async fn signup(
    State(state): State<AppState>,
    Extension(real_ip): Extension<IpAddr>,
    AppQuery(captcha): AppQuery<dto::request::Captcha>,
    AppJson(signup_dto): AppJson<dto::request::Signup>,
) -> AppResult<dto::response::Token> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .signup(signup_dto, real_ip, captcha)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "login",
    request_body = dto::request::Login,
    responses(
        (status = OK, description = "JWT for session or email verification session",
            body = dto::response::Token)
    )
)]
async fn login(
    State(state): State<AppState>,
    AppJson(login_dto): AppJson<dto::request::Login>,
) -> AppResult<dto::response::Token> {
    Ok(AppJson(
        AuthenticationHandler(state).login(login_dto).await?,
    ))
}

#[utoipa::path(
    get, path = "login/{account_identifier}/kdf",
    responses(
        (status = OK, description = "KDF", body = dto::response::AccountKdf)
    ),
    params(("account_identifier" = dto::request::AccountIdentifier, Path))
)]
async fn get_kdf(
    State(state): State<AppState>,
    Path(account_identifier): Path<dto::request::AccountIdentifier>,
) -> AppResult<dto::response::AccountKdf> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .get_kdf(account_identifier)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "logout",
    request_body = dto::request::Token,
    responses(
        (status = OK)
    )
)]
async fn logout(
    State(state): State<AppState>,
    AppJson(token_dto): AppJson<dto::request::Token>,
) -> AppResult<()> {
    Ok(AppJson(
        AuthenticationHandler(state).logout(token_dto).await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    // TODO: Add dummy delay to prevent timing attacks.
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/login/{account_identifier}/kdf", get(get_kdf))
        .route("/logout", post(logout))
        .layer(basic::<GovernorIpKeyExtractor>(5, Duration::from_secs(5)))
        .with_state(state.clone())
}

#[derive(OpenApi)]
#[openapi(
    paths(signup, login, get_kdf, logout),
    components(schemas(
        dto::request::Login,
        dto::request::Token,
        dto::request::Captcha,
        dto::request::AccountIdentifier,
        dto::request::Signup
    ))
)]
pub struct ApiDoc;
