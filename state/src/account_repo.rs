use async_trait::async_trait;
use service::{
    dto, entity,
    id::AccountId,
    repo::{AccountRepo, AccountRepoTransaction, RepoResult},
};

/// Account repository using PostgreSQL.
pub struct AccountRepoImpl;

#[async_trait]
impl AccountRepo for AccountRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AccountRepoTransaction>> {
        // todo!()

        Ok(Box::new(AccountRepoTransactionImpl))
    }

    async fn get_login_by_email(&self, _email: &str) -> RepoResult<Option<dto::repo::Login>> {
        todo!()
    }

    async fn get_login_by_username(&self, _username: &str) -> RepoResult<Option<dto::repo::Login>> {
        todo!()
    }

    async fn get_primary_username(&self, _id: AccountId) -> RepoResult<Option<String>> {
        todo!()
    }

    async fn get_nonexpiring_username_aliases(&self, _id: AccountId) -> RepoResult<Vec<String>> {
        todo!()
    }

    async fn get_primary_email(&self, _id: AccountId) -> RepoResult<Option<String>> {
        todo!()
    }

    async fn get_secondary_emails(&self, _id: AccountId) -> RepoResult<Vec<String>> {
        todo!()
    }

    async fn get_keys(&self, _id: AccountId) -> RepoResult<Option<dto::repo::Keys>> {
        todo!()
    }

    async fn get_account_flags(&self, _id: AccountId) -> RepoResult<Option<entity::AccountFlags>> {
        todo!()
    }

    async fn set_primary_email_if_current_is(
        &self,
        _id: AccountId,
        _current_primary_email: &str,
        _new_primary_email: &str,
    ) -> RepoResult<()> {
        todo!()
    }

    async fn remove_email_if_not_primary(&self, _id: AccountId, _email: &str) -> RepoResult<()> {
        todo!()
    }

    async fn is_username_taken(&self, _username: &str) -> RepoResult<bool> {
        todo!()
    }

    async fn is_email_taken(&self, _email: &str) -> RepoResult<bool> {
        todo!()
    }

    async fn is_email_taken_by(&self, _id: AccountId, _email: &str) -> RepoResult<bool> {
        todo!()
    }
}

struct AccountRepoTransactionImpl;

#[async_trait]
impl AccountRepoTransaction for AccountRepoTransactionImpl {
    async fn commit(self: Box<Self>) -> RepoResult<()> {
        todo!()
    }

    async fn upsert_account(
        &mut self,
        _id: AccountId,
        _password_hash: &str,
        _keys: &dto::repo::Keys,
    ) -> RepoResult<()> {
        todo!()
    }

    async fn insert_default_flags(&mut self, _id: AccountId) -> RepoResult<()> {
        todo!()
    }

    async fn add_email(
        &mut self,
        _id: AccountId,
        _email: &str,
        _is_primary: bool,
    ) -> RepoResult<()> {
        todo!()
    }

    async fn add_username(
        &mut self,
        _id: AccountId,
        _username: &str,
        _is_primary: bool,
    ) -> RepoResult<()> {
        todo!()
    }

    async fn set_verified_flag(&mut self, _id: AccountId, _is_verified: bool) -> RepoResult<()> {
        todo!()
    }
}
