use super::super::{EmailLimitingRepo, RepoResult};
use crate::{dto, repo::rate_limits::email_limiting_repo::*};
use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Default)]
struct State {
    block_email: HashSet<String>,
    used_quota_email: HashMap<String, (u64, u64)>,
    block_ip: HashMap<IpAddr, String>,
    used_quota_ip: HashMap<IpAddr, (u64, u64)>,
    quota_nonce: u64,
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
    async fn check_and_consume_quota(
        &self,
        ip: &IpAddr,
        email: &str,
    ) -> RepoResult<dto::repo::EmailLimitingResult> {
        let mut state = self.state.lock().await;

        if state.block_ip.contains_key(ip) {
            return Ok(dto::repo::EmailLimitingResult::IpLimited(IP_COOLDOWN));
        }

        if state.block_email.contains(email) {
            return Ok(dto::repo::EmailLimitingResult::EmailLimited(EMAIL_COOLDOWN));
        }

        if let Some(&(used_quota, _)) = state.used_quota_ip.get(ip)
            && used_quota >= IP_QUOTA
        {
            return Ok(dto::repo::EmailLimitingResult::IpLimited(
                EMAIL_QUOTA_REFILL_DURATION,
            ));
        }

        if let Some(&(used_quota, _)) = state.used_quota_email.get(email)
            && used_quota >= EMAIL_QUOTA
        {
            return Ok(dto::repo::EmailLimitingResult::EmailLimited(
                EMAIL_QUOTA_REFILL_DURATION,
            ));
        }

        self.block_email_and_ip(&mut state, email, ip);
        self.email_use_quaota(&mut state, email);
        self.ip_use_quaota(&mut state, ip);

        Ok(dto::repo::EmailLimitingResult::Allowed)
    }

    async fn refund_ip_quota(&self, ip: &IpAddr, email: &str) -> RepoResult<()> {
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

        tokio::spawn({
            let email = email.clone();
            let state = Arc::clone(&self.state);

            async move {
                tokio::time::sleep(EMAIL_COOLDOWN).await;

                let mut state = state.lock().await;

                state.block_email.remove(&email);
            }
        });

        tokio::spawn({
            let state = Arc::clone(&self.state);
            let ip = *ip;

            async move {
                tokio::time::sleep(IP_COOLDOWN).await;

                let mut state = state.lock().await;

                if let Some(ip_blocked_for) = state.block_ip.get(&ip)
                    && *ip_blocked_for == email
                {
                    state.block_ip.remove(&ip);
                }
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
                tokio::time::sleep(IP_QUOTA_REFILL_DURATION).await;

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
                tokio::time::sleep(EMAIL_QUOTA_REFILL_DURATION).await;

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
