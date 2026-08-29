use crate::res::AppMiddlewareResult;
use axum::{
    Extension,
    extract::{Path, Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use service::{
    AppState,
    handlers::{ApplicationManagementHandler, AuthenticationHandler, HandlerError},
    id::{AccountId, ApplicationId},
};
use tower_governor::key_extractor::KeyExtractor;
use utoipa::{Modify, openapi};

fn extract_bearer_token(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|bearer_token| bearer_token.trim().strip_prefix("Bearer "))
        .map(|token_str| token_str.to_string())
}

/// Authenticate a login session.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn require_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppMiddlewareResult<Response> {
    let Some(token) = extract_bearer_token(&req) else {
        return Err(HandlerError::Unauthorized)?;
    };

    match AuthenticationHandler(state)
        .auth_session(token.to_string())
        .await
    {
        Ok(account_id) => {
            req.extensions_mut().insert(account_id);

            Ok(next.run(req).await)
        }
        Err(_) => Err(HandlerError::Unauthorized)?,
    }
}

/// Authenticate the login session before email verification.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn require_email_verification_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppMiddlewareResult<Response> {
    let Some(token) = extract_bearer_token(&req) else {
        return Err(HandlerError::Unauthorized)?;
    };

    match AuthenticationHandler(state)
        .auth_email_verification_session(token.to_string())
        .await
    {
        Ok(account_id) => {
            req.extensions_mut().insert(account_id);

            Ok(next.run(req).await)
        }
        Err(_) => Err(HandlerError::Unauthorized)?,
    }
}

/// Authenticate the login session before email verification.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn require_application_ownership(
    State(state): State<AppState>,
    Extension(account_id): Extension<AccountId>,
    Path(application_id): Path<ApplicationId>,
    mut req: Request,
    next: Next,
) -> AppMiddlewareResult<Response> {
    if ApplicationManagementHandler(state)
        .authorize_ownership(application_id, account_id)
        .await
        .is_ok()
    {
        req.extensions_mut().insert(application_id);

        Ok(next.run(req).await)
    } else {
        Err(HandlerError::Unauthorized)?
    }
}

/// utoipa modifiers for middleware documentation.
pub struct ApiDocSecurityAddon;

impl Modify for ApiDocSecurityAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session_jwt",
                openapi::security::SecurityScheme::Http(
                    openapi::security::HttpBuilder::new()
                        .scheme(openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );

            components.add_security_scheme(
                "email_verification_session_jwt",
                openapi::security::SecurityScheme::Http(
                    openapi::security::HttpBuilder::new()
                        .scheme(openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// A key extractor that tries to get rate limiting key from the extension
/// added by [`require_session`] or [`require_email_verification_session`].
#[derive(Clone, Default)]
pub struct GovernorAccountIdKeyExtractor;

impl KeyExtractor for GovernorAccountIdKeyExtractor {
    type Key = AccountId;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, tower_governor::GovernorError> {
        Ok(*req.extensions().get::<AccountId>().unwrap())
    }
}
