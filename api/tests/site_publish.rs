use chrono::{TimeZone, Utc};
use taozhiyy_api::js_source::{
    update_friend_cards, update_gallery, update_moments, FriendInput, GalleryInput, MomentInput,
};

const FRIENDS: &str = "export const friendCards = [\n];\n";
const MOMENTS: &str = "export const moments = [\n  {\n    year: \"2026\",\n    date: \"6.9\",\n    type: \"记录\",\n    tone: \"aurora\",\n    module: \"postcard\",\n    lines: [\"旧\"],\n  },\n];\n";
const GALLERY: &str = "export const galleryAlbums = [\n  {\n    id: \"misaka\",\n    title: \"御坂\",\n    images: [\n      \"https://images.example/old.jpg\",\n    ],\n  },\n];\n";

#[test]
fn friend_card_is_inserted_once() {
    let input = FriendInput {
        name: "Spume",
        desc: "个人站",
        url: "https://taozhiyy.top",
        avatar: "",
    };
    let first = update_friend_cards(FRIENDS, input).expect("insert");
    assert!(first.changed);
    assert!(first.source.contains("name: \"Spume\""));
    assert!(first
        .source
        .contains("https://www.google.com/s2/favicons?sz=128&domain=taozhiyy.top"));
    let again = update_friend_cards(&first.source, input).expect("duplicate");
    assert!(!again.changed);
    assert_eq!(again.source, first.source);
}

#[test]
fn newer_moment_is_inserted_before_older_one() {
    let now = Utc.with_ymd_and_hms(2026, 6, 13, 2, 0, 0).unwrap();
    let newer = update_moments(
        MOMENTS,
        MomentInput {
            year: "2026",
            date: "6.13",
            kind: "校园",
            category: "",
            content: "彩虹",
        },
        now,
    )
    .expect("newer");
    let older_at = newer.source.find("date: \"6.9\"").expect("old date");
    let newer_at = newer.source.find("date: \"6.13\"").expect("new date");
    assert!(newer_at < older_at);
    let older = update_moments(
        &newer.source,
        MomentInput {
            year: "",
            date: "",
            kind: "",
            category: "实验",
            content: "补上默认日期",
        },
        now,
    )
    .expect("default");
    assert_eq!(older.year, "2026");
    assert_eq!(older.date, "6.13");
}

#[test]
fn gallery_appends_to_an_existing_album_and_creates_a_missing_one() {
    let added = update_gallery(
        GALLERY,
        GalleryInput {
            album_id: "misaka",
            album_title: "",
            image_url: "https://images.example/new.jpg",
        },
    )
    .expect("append");
    assert!(added.changed);
    assert!(added.source.contains("\"https://images.example/new.jpg\""));
    let same = update_gallery(
        &added.source,
        GalleryInput {
            album_id: "misaka",
            album_title: "",
            image_url: "https://images.example/new.jpg",
        },
    )
    .expect("same");
    assert!(!same.changed);
    let created = update_gallery(
        GALLERY,
        GalleryInput {
            album_id: "",
            album_title: "新相册",
            image_url: "https://images.example/cover.jpg",
        },
    )
    .expect("create");
    assert!(created.source.contains("id: \"新相册\""));
}

#[test]
fn javascript_url_is_rejected() {
    let error = update_friend_cards(
        FRIENDS,
        FriendInput {
            name: "x",
            desc: "y",
            url: "javascript:alert(1)",
            avatar: "",
        },
    );
    assert!(error.is_err());
}
