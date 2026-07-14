use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use service::{
    dto,
    repo::{
        EmailLimitingRepo, RepoResult,
        rate_limits::email_limiting_repo::{
            EMAIL_COOLDOWN, EMAIL_QUOTA, EMAIL_QUOTA_REFILL_DURATION, IP_COOLDOWN, IP_QUOTA,
            IP_QUOTA_REFILL_DURATION,
        },
    },
};
use std::{net::IpAddr, time::Duration};
use tokio::sync::Mutex;

/// Email limiting repository using Redis.
pub struct EmailLimitingRepoImpl {
    con: MultiplexedConnection,
    transaction_con_check_and_consume_quota: Mutex<MultiplexedConnection>,
    transaction_con_refund_ip_quota: Mutex<MultiplexedConnection>,
}

impl EmailLimitingRepoImpl {
    /// Creates a new token repository.
    pub async fn boxed_new(con_generator: &impl AsyncFn() -> MultiplexedConnection) -> Box<Self> {
        Box::new(Self {
            con: con_generator().await,
            transaction_con_check_and_consume_quota: Mutex::new(con_generator().await),
            transaction_con_refund_ip_quota: Mutex::new(con_generator().await),
        })
    }
}

#[async_trait]
impl EmailLimitingRepo for EmailLimitingRepoImpl {
    async fn check_and_consume_quota(
        &self,
        ip: &IpAddr,
        email: &str,
    ) -> RepoResult<dto::repo::EmailLimitingResult> {
        let transaction_con_guard = self
            .transaction_con_check_and_consume_quota
            .lock()
            .await;
        let con = transaction_con_guard.clone();

        let used_email_quota_key = to_used_email_quota_key(email);
        let block_email_key = to_block_email_key(email);
        let used_ip_quota_key = to_used_ip_quota_key(ip);
        let block_ip_key = to_block_ip_key(ip);

        let (
            block_email_ttl,
            block_ip_ttl,
            used_email_quota,
            used_ip_quota,
            used_email_quota_ttl,
            used_ip_quota_ttl,
        ): (i64, i64, Option<u64>, Option<u64>, i64, i64) = redis::aio::transaction_async(
            con,
            &[
                &used_email_quota_key,
                &block_email_key,
                &used_ip_quota_key,
                &block_ip_key,
            ],
            |mut con, mut pipe| {
                let used_email_quota_key = used_email_quota_key.clone();
                let block_email_key = block_email_key.clone();
                let used_ip_quota_key = used_ip_quota_key.clone();
                let block_ip_key = block_ip_key.clone();

                let email = email.to_string();

                async move {
                    let existing_used_email_quota: Option<i64> =
                        con.get(&used_email_quota_key).await?;
                    let existing_used_email_quota_ttl: i64 =
                        con.pttl(&used_email_quota_key).await?;
                    let block_email: bool = con.exists(&block_email_key).await?;

                    let existing_used_ip_quota: Option<i64> = con.get(&used_ip_quota_key).await?;
                    let existing_used_ip_quota_ttl: i64 = con.pttl(&used_ip_quota_key).await?;
                    let block_ip: bool = con.exists(&block_ip_key).await?;

                    let blocked = block_ip
                        || block_email
                        || existing_used_email_quota.is_some_and(|v| v >= EMAIL_QUOTA as i64)
                        || existing_used_ip_quota.is_some_and(|v| v >= IP_QUOTA as i64);

                    let pipe = pipe
                        .pttl(&block_email_key)
                        .pttl(&block_ip_key)
                        .get(&used_email_quota_key)
                        .get(&used_ip_quota_key)
                        .pttl(&used_email_quota_key)
                        .pttl(&used_ip_quota_key);

                    if !blocked {
                        let new_used_email_quota = existing_used_email_quota.unwrap_or(0) + 1;
                        let new_used_email_quota_ttl = if existing_used_email_quota_ttl < 0 {
                            EMAIL_QUOTA_REFILL_DURATION.as_millis() as u64
                        } else {
                            existing_used_email_quota_ttl as u64
                        };

                        let new_used_ip_quota = existing_used_ip_quota.unwrap_or(0) + 1;
                        let new_used_ip_quota_ttl = if existing_used_ip_quota_ttl < 0 {
                            IP_QUOTA_REFILL_DURATION.as_millis() as u64
                        } else {
                            existing_used_ip_quota_ttl as u64
                        };

                        pipe.set_ex(block_email_key, "", EMAIL_COOLDOWN.as_secs())
                            .ignore()
                            .set_ex(block_ip_key, email, IP_COOLDOWN.as_secs())
                            .ignore()
                            .pset_ex(
                                used_email_quota_key,
                                new_used_email_quota,
                                new_used_email_quota_ttl,
                            )
                            .ignore()
                            .pset_ex(used_ip_quota_key, new_used_ip_quota, new_used_ip_quota_ttl)
                            .ignore()
                    } else {
                        pipe
                    }
                    .query_async(&mut con)
                    .await
                }
            },
        )
        .await?;

        let used_email_quota = used_email_quota.unwrap_or(0);
        let used_ip_quota = used_ip_quota.unwrap_or(0);

        if block_ip_ttl > 0 || used_ip_quota >= IP_QUOTA {
            Ok(dto::repo::EmailLimitingResult::IpLimited(
                if used_ip_quota >= IP_QUOTA {
                    Duration::from_millis(used_ip_quota_ttl as u64)
                } else {
                    Duration::from_millis(block_ip_ttl as u64)
                },
            ))
        } else if block_email_ttl > 0 || used_email_quota >= EMAIL_QUOTA {
            Ok(dto::repo::EmailLimitingResult::EmailLimited(
                if used_email_quota >= EMAIL_QUOTA {
                    Duration::from_millis(used_email_quota_ttl as u64)
                } else {
                    Duration::from_millis(block_email_ttl as u64)
                },
            ))
        } else {
            Ok(dto::repo::EmailLimitingResult::Allowed)
        }
    }

