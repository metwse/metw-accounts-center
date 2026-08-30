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

/// Account repository limits.
pub mod account_repo {
    /// The number of email addresses allowed to be added.
    pub static MAXIMUM_EMAIL_COUNT: usize = 10;
}

/// Application repository limits.
pub mod application_repo {
    /// The number of applications allowed per account to be created.
    pub static MAXIMUM_APPLICATION_COUNT: usize = 10;

    /// The number of redirect URLs allowed per application to be added.
    pub static MAXIMUM_REDIRECT_URL_COUNT: usize = 10;
}
