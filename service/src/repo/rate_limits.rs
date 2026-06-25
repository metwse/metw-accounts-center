/// Email rate limiting configuration.
pub mod email_limiting_repo {
    use std::time::Duration;

    /// Minimum time between two emails sent to the same address.
    pub static EMAIL_COOLDOWN: Duration = Duration::from_mins(1);

    /// Minimum time between two email requests originating from the same IP.
    pub static IP_COOLDOWN: Duration = Duration::from_mins(1);

    /// Maximum quota for a single email address.
    pub static EMAIL_QUOTA: u64 = 5;

    /// Time required to fully replenish the email quota.
    pub static EMAIL_QUOTA_REFILL_DURATION: Duration = Duration::from_hours(24);

    /// Maximum quota for a single IP address.
    pub static IP_QUOTA: u64 = 10;

    /// Time required to fully replenish the IP quota.
    pub static IP_QUOTA_REFILL_DURATION: Duration = Duration::from_hours(24);
}
