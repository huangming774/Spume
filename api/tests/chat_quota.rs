use chrono::{Duration, TimeZone, Utc};
use taozhiyy_api::chat_quota::{guest_key, pace_open};

#[test]
fn five_seconds_must_pass_between_questions() {
    let now = Utc
        .with_ymd_and_hms(2026, 11, 12, 2, 0, 0)
        .single()
        .expect("time");
    assert!(pace_open(None, now));
    assert!(!pace_open(Some(now - Duration::seconds(4)), now));
    assert!(pace_open(Some(now - Duration::seconds(5)), now));
}

#[test]
fn guest_key_uses_the_ip_and_user_agent() {
    let key = guest_key("203.0.113.8", "browser");
    assert!(key.starts_with("guest:"));
    assert_eq!(key, guest_key("203.0.113.8", "browser"));
    assert_ne!(key, guest_key("203.0.113.8", "other"));
    assert_ne!(key, guest_key("203.0.113.9", "browser"));
}
