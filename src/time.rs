use chrono::{DateTime, Local, TimeZone};

#[must_use]
pub fn parse_timestamp(s: &str) -> i64 {
    chrono_systemd_time::parse_timestamp_tz(s, Local)
        .unwrap_or_else(|err| panic!("McFly error: Failed to parse timestamp ({err})"))
        .latest()
        .timestamp()
}

#[inline]
#[must_use]
pub fn to_datetime(timestamp: i64) -> String {
    let utc = DateTime::from_timestamp(timestamp, 0).unwrap();
    Local.from_utc_datetime(&utc.naive_utc()).to_rfc3339()
}
