//! See [`ApplicationManagementHandler`].

use crate::{
    middleware::{
        auth::{
            ApiDocAuthAddon, GovernorAccountIdKeyExtractor, auth_application_ownership,
            auth_session,
        },
        extract_real_ip::GovernorIpKeyExtractor,
        limiter,
    },
    res::{AppJson, AppResult},
};
use axum::{
    Extension, Router,
    extract::State,
    middleware,
    routing::{delete, get, patch, post},
};
use service::{AppState, dto, handlers::ApplicationManagementHandler, id::ApplicationId};
use std::time::Duration;
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "me/applications/{application_id}/rotate-client-secret",
    params(("application_id" = ApplicationId, Path)),
    responses(
        (status = OK, body = dto::response::RotatedClientSecret)
    )
)]
async fn rotate_client_secret(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
) -> AppResult<dto::response::RotatedClientSecret> {
    Ok(AppJson(
        ApplicationManagementHandler(state)
            .rotate_client_secret(application_id)
            .await?,
    ))
}

#[utoipa::path(
    patch, path = "me/applications/{application_id}/name",
    request_body = dto::request::ApplicationName,
    params(("application_id" = ApplicationId, Path)),
    responses((status = OK))
)]
async fn rename_application(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
    AppJson(application_name_dto): AppJson<dto::request::ApplicationName>,
) -> AppResult<()> {
    Ok(AppJson(
        ApplicationManagementHandler(state)
            .rename(application_id, application_name_dto)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "me/applications/{application_id}",
    params(("application_id" = ApplicationId, Path)),
    responses((status = OK))
)]
async fn delete_application(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
) -> AppResult<()> {
    Ok(AppJson(
        ApplicationManagementHandler(state)
            .delete(application_id)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "me/applications/{application_id}/redirect-urls",
    request_body = dto::request::ApplicationRedirectUrl,
    params(("application_id" = ApplicationId, Path)),
    responses((status = OK))
)]
async fn add_redirect_url(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
    AppJson(redirect_url_dto): AppJson<dto::request::ApplicationRedirectUrl>,
) -> AppResult<()> {
    Ok(AppJson(
        ApplicationManagementHandler(state)
            .add_redirect_url(application_id, redirect_url_dto)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "me/applications/{application_id}/redirect-urls",
    request_body = dto::request::ApplicationRedirectUrl,
    params(("application_id" = ApplicationId, Path)),
    responses((status = OK))
)]
async fn remove_redirect_url(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
    AppJson(redirect_url_dto): AppJson<dto::request::ApplicationRedirectUrl>,
) -> AppResult<()> {
    Ok(AppJson(
        ApplicationManagementHandler(state)
            .remove_redirect_url(application_id, redirect_url_dto)
            .await?,
    ))
}

#[utoipa::path(
    get, path = "me/applications/{application_id}/redirect-urls",
    params(("application_id" = ApplicationId, Path)),
    responses((status = OK, body = Vec<String>))
)]
async fn get_redirect_urls(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
) -> AppResult<Vec<String>> {
    Ok(AppJson(
        ApplicationManagementHandler(state)
            .list_redirect_urls(application_id)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/me/applications/{application_id}/rotate-client-secret",
            post(rotate_client_secret),
        )
        .route(
            "/me/applications/{application_id}/name",
            patch(rename_application),
        )
        .route(
            "/me/applications/{application_id}",
            delete(delete_application),
        )
        .route(
            "/me/applications/{application_id}/redirect-urls",
            post(add_redirect_url),
        )
        .route(
            "/me/applications/{application_id}/redirect-urls",
            delete(remove_redirect_url),
        )
        .route(
            "/me/applications/{application_id}/redirect-urls",
            get(get_redirect_urls),
        )
        .layer(limiter::basic::<GovernorAccountIdKeyExtractor>(
            5,
            Duration::from_secs(5),
        ))
        .layer(limiter::basic::<GovernorIpKeyExtractor>(
            5,
            Duration::from_secs(5),
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_application_ownership,
        ))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_session))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        rotate_client_secret, rename_application, delete_application,
        add_redirect_url, remove_redirect_url, get_redirect_urls
    ),
    components(schemas(
        dto::response::RotatedClientSecret,
        dto::request::ApplicationName,
        dto::request::ApplicationRedirectUrl,
    )),
    modifiers(&ApiDocAuthAddon),
    security(("session_jwt" = []))
)]
pub struct ApiDoc;
