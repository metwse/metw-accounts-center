use crate::res::AppMiddlewareResult;
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use service::{
    AppState,
    handlers::{AuthenticationHandler, HandlerError},
};
use utoipa::{Modify, openapi};

fn extract_token(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|bearer_token| bearer_token.trim().strip_prefix("Bearer "))
        .map(|token_str| token_str.to_string())
}

/// Authenticate a login session.
pub async fn auth_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppMiddlewareResult<Response> {
    let Some(token) = extract_token(&req) else {
        return Err(HandlerError::Unauthorized)?;
    };

    match AuthenticationHandler(state)
        .auth_session(token.to_string())
        .await
    {
        Ok(id) => {
            req.extensions_mut().insert(id);

            Ok(next.run(req).await)
        }
        Err(_) => Err(HandlerError::Unauthorized)?,
    }
}

/// Authenticate the login session before email verification.
pub async fn auth_email_verification_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppMiddlewareResult<Response> {
    let Some(token) = extract_token(&req) else {
        return Err(HandlerError::Unauthorized)?;
    };

    match AuthenticationHandler(state)
        .auth_email_verification_session(token.to_string())
        .await
    {
        Ok(id) => {
            req.extensions_mut().insert(id);

            Ok(next.run(req).await)
        }
        Err(_) => Err(HandlerError::Unauthorized)?,
    }
}

/// utoipa modifiers for middleware documentations.
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
