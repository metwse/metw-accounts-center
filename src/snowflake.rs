use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use lazy_static::lazy_static;
use std::{sync::Mutex, time::Duration};

lazy_static! {
    /// Year zero of metw.cc.
    ///
    /// 2022 Aug 12, 00:00:00 (`1660262400000`)
    pub static ref EPOCH: u64 = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2022, 8, 12).unwrap(),
        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    ).and_utc().timestamp_millis() as u64;
}

static LAST_OVERFLOW: Mutex<i64> = Mutex::new(0);
static INCREMENT_MAX: i64 = 2i64.pow(12);

static INCREMENT: Mutex<i64> = Mutex::new(0);

/// UUID generator inspired from Twitter's snowflake format.
///
/// | Field | Bits | Description |
/// | -- | -- | -- |
/// | Timestamp | 22 to 63 | Milliseconds since metw.cc [`EPOCH`] |
/// | Reserved for future use | 12 to 21 | |
/// | Increment | 0 to 11 | For every ID that is generated, this number is incremented |
pub fn snowflake() -> i64 {
    let timestamp = Utc::now().timestamp_millis() - *EPOCH as i64;

    // Ensure the time is not yet May 15 2109 07:35:11
    assert!(timestamp < 2i64.pow(42) - 1);

    let mut increment = INCREMENT.lock().unwrap();
    let mut last_overflow = LAST_OVERFLOW.lock().unwrap();

    if *increment == 0 {
        *last_overflow = timestamp;
    }

    *increment += 1;

    if *increment == INCREMENT_MAX {
        *increment = 0;

        if *last_overflow == timestamp {
            std::thread::sleep(Duration::from_millis(1));
            return snowflake();
        }
    }

    (timestamp << 22) | *increment
}

#[cfg(test)]
#[test]
#[serial_test::serial]
fn test_snowflake() {
    use std::collections::HashSet;

    let cap = 2usize.pow(16);
    let mut snowflakes = HashSet::with_capacity(cap);

    for _ in 0..cap {
        let snowflake = snowflake();

        assert!(!snowflakes.contains(&snowflake));

        snowflakes.insert(snowflake);
    }
}
