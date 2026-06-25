use crate::res::{AppError, AppMiddlewareResult};
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
use std::{net::IpAddr, str::FromStr};
use utoipa::{Modify, openapi};

fn extract_token(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|bearer_token| bearer_token.trim().strip_prefix("Bearer "))
        .map(|token_str| token_str.to_string())
}

/// Extract the remote IP from X-Real-IP header.
#[tracing::instrument(skip_all)]
pub async fn extract_real_ip(mut req: Request, next: Next) -> AppMiddlewareResult<Response> {
    let real_ip: IpAddr = match req.headers().get("X-Real-IP") {
        Some(header_value) => header_value
            .to_str()
            .map_err(|_| AppError::MissingOrInvalidXRealIp)
            .and_then(|header| {
                IpAddr::from_str(header).map_err(|_| AppError::MissingOrInvalidXRealIp)
            })?,
        None => {
            #[cfg(debug_assertions)]
            {
                use service::testutil::random_ipv6;

                tracing::debug!("no X-Real-IP is given, using random IP address");

                random_ipv6()
            }

            #[cfg(not(debug_assertions))]
            return Err(AppError::MissingOrInvalidXRealIp);
        }
    };

    req.extensions_mut().insert(real_ip);

    Ok(next.run(req).await)
}

/// Authenticate a login session.
#[tracing::instrument(skip_all)]
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
#[tracing::instrument(skip_all)]
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
