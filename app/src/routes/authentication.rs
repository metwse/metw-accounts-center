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
    params(dto::request::Captcha),
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
    get, path = "login/username/{username}/kdf",
    responses(
        (status = OK, description = "KDF", body = dto::response::AccountKdf)
    )
)]
async fn kdf_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> AppResult<dto::response::AccountKdf> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .get_kdf_by_username(dto::request::Username { username })
            .await?,
    ))
}

#[utoipa::path(
    get, path = "login/email/{email}/kdf",
    responses(
        (status = OK, description = "KDF", body = dto::response::AccountKdf)
    )
)]
async fn kdf_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> AppResult<dto::response::AccountKdf> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .get_kdf_by_email(dto::request::Email { email })
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
        .route("/login/email", post(login_with_email))
        .route("/login/username", post(login_with_username))
        .route("/login/email/{email}/kdf", get(kdf_by_email))
        .route("/login/username/{username}/kdf", get(kdf_by_username))
        .route("/logout", post(logout))
        .layer(basic::<GovernorIpKeyExtractor>(5, Duration::from_secs(5)))
        .with_state(state.clone())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        signup,
        login_with_email,
        login_with_username,
        kdf_by_email,
        kdf_by_username,
        logout
    ),
    components(schemas(dto::request::Email, dto::request::Username))
)]
pub struct ApiDoc;
