use chrono::{DateTime, Utc};
use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::Duration,
};

static LAST_TIMESTAMP: AtomicI64 = AtomicI64::new(0);

/// Current timestamp, with clock skew detection.
pub fn checked_now() -> DateTime<Utc> {
    loop {
        let now = Utc::now().timestamp_millis();
        let last = LAST_TIMESTAMP.load(Ordering::Relaxed);

        if now < last {
            let skew = last - now;

            if skew > 5000 {
                panic!("clock skew detected: {}ms", skew);
            }

            // reject this timestamp, retry
            std::thread::sleep(Duration::from_millis(skew as u64));
            continue;
        }

        let next = now.max(last);

        if LAST_TIMESTAMP
            .compare_exchange_weak(last, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return DateTime::<Utc>::from_timestamp_millis(next).unwrap();
        }
    }
}
