use crate::{
    dto,
    repo::EmailLimitingRepo,
    service::{ServiceError, ServiceResult},
};
use std::net::IpAddr;

/// Email limiting state.
pub struct EmailLimitingService {
    repo: Box<dyn EmailLimitingRepo>,
}

impl EmailLimitingService {
    /// Creates a new email limiting service.
    pub fn new(repo: Box<dyn EmailLimitingRepo>) -> Self {
        Self { repo }
    }

    /// Same as [`check_and_consume_quota`], but maps `IpLimited` and
    /// `EmailLimited` to `ServiceResult::EmailLimited`.
    ///
    /// [`check_and_consume_quota`]: EmailLimitingRepo::check_and_consume_quota
    pub async fn check_and_consume_quota(&self, ip: &IpAddr, email: &str) -> ServiceResult<()> {
        match self.repo.check_and_consume_quota(ip, email).await? {
            dto::repo::EmailLimitingResult::IpLimited(duration)
            | dto::repo::EmailLimitingResult::EmailLimited(duration) => {
                Err(ServiceError::EmailLimited(duration))
            }
            dto::repo::EmailLimitingResult::Allowed => Ok(()),
        }
    }

    /// See [`refund_ip_quota`].
    ///
    /// [`refund_ip_quota`]: EmailLimitingRepo::refund_ip_quota
    pub async fn refund_ip_quota(&self, ip: &IpAddr, email: &str) -> ServiceResult<()> {
        Ok(self.repo.refund_ip_quota(ip, email).await?)
    }

    /// See [`check_and_consume_quota`].
    ///
    /// [`check_and_consume_quota`]: EmailLimitingRepo::check_and_consume_quota
    pub async fn clear_email_limit(&self, email: &str) -> ServiceResult<()> {
        Ok(self.repo.clear_email_limit(email).await?)
    }
}
