// TODO: Better error handling

use axum::response::IntoResponse;
use service::handlers::{HandlerError, HandlerResult};
use std::ops::Deref;

pub struct AppResult<T>(Result<T, HandlerError>);

impl<T> IntoResponse for AppResult<T> {
    fn into_response(self) -> axum::response::Response {
        todo!()
    }
}

impl<T> From<HandlerResult<T>> for AppResult<T> {
    fn from(value: HandlerResult<T>) -> Self {
        Self(value)
    }
}

impl<T> Deref for AppResult<T> {
    type Target = Result<T, HandlerError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
