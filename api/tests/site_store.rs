use serde_json::json;
use taozhiyy_api::site_store::{entry_json, known_kind, NewEntry, StoreError};

fn sample(kind: &str, title: &str, body: &str, image: &str) -> NewEntry {
    NewEntry {
        kind: kind.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        image_url: image.to_string(),
        extra: json!({}),
    }
}

#[test]
fn article_text_is_accepted_and_the_image_must_be_a_public_url() {
    assert!(known_kind("article"));
    assert!(!known_kind("secret"));
    let saved = taozhiyy_api::site_store::prepare(sample(
        "article",
        "清沫",
        "一段正文",
        "https://fs.964643.xyz/cover.jpg",
    ))
    .expect("article");
    assert_eq!(saved.title, "清沫");
    assert_eq!(saved.image_url, "https://fs.964643.xyz/cover.jpg");
    assert!(taozhiyy_api::site_store::prepare(sample(
        "article",
        "清沫",
        "一段正文",
        "javascript:alert(1)",
    ))
    .is_err());
    assert!(taozhiyy_api::site_store::prepare(sample("journal", "日志", "", "")).is_err());
}

#[test]
fn gallery_and_project_require_an_s3_url_and_bili_requires_a_bvid() {
    match taozhiyy_api::site_store::prepare(sample("gallery", "旅途", "", "")) {
        Err(StoreError::Invalid(message)) => assert_eq!(message, "请先把图片上传到 S3"),
        Err(error) => panic!("{error}"),
        Ok(_) => panic!("gallery without an image was saved"),
    }
    let project = taozhiyy_api::site_store::prepare(NewEntry {
        kind: "project".into(),
        title: "星图".into(),
        body: "一颗新的星".into(),
        image_url: "https://fs.964643.xyz/star.jpg".into(),
        extra: json!({"url": "https://www.miku.coffee/", "tags": "星空"}),
    })
    .expect("project");
    assert_eq!(project.extra["url"], "https://www.miku.coffee/");
    assert!(taozhiyy_api::site_store::prepare(NewEntry {
        kind: "bili".into(),
        title: "演示".into(),
        body: "".into(),
        image_url: "https://fs.964643.xyz/v.jpg".into(),
        extra: json!({"bvid": "nope"}),
    })
    .is_err());
}

#[test]
fn entry_json_keeps_the_body_and_the_image_address() {
    let entry = taozhiyy_api::site_store::Entry {
        id: 7,
        kind: "about".into(),
        title: "近况".into(),
        body: "在写站点".into(),
        image_url: "https://fs.964643.xyz/me.jpg".into(),
        extra: json!({}),
        created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2026, 11, 16, 4, 0, 0).unwrap(),
    };
    let value = entry_json(&entry);
    assert_eq!(value["body"], "在写站点");
    assert_eq!(value["imageUrl"], "https://fs.964643.xyz/me.jpg");
}
