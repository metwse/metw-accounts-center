use crate::res::AppError;
use axum::{body::Body, response::IntoResponse};
use governor::middleware::NoOpMiddleware;
use std::time::Duration;
use tower_governor::{
    GovernorError, GovernorLayer, governor::GovernorConfigBuilder, key_extractor::KeyExtractor,
};

/// Basic rate limiter middleware.
pub fn basic<K: KeyExtractor + Default>(
    burst: u32,
    per: Duration,
) -> GovernorLayer<K, NoOpMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(K::default())
        .burst_size(burst)
        .period(per)
        .finish()
        .unwrap();

    GovernorLayer::new(config).error_handler(|governor_error| match governor_error {
        GovernorError::TooManyRequests { wait_time, .. } => {
            AppError::RateLimited(Duration::from_secs(wait_time)).into_response()
        }
        _ => unreachable!(),
    })
}