    async fn refund_ip_quota(&self, ip: &IpAddr, email: &str) -> RepoResult<()> {
        let transaction_con_guard = self.transaction_con_refund_ip_quota.lock().await;
        let con = transaction_con_guard.clone();

        let used_ip_quota_key = to_used_ip_quota_key(ip);
        let block_ip_key = to_block_ip_key(ip);

        let _: () = redis::aio::transaction_async(
            con,
            &[&used_ip_quota_key, &block_ip_key],
            |mut con, mut pipe| {
                let used_ip_quota_key = used_ip_quota_key.clone();
                let block_ip_key = block_ip_key.clone();

                let email = email.to_string();

                async move {
                    let existing_used_ip_quota: Option<i64> = con.get(&used_ip_quota_key).await?;
                    let existing_used_ip_quota_ttl: u64 = con.pttl(&used_ip_quota_key).await?;
                    let block_ip_for: Option<String> = con.get(&block_ip_key).await?;

                    let pipe = if let Some(existing_used_ip_quota) = existing_used_ip_quota
                        && existing_used_ip_quota > 1
                    {
                        pipe.pset_ex(
                            &used_ip_quota_key,
                            existing_used_ip_quota - 1,
                            existing_used_ip_quota_ttl,
                        )
                        .ignore()
                    } else {
                        pipe.del(&used_ip_quota_key).ignore()
                    };

                    let pipe = if let Some(block_ip_for) = block_ip_for
                        && block_ip_for == email
                    {
                        pipe.del(&block_ip_key).ignore()
                    } else {
                        pipe
                    };

                    pipe.query_async(&mut con).await
                }
            },
        )
        .await?;

        Ok(())
    }

    async fn clear_email_limit(&self, email: &str) -> RepoResult<()> {
        let mut con = self.con.clone();

        let used_email_quota_key = to_used_email_quota_key(email);
        let block_email_key = to_block_email_key(email);

        let _: () = con.del(&used_email_quota_key).await?;
        let _: () = con.del(&block_email_key).await?;

        Ok(())
    }
}

/// Key for limiting an email address.
pub fn to_used_email_quota_key(email: &str) -> String {
    format!("email-limiting:email:{email}:used-quota")
}

/// Key for temporarily blocking an email address.
pub fn to_block_email_key(email: &str) -> String {
    format!("email-limiting:email:{email}:block")
}

/// Key for limiting an IP address.
pub fn to_used_ip_quota_key(ip: &IpAddr) -> String {
    format!("email-limiting:ip:{ip}:used-quota")
}

/// Key for temporarily blocking an IP address.
pub fn to_block_ip_key(ip: &IpAddr) -> String {
    format!("email-limiting:ip:{ip}:block")
}
