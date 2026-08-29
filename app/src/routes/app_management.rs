//! See [`AppManagementHandler`].

use crate::{
    middleware::{
        auth::{ApiDocAuthAddon, GovernorAccountIdKeyExtractor, auth_app_owership, auth_session},
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
use service::{AppState, dto, handlers::AppManagementHandler, id::AppId};
use std::time::Duration;
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "me/apps/{app_id}/roll-client-secret",
    params(("app_id" = AppId, Path)),
    responses(
        (status = OK, body = dto::response::AppClientSecret)
    )
)]
async fn roll_client_secret(
    State(state): State<AppState>,
    Extension(app_id): Extension<AppId>,
) -> AppResult<dto::response::AppClientSecret> {
    Ok(AppJson(
        AppManagementHandler(state)
            .roll_client_secret(app_id)
            .await?,
    ))
}

#[utoipa::path(
    patch, path = "me/apps/{app_id}/name",
    request_body = dto::request::AppName,
    params(("app_id" = AppId, Path)),
    responses((status = OK))
)]
async fn rename_app(
    State(state): State<AppState>,
    Extension(app_id): Extension<AppId>,
    AppJson(app_name_dto): AppJson<dto::request::AppName>,
) -> AppResult<()> {
    Ok(AppJson(
        AppManagementHandler(state)
            .rename_app(app_id, app_name_dto)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "me/apps/{app_id}",
    params(("app_id" = AppId, Path)),
    responses((status = OK))
)]
async fn delete_app(
    State(state): State<AppState>,
    Extension(app_id): Extension<AppId>,
) -> AppResult<()> {
    Ok(AppJson(
        AppManagementHandler(state).delete_app(app_id).await?,
    ))
}

#[utoipa::path(
    post, path = "me/apps/{app_id}/redirect-urls",
    request_body = dto::request::RedirectUrl,
    params(("app_id" = AppId, Path)),
    responses((status = OK))
)]
async fn add_redirect_url(
    State(state): State<AppState>,
    Extension(app_id): Extension<AppId>,
    AppJson(redirect_url_dto): AppJson<dto::request::RedirectUrl>,
) -> AppResult<()> {
    Ok(AppJson(
        AppManagementHandler(state)
            .add_redirect_url(app_id, redirect_url_dto)
            .await?,
    ))
}

#[utoipa::path(
    delete, path = "me/apps/{app_id}/redirect-urls",
    request_body = dto::request::RedirectUrl,
    params(("app_id" = AppId, Path)),
    responses((status = OK))
)]
async fn remove_redirect_url(
    State(state): State<AppState>,
    Extension(app_id): Extension<AppId>,
    AppJson(redirect_url_dto): AppJson<dto::request::RedirectUrl>,
) -> AppResult<()> {
    Ok(AppJson(
        AppManagementHandler(state)
            .remove_redirect_url(app_id, redirect_url_dto)
            .await?,
    ))
}

#[utoipa::path(
    get, path = "me/apps/{app_id}/redirect-urls",
    params(("app_id" = AppId, Path)),
    responses((status = OK, body = Vec<String>))
)]
async fn get_redirect_urls(
    State(state): State<AppState>,
    Extension(app_id): Extension<AppId>,
) -> AppResult<Vec<String>> {
    Ok(AppJson(
        AppManagementHandler(state)
            .get_redirect_urls(app_id)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/me/apps/{app_id}/roll-client-secret",
            post(roll_client_secret),
        )
        .route("/me/apps/{app_id}/name", patch(rename_app))
        .route("/me/apps/{app_id}", delete(delete_app))
        .route("/me/apps/{app_id}/redirect-urls", post(add_redirect_url))
        .route(
            "/me/apps/{app_id}/redirect-urls",
            delete(remove_redirect_url),
        )
        .route("/me/apps/{app_id}/redirect-urls", get(get_redirect_urls))
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
            auth_app_owership,
        ))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_session))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        roll_client_secret, rename_app, delete_app,
        add_redirect_url, remove_redirect_url, get_redirect_urls
    ),
    components(schemas(
        dto::response::AppClientSecret,
        dto::request::AppName,
        dto::request::RedirectUrl,
    )),
    modifiers(&ApiDocAuthAddon),
    security(("session_jwt" = []))
)]
pub struct ApiDoc;
