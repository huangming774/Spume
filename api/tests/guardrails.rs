use std::time::Duration;

use taozhiyy_api::audit::normalize;
use taozhiyy_api::mail::{owner_body, owner_subject, GuestbookNotice};
use taozhiyy_api::rate_limit::{pipeline_count, RateGate};

#[test]
fn redis_pipeline_count_reads_the_increment() {
    let body = serde_json::json!([{"result": 3}, {"result": 1}]);
    assert_eq!(pipeline_count(&body).expect("count"), 3);
    assert!(pipeline_count(&serde_json::json!({"result": "nope"})).is_err());
}

#[test]
fn rate_gate_stops_at_the_limit() {
    let gate = RateGate::new(2, Duration::from_secs(60));
    assert!(gate.allow("203.0.113.8"));
    assert!(gate.allow("203.0.113.8"));
    assert!(!gate.allow("203.0.113.8"));
    assert!(gate.allow("203.0.113.9"));
}

#[test]
fn audit_text_collapses_whitespace_and_drops_secrets_by_length() {
    let detail = normalize("bad   password   hunter2", 12);
    assert_eq!(detail, "bad password");
    assert!(!detail.contains("hunter2"));
}

#[test]
fn guestbook_owner_mail_names_the_channel() {
    let notice = GuestbookNotice {
        channel: "friends",
        nickname: "访客",
        content: "想换友链",
        created_at: "2026-11-12T02:00:00Z",
        reply: false,
    };
    assert_eq!(
        owner_subject(notice.channel, notice.reply),
        "桃之夭夭：新的友链留言"
    );
    let body = owner_body(&notice);
    assert!(body.contains("友链留言"));
    assert!(body.contains("https://taozhiyy.top/friends"));
    assert!(body.contains("想换友链"));
}
