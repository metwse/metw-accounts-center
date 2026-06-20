use crate::res::AppResult;
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
) -> AppResult<Response> {
    let Some(token) = extract_token(&req) else {
        return Err(HandlerError::Unauthorized).into();
    };

    match AuthenticationHandler(state)
        .auth_session(token.to_string())
        .await
    {
        Ok(id) => {
            req.extensions_mut().insert(id);

            Ok(next.run(req).await).into()
        }
        Err(_) => Err(HandlerError::Unauthorized).into(),
    }
}

/// Authenticate the login session before email verification.
pub async fn auth_pending_activation_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let Some(token) = extract_token(&req) else {
        return Err(HandlerError::Unauthorized).into();
    };

    match AuthenticationHandler(state)
        .auth_pending_activation_session(token.to_string())
        .await
    {
        Ok(id) => {
            req.extensions_mut().insert(id);

            Ok(next.run(req).await).into()
        }
        Err(_) => Err(HandlerError::Unauthorized).into(),
    }
}
