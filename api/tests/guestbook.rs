use chrono::TimeZone;
use serde_json::json;
use taozhiyy_api::guestbook::{
    accepted_page_size, check_content, compact_region_label, compose, hash_key, is_private_ip,
    looks_unsafe, mask_ip, normalize_channel, notice_detail, notice_title, posts_enabled,
    region_from_ip_api, thread_json, ComposeInput, StoredMessage,
};

#[test]
fn channels_and_page_size_follow_the_go_rules() {
    assert_eq!(normalize_channel(""), Some("guestbook"));
    assert_eq!(normalize_channel("friends"), Some("friends"));
    assert_eq!(normalize_channel("other"), None);
    assert!(posts_enabled(" YES "));
    assert!(!posts_enabled(""));
    assert_eq!(accepted_page_size(None), 20);
    assert_eq!(accepted_page_size(Some(50)), 50);
    assert_eq!(accepted_page_size(Some(51)), 20);
}

#[test]
fn scripts_and_encoded_tags_are_rejected() {
    assert!(looks_unsafe("<script>alert(1)</script>"));
    assert!(looks_unsafe("java\nscript:alert(1)"));
    assert!(looks_unsafe("&#60;img onerror=alert(1)&#62;"));
    assert!(!looks_unsafe("今天也想写一张小纸条"));
    assert!(check_content("").is_err());
    assert!(check_content("你好").is_ok());
}

#[test]
fn friends_root_requires_a_name_and_email() {
    let missing = compose(ComposeInput {
        channel: "friends",
        parent_id: 0,
        nickname: "看",
        email: "",
        account_name: None,
        account_avatar: "",
    });
    assert_eq!(missing.expect_err("email").code, "INVALID_EMAIL");

    let saved = compose(ComposeInput {
        channel: "guestbook",
        parent_id: 0,
        nickname: "路人",
        email: "",
        account_name: Some("站长"),
        account_avatar: "/a.png",
    })
    .expect("account");
    assert_eq!(saved.nickname, "站长");
    assert_eq!(saved.avatar, "/a.png");
}

#[test]
fn ip_mask_region_and_hash_match_the_visitor_contract() {
    assert_eq!(mask_ip("203.0.113.8"), "203.0.*.*");
    assert_eq!(mask_ip("203.0.113.8:443"), "203.0.*.*");
    assert_eq!(mask_ip("2001:db8::1"), "ipv6:*");
    assert_eq!(mask_ip(""), "*");
    assert!(is_private_ip("127.0.0.1"));
    assert!(is_private_ip("10.1.2.3"));
    assert!(!is_private_ip("203.0.113.8"));
    assert_eq!(compact_region_label("浙江 杭州"), "杭州");
    assert_eq!(compact_region_label("未知地区"), "访客");
    assert_eq!(compact_region_label("一个很长的城市名"), "一个很长的");
    let region = region_from_ip_api(r#"{"status":"success","regionName":"浙江","city":"杭州"}"#)
        .expect("region");
    assert_eq!(region, "浙江 杭州");
    assert_eq!(hash_key("ip", "203.0.113.8"), hash_key("ip", "203.0.113.8"));
    assert_ne!(hash_key("ip", "203.0.113.8"), hash_key("ua", "203.0.113.8"));
}

#[test]
fn owner_notice_names_the_guestbook_and_uses_china_time() {
    let created = chrono::Utc
        .with_ymd_and_hms(2020, 1, 2, 0, 0, 0)
        .single()
        .expect("time");
    assert_eq!(notice_title("guestbook"), "留言板消息");
    assert_eq!(notice_title("friends"), "朋友页消息");
    assert_eq!(
        notice_detail("guestbook", "  ", created),
        "留言板 · 访客 · 2020-01-02 08:00"
    );
}

#[test]
fn reply_threads_nest_and_hide_client_fields() {
    let created = chrono::Utc.with_ymd_and_hms(2026, 11, 12, 0, 0, 0).unwrap();
    let root = sample(1, 0, created);
    let mut child = sample(2, 1, created);
    child.content = "回复".into();
    let mut nested = sample(3, 2, created);
    nested.content = "再回复".into();
    nested.ip_masked = "203.0.*.*".into();
    let public = thread_json(
        vec![root.clone()],
        vec![child.clone(), nested.clone()],
        false,
    );
    assert_eq!(public[0]["replyCount"], json!(2));
    assert_eq!(
        public[0]["replies"][0]["replies"][0]["content"],
        json!("再回复")
    );
    assert!(public[0]["replies"][0]["replies"][0]
        .get("ipMasked")
        .is_none());
    let owner = thread_json(vec![root], vec![child, nested], true);
    assert_eq!(
        owner[0]["replies"][0]["replies"][0]["ipMasked"],
        json!("203.0.*.*")
    );
}

fn sample(id: i64, parent_id: i64, created: chrono::DateTime<chrono::Utc>) -> StoredMessage {
    StoredMessage {
        id,
        parent_id,
        nickname: "看".into(),
        avatar: String::new(),
        content: "楼".into(),
        ip_region: "本地".into(),
        is_login_user: false,
        is_admin_user: false,
        status: "visible".into(),
        ip_masked: String::new(),
        user_agent_hash: String::new(),
        created_at: created,
    }
}
