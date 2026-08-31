//! See [`ApplicationAccessHandler`].

use crate::{
    middleware::access_control::{ApiDocSecurityAddon, require_application_credentials},
    res::{AppJson, AppResult},
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    middleware,
    routing::{get, post},
};
use service::{
    AppState, dto,
    handlers::ApplicationAccessHandler,
    id::{AccountId, ApplicationId},
};
use utoipa::OpenApi;

#[utoipa::path(
    post, path = "application/exchange",
    request_body = dto::request::AuthorizationCode,
    responses(
        (status = OK, body = dto::response::AuthorizationCodeExchangeResult)
    ),
    security(("application_basic" = [])),
)]
async fn exchange_authorization_code(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
    AppJson(authorization_code_dto): AppJson<dto::request::AuthorizationCode>,
) -> AppResult<dto::response::AuthorizationCodeExchangeResult> {
    Ok(AppJson(
        ApplicationAccessHandler(state)
            .exchange(application_id, authorization_code_dto)
            .await?,
    ))
}

#[utoipa::path(
    get, path = "application/accounts/{account_id}",
    params(("account_id" = AccountId, Path)),
    responses(
        (status = OK, body = dto::response::Account)
    ),
    security(("application_basic" = [])),
)]
async fn get_account(
    State(state): State<AppState>,
    Extension(application_id): Extension<ApplicationId>,
    Path(account_id): Path<AccountId>,
) -> AppResult<dto::response::Account> {
    Ok(AppJson(
        ApplicationAccessHandler(state)
            .get_account(application_id, account_id)
            .await?,
    ))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/application/exchange", post(exchange_authorization_code))
        .route("/application/accounts/{account_id}", get(get_account))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_application_credentials,
        ))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        exchange_authorization_code, get_account
    ),
    components(schemas(
        dto::response::Account,
        dto::request::AuthorizationCode
    )),
    modifiers(&ApiDocSecurityAddon),
)]
pub struct ApiDoc;
