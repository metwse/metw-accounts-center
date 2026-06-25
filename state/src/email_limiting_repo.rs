use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use service::{
    dto,
    repo::{EmailLimitingRepo, RepoResult},
};
use std::net::IpAddr;

/// Email limiting repository using Redis.
pub struct EmailLimitingRepoImpl {
    con: MultiplexedConnection,
}

impl EmailLimitingRepoImpl {
    /// Creates a new token repository.
    pub fn boxed_new(con: MultiplexedConnection) -> Box<Self> {
        Box::new(Self { con })
    }
}

#[async_trait]
impl EmailLimitingRepo for EmailLimitingRepoImpl {
    async fn check_and_limit_email(
        &self,
        ip: &IpAddr,
        email: &str,
    ) -> RepoResult<dto::repo::EmailLimitingResult> {
        let con = self.con.clone();

        let used_email_quota_key = to_used_email_quota_key(email);
        let block_email_key = to_block_email_key(email);
        let used_ip_quota_key = to_used_ip_quota_key(ip);
        let block_ip_key = to_block_ip_key(ip);

        let (block_email_ttl, block_ip_ttl, used_email_quota_ttl, used_ip_quota_ttl): (
            i64,
            i64,
            i64,
            i64,
        ) = redis::aio::transaction_async(
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
                        | block_email
                        | existing_used_email_quota.is_some_and(|v| v >= 5)
                        | existing_used_ip_quota.is_some_and(|v| v >= 10);

                    let pipe = pipe
                        .ttl(&block_email_key)
                        .ttl(&block_ip_key)
                        .ttl(&used_email_quota_key)
                        .ttl(&used_ip_quota_key);

                    if !blocked {
                        let new_used_email_quota = existing_used_email_quota.unwrap_or(0) + 1;
                        let new_used_email_quota_ttl = if existing_used_email_quota_ttl < 0 {
                            60 * 60 * 24 * 1000
                        } else {
                            existing_used_email_quota_ttl as u64
                        };

                        let new_used_ip_quota = existing_used_ip_quota.unwrap_or(0) + 1;
                        let new_used_ip_quota_ttl = if existing_used_ip_quota_ttl < 0 {
                            60 * 60 * 24 * 1000
                        } else {
                            existing_used_ip_quota_ttl as u64
                        };

                        pipe.set_ex(block_email_key, "", 60)
                            .ignore()
                            .set_ex(block_ip_key, email, 60)
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

        if block_email_ttl > 0 || used_email_quota_ttl > 0 {
            Ok(dto::repo::EmailLimitingResult::EmailTimeOut(
                block_email_ttl.max(used_email_quota_ttl) as usize,
            ))
        } else if block_ip_ttl > 0 || used_ip_quota_ttl > 0 {
            Ok(dto::repo::EmailLimitingResult::IpTimeOut(
                block_ip_ttl.max(used_ip_quota_ttl) as usize,
            ))
        } else {
            Ok(dto::repo::EmailLimitingResult::NoTimeOut)
        }
    }

    async fn reclaim_ip_quota(&self, _ip: &IpAddr, _email: &str) -> RepoResult<()> {
        todo!()
    }

    async fn clear_email_limit(&self, _email: &str) -> RepoResult<()> {
        todo!()
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
