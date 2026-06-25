//! See [`AuthenticationHandler`].

use crate::res::{AppJson, AppResult};
use axum::{
    Extension, Router,
    extract::{Query, State},
    routing::post,
};
use service::{AppState, dto, handlers::AuthenticationHandler};
use std::net::IpAddr;
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "signup",
    request_body = dto::request::Signup,
    params(dto::request::Captcha),
    responses(
        (status = OK, description = "JWT for email verification session",
            body = dto::response::Token)
    )
)]
async fn signup(
    State(state): State<AppState>,
    Extension(real_ip): Extension<IpAddr>,
    Query(captcha): Query<dto::request::Captcha>,
    AppJson(signup_dto): AppJson<dto::request::Signup>,
) -> AppResult<dto::response::Token> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .signup(signup_dto, real_ip, captcha)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "login/username",
    request_body = dto::request::LoginWithUsername,
    responses(
        (status = OK, description = "JWT for session or email verification session",
            body = dto::response::Token)
    )
)]
async fn login_with_username(
    State(state): State<AppState>,
    AppJson(login_dto): AppJson<dto::request::LoginWithUsername>,
) -> AppResult<dto::response::Token> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .login_with_username(login_dto)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "login/email",
    request_body = dto::request::LoginWithEmail,
    responses(
        (status = OK, description = "JWT for session or email verification session",
            body = dto::response::Token)
    )
)]
async fn login_with_email(
    State(state): State<AppState>,
    AppJson(login_dto): AppJson<dto::request::LoginWithEmail>,
) -> AppResult<dto::response::Token> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .login_with_email(login_dto)
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
    // TODO: Connect CAPTCHA.
    Router::new()
        .route("/signup", post(signup))
        .route("/login/email", post(login_with_email))
        .route("/login/username", post(login_with_username))
        .route("/logout", post(logout))
        .with_state(state.clone())
}

#[derive(OpenApi)]
#[openapi(paths(signup, login_with_email, login_with_username, logout))]
pub struct ApiDoc;
