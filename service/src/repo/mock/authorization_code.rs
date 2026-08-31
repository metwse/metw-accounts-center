use super::super::{AuthorizationCodeRepo, RepoResult};
use crate::{
    id::{AccountId, ApplicationId},
    util::random_authorization_code,
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Mock authorization code repo implementation.
#[derive(Default)]
pub struct MockAuthorizationCodeRepoImpl {
    authorization_codes: Arc<Mutex<HashMap<(ApplicationId, String), AccountId>>>,
}

impl MockAuthorizationCodeRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }
}

#[async_trait]
impl AuthorizationCodeRepo for MockAuthorizationCodeRepoImpl {
    async fn create(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> RepoResult<String> {
        let mut authorization_codes = self.authorization_codes.lock().await;
        let authorization_code = random_authorization_code();

        authorization_codes.insert((application_id, authorization_code.clone()), account_id);

        Ok(authorization_code)
    }

    async fn consume(
        &self,
        application_id: ApplicationId,
        authorization_code: &str,
    ) -> RepoResult<Option<AccountId>> {
        let mut authorization_codes = self.authorization_codes.lock().await;

        Ok(authorization_codes.remove(&(application_id, authorization_code.to_string())))
    }
}
