use super::super::{EmailLimitingRepo, RepoResult};
use crate::dto;
use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

#[derive(Default)]
struct State {
    block_email: HashSet<String>,
    used_quota_email: HashMap<String, (usize, usize)>,
    block_ip: HashMap<IpAddr, String>,
    used_quota_ip: HashMap<IpAddr, (usize, usize)>,
    quota_nonce: usize,
}

/// Mock email limiting repo implementation.
///
/// The mock implementation does not return real timeout values, instead it
/// returns the maximum.
#[derive(Default)]
pub struct MockEmailLimitingRepoImpl {
    state: Arc<Mutex<State>>,
}

impl MockEmailLimitingRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }
}

#[async_trait]
impl EmailLimitingRepo for MockEmailLimitingRepoImpl {
    async fn check_and_limit_email(
        &self,
        ip: &IpAddr,
        email: &str,
    ) -> RepoResult<dto::repo::EmailLimitingResult> {
        let mut state = self.state.lock().await;

        if state.block_email.contains(email) {
            return Ok(dto::repo::EmailLimitingResult::EmailTimeOut(60));
        }

        if state.block_ip.contains_key(ip) {
            return Ok(dto::repo::EmailLimitingResult::IpTimeOut(60));
        }

        if let Some(&(used_quota, _)) = state.used_quota_email.get(email)
            && used_quota >= 5
        {
            return Ok(dto::repo::EmailLimitingResult::EmailTimeOut(60 * 60 * 24));
        }

        if let Some(&(used_quota, _)) = state.used_quota_ip.get(ip)
            && used_quota >= 10
        {
            return Ok(dto::repo::EmailLimitingResult::IpTimeOut(60 * 60 * 24));
        }

        self.block_email_and_ip(&mut state, email, ip);
        self.email_use_quaota(&mut state, email);
        self.ip_use_quaota(&mut state, ip);

        Ok(dto::repo::EmailLimitingResult::NoTimeOut)
    }

    async fn reclaim_ip_quota(&self, ip: &IpAddr, email: &str) -> RepoResult<()> {
        let mut state = self.state.lock().await;

        if let Some(ip_blocked_for) = state.block_ip.get(ip)
            && *ip_blocked_for == email
        {
            state.block_ip.remove(ip);
        }

        let mut quota_hit_zero = false;
        if let Some((used_quota, _)) = state.used_quota_ip.get_mut(ip) {
            *used_quota -= 1;
            quota_hit_zero = *used_quota == 0;
        }

        if quota_hit_zero {
            state.used_quota_ip.remove(ip);
        }

        Ok(())
    }

    async fn clear_email_limit(&self, email: &str) -> RepoResult<()> {
        let mut state = self.state.lock().await;

        state.block_email.remove(email);
        state.used_quota_email.remove(email);

        Ok(())
    }
}

impl MockEmailLimitingRepoImpl {
    fn block_email_and_ip(&self, state: &mut State, email: &str, ip: &IpAddr) {
        let email = email.to_string();

        state.block_email.insert(email.clone());
        state.block_ip.insert(*ip, email.clone());

        let state = Arc::clone(&self.state);
        let ip = *ip;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_mins(1)).await;

            let mut state = state.lock().await;

            state.block_email.remove(&email);

            if let Some(ip_blocked_for) = state.block_ip.get(&ip)
                && *ip_blocked_for == email
            {
                state.block_ip.remove(&ip);
            }
        });
    }

    fn ip_use_quaota(&self, state: &mut State, ip: &IpAddr) {
        if let Some((used_quota, _)) = state.used_quota_ip.get_mut(ip) {
            *used_quota += 1;
        } else {
            let current_nonce = state.quota_nonce;

            state.used_quota_ip.insert(*ip, (1, current_nonce));
            state.quota_nonce += 1;

            let state = Arc::clone(&self.state);
            let ip = *ip;

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_hours(24)).await;

                let mut state = state.lock().await;

                if let Some((_, quota_nonce)) = state.used_quota_ip.get(&ip)
                    && *quota_nonce == current_nonce
                {
                    state.used_quota_ip.remove(&ip);
                }
            });
        }
    }

    fn email_use_quaota(&self, state: &mut State, email: &str) {
        if let Some((used_quota, _)) = state.used_quota_email.get_mut(email) {
            *used_quota += 1;
        } else {
            let email = email.to_string();

            let current_nonce = state.quota_nonce;

            state
                .used_quota_email
                .insert(email.clone(), (1, current_nonce));
            state.quota_nonce += 1;

            let state = Arc::clone(&self.state);

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_hours(24)).await;

                let mut state = state.lock().await;

                if let Some((_, quota_nonce)) = state.used_quota_email.get(&email)
                    && *quota_nonce == current_nonce
                {
                    state.used_quota_email.remove(&email);
                }
            });
        }
    }
}
