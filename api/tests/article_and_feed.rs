use chrono::{TimeZone, Utc};
use serde_json::json;
use taozhiyy_api::feed::{creators, is_fresh, parse_bangumi};
use taozhiyy_api::post_markdown::{build_article, conflict_path, ArticleInput};

#[test]
fn article_keeps_an_existing_date_and_uses_the_requested_cover() {
    let now = Utc.with_ymd_and_hms(2026, 11, 12, 1, 2, 3).unwrap();
    let article = build_article(
        ArticleInput {
            title: "Spume",
            body: "---\ndate: 2026-06-01 08:00:00\ncover: https://old.example/a.jpg\ntags:\n  - 随笔\n---\n\n正文\n",
            cover_url: "https://images.example/new.jpg",
        },
        now,
    )
    .expect("article");
    assert_eq!(article.path, "blog/source/_posts/Spume.md");
    assert!(article.markdown.contains("title: Spume\n"));
    assert!(article.markdown.contains("date: 2026-06-01 08:00:00\n"));
    assert!(article
        .markdown
        .contains("cover: https://images.example/new.jpg\n"));
    assert!(article.markdown.contains("tags:\n"));
    assert!(article.markdown.ends_with("正文\n"));
}

#[test]
fn empty_article_gets_a_heading_and_conflict_path_keeps_the_extension() {
    let now = Utc.with_ymd_and_hms(2026, 11, 12, 1, 2, 3).unwrap();
    assert!(build_article(
        ArticleInput {
            title: "",
            body: "正文",
            cover_url: ""
        },
        now
    )
    .is_err());
    let article = build_article(
        ArticleInput {
            title: "Hello World",
            body: "---\ndate: 2020-01-01 00:00:00\n---\n",
            cover_url: "",
        },
        now,
    )
    .expect("heading");
    assert!(article.markdown.contains("# Hello World\n"));
    assert_eq!(
        conflict_path(&article.path, now),
        "blog/source/_posts/hello-world-20261112-010203.md"
    );
}

#[test]
fn bangumi_progress_and_cover_are_normalized() {
    let rows = parse_bangumi(&json!({
        "code": 0,
        "data": {"list": [{
            "season_id": 42,
            "title": "测试番",
            "cover": "//i0.hdslb.com/bfs/bangumi/cover.jpg",
            "total_count": 12,
            "progress": "看到第3话",
            "new_ep": {"title": "第4话", "index_show": ""},
            "url": "https://www.bilibili.com/bangumi/play/ss42"
        }]}
    }))
    .expect("bangumi");
    assert_eq!(rows[0].id, "s42");
    assert_eq!(rows[0].watched, 3);
    assert_eq!(rows[0].latest_episode, 4);
    assert_eq!(rows[0].cover, "https://i0.hdslb.com/bfs/bangumi/cover.jpg");
}

#[test]
fn radar_defaults_and_fresh_window() {
    let list = creators("").expect("defaults");
    assert_eq!(list.len(), 4);
    assert!(creators("nope").is_err());
    let published = Utc.with_ymd_and_hms(2026, 11, 12, 0, 0, 0).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 11, 12, 20, 0, 0).unwrap();
    assert!(is_fresh(published, now));
    let later = Utc.with_ymd_and_hms(2026, 11, 14, 0, 0, 0).unwrap();
    assert!(!is_fresh(published, later));
}
