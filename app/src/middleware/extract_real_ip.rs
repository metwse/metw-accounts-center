use crate::res::{AppError, AppMiddlewareResult};
use axum::{extract::Request, middleware::Next, response::Response};
use std::{net::IpAddr, str::FromStr};
use tower_governor::key_extractor::KeyExtractor;

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

/// A key extractor that tries to get rate limiting key from the extension
/// added by [`extract_real_ip`].
#[derive(Default, Clone)]
pub struct GovernorIpKeyExtractor;

impl KeyExtractor for GovernorIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, tower_governor::GovernorError> {
        Ok(*req.extensions().get::<IpAddr>().unwrap())
    }
}
