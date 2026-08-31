//! See [`SessionHandler`].

use crate::{
    middleware::{
        access_control::{ApiDocSecurityAddon, GovernorAccountIdKeyExtractor, require_session},
        extract_real_ip::GovernorIpKeyExtractor,
        limiter,
    },
    res::{AppJson, AppQuery, AppResult},
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    middleware,
    routing::{delete, get, post, put},
};
use service::{
    AppState,
    dto::{self, request::ApplicationRedirectUrl},
    handlers::SessionHandler,
    id::{AccountId, ApplicationId},
};
use std::{net::IpAddr, time::Duration};
use utoipa::OpenApi;

#[utoipa::path(
    get, path = "me",
    responses((status = OK, body = dto::response::Account)),
    security(("session_jwt" = [])),
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
    responses((status = OK)),
    security(("session_jwt" = [])),
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
    responses((status = OK)),
    security(("session_jwt" = [])),
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
    responses((status = OK)),
    security(("session_jwt" = [])),
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
    security(("session_jwt" = [])),
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
    ),
    security(("session_jwt" = [])),
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
    ),
    security(("session_jwt" = [])),
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

#[utoipa::path(
    get, path = "me/application-consents",
    params(("after_application_id" = Option<ApplicationId>, Query)),
    responses(
        (status = OK, body = Vec<dto::response::ApplicationConsent>)
    ),
    security(("session_jwt" = [])),
)]
async fn get_application_consents(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    AppQuery(pagination_dto): AppQuery<dto::request::ConsentPagination>,
) -> AppResult<Vec<dto::response::ApplicationConsent>> {
    Ok(AppJson(
        SessionHandler(state)
            .get_application_consents(account_id, pagination_dto)
            .await?,
    ))
}

#[utoipa::path(
    get, path = "me/application-consents/{application_id}",
    params(("application_id" = ApplicationId, Path)),
    responses(
        (status = OK, body = dto::response::ApplicationConsentStatus)
    ),
    security(("session_jwt" = [])),
)]
async fn get_application_consent_status(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Path(application_id): Path<ApplicationId>,
) -> AppResult<dto::response::ApplicationConsentStatus> {
    Ok(AppJson(
        SessionHandler(state)
            .get_application_consent_status(account_id, application_id)
            .await?,
    ))
}

#[utoipa::path(
    put, path = "me/application-consents/{application_id}",
    params(("application_id" = ApplicationId, Path)),
    security(("session_jwt" = [])),
)]
async fn authorize_application(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Path(application_id): Path<ApplicationId>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .authorize_application(account_id, application_id)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "me/application-consents/{application_id}",
    params(("application_id" = ApplicationId, Path)),
    security(("session_jwt" = [])),
)]
async fn unauthorize_application(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Path(application_id): Path<ApplicationId>,
) -> AppResult<()> {
    Ok(AppJson(
        SessionHandler(state)
            .unauthorize_application(account_id, application_id)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "me/application-consents/{application_id}/authorization-code",
    request_body = dto::request::ApplicationRedirectUrl,
    params(("application_id" = ApplicationId, Path)),
    responses(
        (status = OK, body = dto::response::AuthorizationCode)
    ),
    security(("session_jwt" = [])),
)]
async fn create_authorization_code(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Path(application_id): Path<ApplicationId>,
    AppJson(redirect_url_dto): AppJson<ApplicationRedirectUrl>,
) -> AppResult<dto::response::AuthorizationCode> {
    Ok(AppJson(
        SessionHandler(state)
            .create_authorization_code(account_id, application_id, redirect_url_dto)
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
        .route("/me/application-consents", get(get_application_consents))
        .route(
            "/me/application-consents/{application_id}",
            get(get_application_consent_status),
        )
        .route(
            "/me/application-consents/{application_id}",
            put(authorize_application),
        )
        .route(
            "/me/application-consents/{application_id}",
            delete(unauthorize_application),
        )
        .route(
            "/me/application-consents/{application_id}/authorization-code",
            post(create_authorization_code),
        )
        .route_layer(limiter::basic::<GovernorAccountIdKeyExtractor>(
            10,
            Duration::from_secs(5),
        ))
        .route_layer(limiter::basic::<GovernorIpKeyExtractor>(
            25,
            Duration::from_secs(5),
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_session,
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_current_account, add_email,
        remove_email, change_primary_email, change_password,
        get_owned_applications, create_application,
        get_application_consents, get_application_consent_status,
        authorize_application, unauthorize_application,
        create_authorization_code
    ),
    components(schemas(
        dto::response::Account,
        dto::response::ApplicationConsent,
        dto::response::ApplicationConsentStatus,
        dto::response::ApplicationSummary,
        dto::response::CreatedApplication,
        dto::request::ApplicationName,
        dto::request::ApplicationRedirectUrl,
        dto::request::ConsentPagination,
        dto::request::ChangePassword,
        dto::request::Email,
    )),
    modifiers(&ApiDocSecurityAddon),
)]
pub struct ApiDoc;
