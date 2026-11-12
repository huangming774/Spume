use taozhiyy_api::jobs::documents;

#[test]
fn markdown_becomes_titled_chunks() {
    let markdown = "---\ntitle: Spume\n---\n\n第一段。第二段。";
    let docs = documents("blog", "hello", "/blog/hello", markdown);
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].id, "blog:hello:0");
    assert_eq!(docs[0].title, "Spume");
    assert_eq!(docs[0].url, "/blog/hello");
    assert!(docs[0].data.contains("第一段"));
}

#[test]
fn long_text_is_split_every_seven_hundred_characters() {
    let body = "字".repeat(1400);
    let docs = documents("journal", "log", "/build/post/log", &body);
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[1].id, "journal:log:1");
}
