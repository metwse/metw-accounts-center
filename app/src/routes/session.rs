//! See [`SessionHandler`].

use crate::{
    middleware::{
        auth::{ApiDocAuthAddon, GovernorAccountIdKeyExtractor, auth_session},
        extract_real_ip::GovernorIpKeyExtractor,
        limiter,
    },
    res::{AppJson, AppQuery, AppResult},
};
use axum::{
    Extension, Router,
    extract::State,
    middleware,
    routing::{delete, get, post},
};
use service::{AppState, dto, handlers::SessionHandler, id::AccountId};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    get, path = "me",
    responses(
        (status = OK, body = dto::response::Account)
    )
)]
async fn get_current_account(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
) -> AppResult<dto::response::Account> {
    Ok(AppJson(
        SessionHandler(state)
            .get_current_account(account_id)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "me/emails",
    request_body = dto::request::Email,
    params(("captcha" = dto::request::Captcha, Query)),
    responses(
        (status = OK)
    )
)]
async fn add_email(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Extension(real_ip): Extension<IpAddr>,
    AppQuery(captcha): AppQuery<dto::request::Captcha>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .add_email(account_id, email_dto, real_ip, captcha)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "me/emails",
    request_body = dto::request::Email,
    responses(
        (status = OK)
    )
)]
async fn remove_email(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .remove_email(account_id, email_dto)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "me/emails/set-primary",
    request_body = dto::request::Email,
    params(("captcha" = dto::request::Captcha, Query)),
    responses(
        (status = OK)
    )
)]
async fn change_primary_email(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    AppQuery(captcha): AppQuery<dto::request::Captcha>,
    AppJson(email_dto): AppJson<dto::request::Email>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .change_primary_email(account_id, email_dto, captcha)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "me/change-password",
    request_body = dto::request::ChangePassword,
)]
async fn change_password(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    AppJson(change_password_dto): AppJson<dto::request::ChangePassword>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .change_password(account_id, change_password_dto)
            .await?,
    ))
}

#[utoipa::path(
    get, path = "me/applications",
    responses(
        (status = OK, body = Vec<dto::response::ApplicationSummary>)
    )
)]
async fn get_owned_applications(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
) -> AppResult<Vec<dto::response::ApplicationSummary>> {
    Ok(AppJson(
        SessionHandler(state)
            .get_owned_applications(account_id)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "me/applications",
    responses(
        (status = OK, body = dto::response::CreatedApplication)
    )
)]
async fn create_application(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    AppQuery(captcha): AppQuery<dto::request::Captcha>,
    AppJson(create_app_dto): AppJson<dto::request::ApplicationName>,
) -> AppResult<dto::response::CreatedApplication> {
    Ok(AppJson(
        SessionHandler(state)
            .create_application(account_id, create_app_dto, captcha)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/me", get(get_current_account))
        .route("/me/emails", post(add_email))
        .route("/me/emails", delete(remove_email))
        .route("/me/emails/set-primary", post(change_primary_email))
        .route("/me/change-password", post(change_password))
        .route("/me/applications", get(get_owned_applications))
        .route("/me/applications", post(create_application))
        .layer(limiter::basic::<GovernorAccountIdKeyExtractor>(
            10,
            Duration::from_secs(5),
        ))
        .layer(limiter::basic::<GovernorIpKeyExtractor>(
            25,
            Duration::from_secs(5),
        ))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_session))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_current_account, add_email,
        remove_email, change_primary_email, change_password,
        get_owned_applications, create_application
    ),
    components(schemas(
        dto::response::Account,
        dto::response::ApplicationSummary,
        dto::response::CreatedApplication,
        dto::request::Email,
        dto::request::ChangePassword,
        dto::request::ApplicationName,
    )),
    modifiers(&ApiDocAuthAddon),
    security(("session_jwt" = []))
)]
pub struct ApiDoc;
