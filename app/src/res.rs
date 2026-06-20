use axum::{
    Json,
    extract::{FromRequest, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use service::{handlers::HandlerError, service::ServiceError};
use thiserror::Error;

/// Application error reporting.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum AppError {
    #[error("handler: {0}")]
    Handler(#[from] HandlerError),

    #[error("json rejection: {0}")]
    JsonRejection(#[from] JsonRejection),
}

/// API result.
pub type AppResult<T> = Result<AppJson<T>, AppError>;

/// Middleware result.
pub type AppMiddlewareResult<T> = Result<T, AppError>;

/// Axum JSON extractor and response wrapper.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
            #[cfg(feature = "transparent-errors")]
            debug: Option<HandlerError>,
        }

        let status_code = self.status_code();

        let body = ErrorResponse {
            message: self.message(),
            #[cfg(feature = "transparent-errors")]
            debug: match self {
                Self::Handler(handler_error) => Some(handler_error),
                _ => None,
            },
        };

        (status_code, Json(body)).into_response()
    }
}

impl<T: Serialize> IntoResponse for AppJson<T> {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::JsonRejection(..) => StatusCode::BAD_REQUEST,

            Self::Handler(handler_error) => match handler_error {
                HandlerError::Service(service_error) => match service_error {
                    ServiceError::Repo(..) => StatusCode::CONFLICT,

                    ServiceError::EmailNotFound | ServiceError::AccountNotFound => {
                        StatusCode::NOT_FOUND
                    }

                    _ => StatusCode::BAD_REQUEST,
                },
                HandlerError::Validation(..) => StatusCode::BAD_REQUEST,
                HandlerError::Unauthorized => StatusCode::UNAUTHORIZED,
                HandlerError::UnexpectedError(..) => StatusCode::CONFLICT,
            },
        }
    }

    fn message(&self) -> String {
        #[cfg(feature = "transparent-errors")]
        return self.to_string();

        #[cfg(not(feature = "transparent-errors"))]
        match self {
            Self::Handler(handler_error) => match handler_error {
                HandlerError::Service(service_error) => match service_error {
                    ServiceError::Repo(repo_error) => {
                        tracing::error!(?repo_error, "Repository error");

                        "error details are redacted".to_string()
                    }

                    ServiceError::InvalidJwt | ServiceError::TokenRevoked => {
                        "invalid JWT".to_string()
                    }

                    service_error => service_error.to_string(),
                },

                handler_error => handler_error.to_string(),
            },

            app_error => app_error.to_string(),
        }
    }
}
