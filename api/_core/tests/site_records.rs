use chrono::{TimeZone, Utc};
use taozhiyy_api::site_records::{
    blog_post_path, insert_about, insert_bili, insert_project, journal_article, journal_markdown,
    journal_slug, remove_about, remove_bili, remove_gallery_image, remove_journal, remove_moment,
    remove_project, upsert_journal, AboutDraft, BiliDraft, ProjectDraft,
};

const ABOUT: &str = "export const aboutEntries = [\n];\n";
const BILI: &str =
    "export const biliOwner = {\n  name: \"Spume\",\n};\n\nexport const ownerVideos = [\n];\n";
const PROJECTS: &str = "export const projects = [\n];\n\nexport function filterProjects() {}\n";
const GALLERY: &str = "export const galleryAlbums = [\n  {\n    id: \"trip\",\n    title: \"旅途\",\n    cover: \"https://fs.964643.xyz/a.jpg\",\n    images: [\n      \"https://fs.964643.xyz/a.jpg\",\n      \"https://fs.964643.xyz/b.jpg\",\n    ],\n  },\n];\n";
const MOMENTS: &str = "export const moments = [\n  {\n    year: \"2026\",\n    date: \"6.8\",\n    type: \"碎碎念\",\n    tone: \"aurora\",\n    module: \"postcard\",\n    lines: [\"今天的云\"],\n  },\n];\n";

#[test]
fn blog_path_stays_inside_posts() {
    assert_eq!(
        blog_post_path("blog/source/_posts/hello.md").expect("ok"),
        "blog/source/_posts/hello.md"
    );
    assert!(blog_post_path("blog/source/_posts/../secret.md").is_err());
    assert!(blog_post_path("main/src/data/moments.js").is_err());
}

#[test]
fn journal_round_trip_adds_cover_html_and_removes_by_slug() {
    let now = Utc.with_ymd_and_hms(2026, 11, 16, 4, 0, 0).unwrap();
    let slug = journal_slug("清沫", now);
    assert!(slug.starts_with("journal-"));
    let article = journal_article(
        &slug,
        "清沫",
        "第一段\n\n第二段",
        "https://fs.964643.xyz/cover.jpg",
        now,
    )
    .expect("article");
    let markdown = journal_markdown(&article);
    assert!(markdown.contains("title_zh: \"清沫\""));
    assert!(markdown.contains("---en---"));
    let saved = upsert_journal("{\n  \"articles\": []\n}\n", &article).expect("save");
    assert!(saved.contains(&slug));
    assert!(saved.contains("fs.964643.xyz/cover.jpg"));
    assert!(!saved.contains("\"body\""));
    let removed = remove_journal(&saved, &slug).expect("remove");
    assert!(!removed.contains(&slug));
    assert!(remove_journal(&removed, &slug).is_err());
}

#[test]
fn about_bili_and_project_can_be_added_and_removed() {
    let about = insert_about(
        ABOUT,
        AboutDraft {
            id: "ab12",
            title: "近况",
            text: "在写站点",
            image: "https://fs.964643.xyz/me.jpg",
        },
    )
    .expect("about");
    assert!(about.contains("近况"));
    let cleared = remove_about(&about, "ab12").expect("remove about");
    assert!(!cleared.contains("近况"));

    let (bili, changed) = insert_bili(
        BILI,
        BiliDraft {
            bvid: "BV1xx411c7mD",
            title: "演示",
            description: "一条视频",
            cover: "https://fs.964643.xyz/v.jpg",
            duration: "01:02",
            published_at: "2026-11-16",
        },
    )
    .expect("bili");
    assert!(changed);
    assert!(bili.contains("biliOwner"));
    let (again, changed) = insert_bili(
        &bili,
        BiliDraft {
            bvid: "BV1xx411c7mD",
            title: "演示",
            description: "",
            cover: "",
            duration: "",
            published_at: "",
        },
    )
    .expect("dup");
    assert!(!changed);
    assert_eq!(again, bili);
    let cleared = remove_bili(&bili, "BV1xx411c7mD").expect("remove bili");
    assert!(!cleared.contains("BV1xx411c7mD"));

    let project = insert_project(
        PROJECTS,
        ProjectDraft {
            id: "star01",
            title: "星图",
            summary: "一颗新的星",
            cover: "https://fs.964643.xyz/star.jpg",
            url: "https://www.miku.coffee/",
            tags: "星空, 记录",
            reviewed_at: "2026-11-16",
        },
    )
    .expect("project");
    assert!(project.contains("filterProjects"));
    assert!(project.contains("星图"));
    let cleared = remove_project(&project, "star01").expect("remove project");
    assert!(!cleared.contains("星图"));
    assert!(cleared.contains("filterProjects"));
}

#[test]
fn gallery_image_and_moment_can_be_removed() {
    let left = remove_gallery_image(GALLERY, "trip", "https://fs.964643.xyz/a.jpg").expect("image");
    assert!(!left.contains("https://fs.964643.xyz/a.jpg"));
    assert!(left.contains("https://fs.964643.xyz/b.jpg"));
    assert!(left.contains("cover: \"https://fs.964643.xyz/b.jpg\""));
    let gone = remove_gallery_image(&left, "trip", "https://fs.964643.xyz/b.jpg").expect("last");
    assert!(!gone.contains("trip"));

    let moments = remove_moment(MOMENTS, "2026", "6.8", "碎碎念", "今天的云").expect("moment");
    assert!(!moments.contains("今天的云"));
    assert!(remove_moment(&moments, "2026", "6.8", "碎碎念", "今天的云").is_err());
}
