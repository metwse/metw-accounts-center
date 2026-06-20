use crate::res::{AppJson, AppResult};
use axum::{Router, extract::State, routing::post};
use service::{AppState, dto, handlers::AuthenticationHandler, id::AccountId};

#[axum::debug_handler]
async fn signup(
    State(state): State<AppState>,
    AppJson(signup_dto): AppJson<dto::request::Signup>,
) -> AppResult<AccountId> {
    Ok(AppJson(
        AuthenticationHandler(state).signup(signup_dto).await?,
    ))
}

async fn login_with_username(
    State(state): State<AppState>,
    AppJson(login_dto): AppJson<dto::request::LoginWithUsername>,
) -> AppResult<String> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .login_with_username(login_dto)
            .await?,
    ))
}

async fn login_with_email(
    State(state): State<AppState>,
    AppJson(login_dto): AppJson<dto::request::LoginWithEmail>,
) -> AppResult<String> {
    Ok(AppJson(
        AuthenticationHandler(state)
            .login_with_email(login_dto)
            .await?,
    ))
}

async fn logout(State(state): State<AppState>, AppJson(token): AppJson<String>) -> AppResult<()> {
    Ok(AppJson(AuthenticationHandler(state).logout(token).await?))
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
