use crate::res::AppResult;
use axum::{Json, Router, extract::State, routing::post};
use service::{AppState, dto, handlers::AuthenticationHandler, id};

async fn signup(
    State(state): State<AppState>,
    Json(signup_dto): Json<dto::request::Signup>,
) -> AppResult<id::AccountId> {
    AuthenticationHandler(state).signup(signup_dto).await.into()
}

async fn login_with_username(
    State(state): State<AppState>,
    Json(login_dto): Json<dto::request::LoginWithUsername>,
) -> AppResult<String> {
    AuthenticationHandler(state)
        .login_with_username(login_dto)
        .await
        .into()
}

async fn login_with_email(
    State(state): State<AppState>,
    Json(login_dto): Json<dto::request::LoginWithEmail>,
) -> AppResult<String> {
    AuthenticationHandler(state)
        .login_with_email(login_dto)
        .await
        .into()
}

async fn logout(State(state): State<AppState>, Json(token): Json<String>) -> AppResult<()> {
    AuthenticationHandler(state).logout(token).await.into()
}

/// See [`AuthenticationHandler`].
pub fn authentication_routes(state: AppState) -> Router {
    // TODO: Add dummy delay to prevent timing attacks.
    // TODO: Connect CAPTCHA.
    Router::new()
        .route("/signup", post(signup))
        .route("/login-with-email", post(login_with_email))
        .route("/login-with-username", post(login_with_username))
        .route("/logout", post(logout))
        .with_state(state.clone())
}
