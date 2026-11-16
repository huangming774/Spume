use taozhiyy_api::wallpaper::wallpaper_from_redirect;

#[test]
fn absolute_webp_redirect_gets_a_stable_id() {
    let item = wallpaper_from_redirect(
        "https://t.alcy.cc/pc",
        "https://t.alcy.cc/wall/cover.webp?x=1",
    )
    .expect("wallpaper");
    assert!(item.id.starts_with("alcy-"));
    // 形状和 Go 的 stableWallpaperID 一致：前缀 + 16 个十六进制字符。
    assert_eq!(item.id.len(), "alcy-".len() + 16);
    assert!(item.id["alcy-".len()..]
        .chars()
        .all(|ch| ch.is_ascii_hexdigit()));
    assert!(item.url.ends_with(".webp?x=1") || item.url.contains(".webp?"));
    let again = wallpaper_from_redirect("https://t.alcy.cc/pc", &item.url).expect("again");
    assert_eq!(item.id, again.id);
}

#[test]
fn relative_redirect_uses_the_endpoint_origin() {
    let item =
        wallpaper_from_redirect("https://t.alcy.cc/pc", "/wall/cover.webp").expect("relative");
    assert_eq!(item.url, "https://t.alcy.cc/wall/cover.webp");
}

#[test]
fn non_webp_redirect_is_rejected() {
    assert!(
        wallpaper_from_redirect("https://t.alcy.cc/pc", "https://t.alcy.cc/wall/cover.jpg")
            .is_err()
    );
}
