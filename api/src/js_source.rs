//! 站长发布时改写 main 里的 JS 数据文件。
//!
//! 括号匹配按字节扫描，和 Go 版一致。字符串里的中文不会被当成括号。
//! 返回值里的 `&str` 借用调用方传入的草稿，不能比草稿活得更久。

use thiserror::Error;

/// 友链草稿。字段借用请求体，避免先拷贝再校验。
#[derive(Clone, Copy)]
pub struct FriendInput<'a> {
    /// 站点名称。
    pub name: &'a str,
    /// 站点描述。
    pub desc: &'a str,
    /// 公开的 http 或 https 地址。
    pub url: &'a str,
    /// 头像地址。空字符串时用站点域名生成 favicon。
    pub avatar: &'a str,
}

/// 碎语草稿。年份和日期留空时用 `now` 的东八区时间补上。
#[derive(Clone, Copy)]
pub struct MomentInput<'a> {
    /// 四位年份。空则取东八区当前年。
    pub year: &'a str,
    /// `6.8` 这种月.日。空则取东八区当天。
    pub date: &'a str,
    /// 分类。空则改用 `category`。
    pub kind: &'a str,
    /// 备用分类字段。
    pub category: &'a str,
    /// 正文，按行拆开。
    pub content: &'a str,
}

/// 相册图片草稿。
#[derive(Clone, Copy)]
pub struct GalleryInput<'a> {
    /// 相册 id。空则从标题生成。
    pub album_id: &'a str,
    /// 相册标题。
    pub album_title: &'a str,
    /// 已经上传好的公开图片地址。
    pub image_url: &'a str,
}

/// 改写后的友链文件。`name` 和 `url` 指向原始草稿。
pub struct FriendUpdate<'a> {
    /// 写回仓库的完整文件。
    pub source: String,
    /// 去掉首尾空白后的名称。
    pub name: &'a str,
    /// 去掉首尾空白后的地址。
    pub url: &'a str,
    /// 地址已经存在时为 false，调用方不应再提交。
    pub changed: bool,
}

/// 改写后的碎语文件。年份、日期和分类是补全后的自有字符串。
pub struct MomentUpdate {
    /// 写回仓库的完整文件。
    pub source: String,
    /// 实际写入的年份。
    pub year: String,
    /// 实际写入的月.日。
    pub date: String,
    /// 实际写入的分类。
    pub kind: String,
    /// 碎语每次都是新增，所以成功时总是 true。
    pub changed: bool,
}

/// 改写后的相册文件。
pub struct GalleryUpdate {
    /// 写回仓库的完整文件。
    pub source: String,
    /// 实际使用的相册 id。
    pub album_id: String,
    /// 图片地址。
    pub image_url: String,
    /// 同一相册里已经有这张图时为 false。
    pub changed: bool,
}

/// 数据文件不符合预期，或者草稿本身不能发布。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SourceError {
    /// 草稿或源文件无法安全改写。
    #[error("{0}")]
    Invalid(&'static str),
}

const TONES: [&str; 6] = ["aurora", "ticket", "watercolor", "mist", "journal", "mint"];
const MODULES: [&str; 6] = [
    "postcard",
    "ticket",
    "watercolor",
    "poem",
    "journal",
    "ribbon",
];

/// 在 `friendCards` 数组结束前插入一张卡片。相同 URL 不重复插入。
pub fn update_friend_cards<'a>(
    source: &'a str,
    input: FriendInput<'a>,
) -> Result<FriendUpdate<'a>, SourceError> {
    let name = input.name.trim();
    let desc = input.desc.trim();
    let url = input.url.trim();
    let avatar = input.avatar.trim();
    if name.is_empty() {
        return Err(SourceError::Invalid("友链站点名称不能为空"));
    }
    if desc.is_empty() {
        return Err(SourceError::Invalid("友链站点描述不能为空"));
    }
    if !public_http_url(url) {
        return Err(SourceError::Invalid(
            "友链 URL 必须是公开的 http 或 https 地址",
        ));
    }
    let generated;
    let avatar = if avatar.is_empty() {
        generated = default_avatar(url)?;
        generated.as_str()
    } else if public_http_url(avatar) {
        avatar
    } else {
        return Err(SourceError::Invalid(
            "友链头像 URL 必须是公开的 http 或 https 地址",
        ));
    };
    if source.contains(&js_string(url)) {
        return Ok(FriendUpdate {
            source: source.to_string(),
            name,
            url,
            changed: false,
        });
    }
    let close = array_close(source, "export const friendCards = [")?;
    let card = friend_card(name, desc, url, avatar);
    Ok(FriendUpdate {
        source: insert_before_line(source, close, &card)?,
        name,
        url,
        changed: true,
    })
}

/// 按时间倒序把一条碎语插进 `moments`。`now` 只在年份或日期为空时使用，时区按东八区。
pub fn update_moments(
    source: &str,
    input: MomentInput<'_>,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<MomentUpdate, SourceError> {
    let site = now + chrono::Duration::hours(8);
    let year = filled_year(input.year, site)?;
    let date = filled_date(input.date, site)?;
    let kind = input.kind.trim();
    let kind = if kind.is_empty() {
        input.category.trim()
    } else {
        kind
    };
    if kind.is_empty() {
        return Err(SourceError::Invalid("碎语分类不能为空"));
    }
    let lines = content_lines(input.content);
    if lines.is_empty() {
        return Err(SourceError::Invalid("碎语内容不能为空"));
    }
    let (open, close) = array_bounds(source, "export const moments = [")?;
    let inside = source
        .get(open..close)
        .ok_or(SourceError::Invalid("moments 数组无效"))?;
    let count = inside.matches("date:").count();
    let card = moment_card(
        &year,
        &date,
        kind,
        TONES[count % TONES.len()],
        MODULES[count % MODULES.len()],
        &lines,
    );
    let at = moment_insert_at(source, open, close, &year, &date)?;
    Ok(MomentUpdate {
        source: insert_at(source, at, &card)?,
        year,
        date,
        kind: kind.to_string(),
        changed: true,
    })
}

/// 往已有相册追加图片；相册不存在时在 `galleryAlbums` 末尾新建。
pub fn update_gallery(source: &str, input: GalleryInput<'_>) -> Result<GalleryUpdate, SourceError> {
    let image_url = input.image_url.trim();
    if !public_http_url(image_url) {
        return Err(SourceError::Invalid(
            "相册图片 URL 必须是公开的 http 或 https 地址",
        ));
    }
    let album_id = input.album_id.trim();
    let album_title = input.album_title.trim();
    let album_id = if album_id.is_empty() {
        album_slug(album_title)
    } else {
        album_id.to_string()
    };
    let album_title = if album_title.is_empty() {
        album_id.clone()
    } else {
        album_title.to_string()
    };
    if album_id.is_empty() || album_id == "default" {
        return Err(SourceError::Invalid("请选择相册或填写自定义相册名"));
    }
    if let Some(update) = append_existing_album(source, &album_id, image_url)? {
        return Ok(update);
    }
    let close = array_close(source, "export const galleryAlbums = [")?;
    let album = new_album(&album_id, &album_title, image_url);
    Ok(GalleryUpdate {
        source: insert_before_line(source, close, &album)?,
        album_id,
        image_url: image_url.to_string(),
        changed: true,
    })
}

fn append_existing_album(
    source: &str,
    album_id: &str,
    image_url: &str,
) -> Result<Option<GalleryUpdate>, SourceError> {
    let Some((start, end)) = find_album(source, album_id)? else {
        return Ok(None);
    };
    let object = source
        .get(start..end)
        .ok_or(SourceError::Invalid("相册对象无效"))?;
    if object.contains(&js_string(image_url)) {
        return Ok(Some(GalleryUpdate {
            source: source.to_string(),
            album_id: album_id.to_string(),
            image_url: image_url.to_string(),
            changed: false,
        }));
    }
    let images_rel = object
        .find("images: [")
        .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let open = start
        + images_rel
        + object[images_rel..]
            .find('[')
            .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let close = find_matching(source, open, b'[', b']')?;
    if close > end {
        return Err(SourceError::Invalid("相册的 images 数组无效"));
    }
    let entry = format!(
        "{}{},\n",
        entry_indent(source, open, close),
        js_string(image_url)
    );
    Ok(Some(GalleryUpdate {
        source: insert_before_line(source, close, &entry)?,
        album_id: album_id.to_string(),
        image_url: image_url.to_string(),
        changed: true,
    }))
}

fn find_album(source: &str, album_id: &str) -> Result<Option<(usize, usize)>, SourceError> {
    let pattern = format!("id: {}", js_string(album_id));
    let Some(id_index) = source.find(&pattern) else {
        return Ok(None);
    };
    let start = source[..id_index]
        .rfind('{')
        .ok_or(SourceError::Invalid("相册对象无效"))?;
    let end = find_matching(source, start, b'{', b'}')?;
    Ok(Some((start, end + 1)))
}

fn entry_indent(source: &str, open: usize, close: usize) -> String {
    let body = source.get(open + 1..close).unwrap_or("");
    let close_indent = line_indent(source, close);
    for line in body.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('"') || trimmed.starts_with('`') {
            return line_prefix(line);
        }
    }
    format!("{close_indent}  ")
}

fn friend_card(name: &str, desc: &str, url: &str, avatar: &str) -> String {
    format!(
        "  {{\n    name: {},\n    desc: {},\n    url: {},\n    avatar: {},\n    note: \"FRIEND\",\n  }},\n",
        js_string(name),
        js_string(desc),
        js_string(url),
        js_string(avatar)
    )
}

fn moment_card(
    year: &str,
    date: &str,
    kind: &str,
    tone: &str,
    module: &str,
    lines: &[String],
) -> String {
    let quoted = lines
        .iter()
        .map(|line| js_string(line))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "  {{\n    year: {},\n    date: {},\n    type: {},\n    tone: {},\n    module: {},\n    lines: [{}],\n  }},\n",
        js_string(year),
        js_string(date),
        js_string(kind),
        js_string(tone),
        js_string(module),
        quoted
    )
}

fn new_album(album_id: &str, title: &str, image_url: &str) -> String {
    format!(
        "  {{\n    id: {},\n    title: {},\n    eyebrow: \"站长上传\",\n    description: \"由站长控制器上传。\",\n    tone: \"from-[#F6FBFF] via-[#FFF8F1] to-[#FFEAF4]\",\n    accent: \"#FF8FAB\",\n    cover: {},\n    images: [\n      {},\n    ],\n  }},\n",
        js_string(album_id),
        js_string(title),
        js_string(image_url),
        js_string(image_url)
    )
}

fn moment_insert_at(
    source: &str,
    open: usize,
    close: usize,
    year: &str,
    date: &str,
) -> Result<usize, SourceError> {
    let new_order = date_order(year, date)?;
    let mut pos = open + 1;
    while let Some(rel) = source
        .get(pos..close)
        .ok_or(SourceError::Invalid("moments 数组无效"))?
        .find("date:")
    {
        let date_index = pos + rel;
        let existing_date = quoted_after(source.get(date_index..close).unwrap_or(""));
        let existing_year = year_before(source, date_index);
        if !existing_date.is_empty() && new_order >= date_order(&existing_year, &existing_date)? {
            if let Some(object_start) = source[..date_index].rfind("\n  {") {
                return Ok(object_start + 1);
            }
        }
        pos = date_index + "date:".len();
    }
    Ok(source[..close]
        .rfind('\n')
        .ok_or(SourceError::Invalid("moments 数组无效"))?
        + 1)
}

fn year_before(source: &str, date_index: usize) -> String {
    let Some(object_start) = source[..date_index].rfind("\n  {") else {
        return String::new();
    };
    let Some(year_rel) = source[object_start..date_index].rfind("year:") else {
        return String::new();
    };
    quoted_after(&source[object_start + year_rel..date_index])
}

fn quoted_after(source: &str) -> String {
    let Some(first) = source.find('"') else {
        return String::new();
    };
    let rest = &source[first + 1..];
    let Some(second) = rest.find('"') else {
        return String::new();
    };
    rest[..second].to_string()
}

fn date_order(year: &str, date: &str) -> Result<i32, SourceError> {
    let Some((month, day)) = date.split_once('.') else {
        return Err(SourceError::Invalid("碎语日期格式应为 6.8 这样的月.日"));
    };
    let year_number = year
        .parse::<i32>()
        .map_err(|_| SourceError::Invalid("碎语年份格式应为 2026 这样的四位年份"))?;
    let month = month
        .parse::<i32>()
        .map_err(|_| SourceError::Invalid("碎语日期格式应为 6.8 这样的月.日"))?;
    let day = day
        .parse::<i32>()
        .map_err(|_| SourceError::Invalid("碎语日期格式应为 6.8 这样的月.日"))?;
    Ok(year_number * 10_000 + month * 100 + day)
}

fn filled_year(raw: &str, site: chrono::DateTime<chrono::Utc>) -> Result<String, SourceError> {
    let year = raw.trim();
    let year = if year.is_empty() {
        site.format("%Y").to_string()
    } else {
        year.to_string()
    };
    if year.len() == 4 && year.chars().all(|ch| ch.is_ascii_digit()) {
        Ok(year)
    } else {
        Err(SourceError::Invalid("碎语年份格式应为 2026 这样的四位年份"))
    }
}

fn filled_date(raw: &str, site: chrono::DateTime<chrono::Utc>) -> Result<String, SourceError> {
    let date = raw.trim();
    let date = if date.is_empty() {
        site_date(site)
    } else {
        date.to_string()
    };
    let ok = date
        .split_once('.')
        .is_some_and(|(month, day)| dotted_number(month, 2) && dotted_number(day, 2));
    if ok {
        Ok(date)
    } else {
        Err(SourceError::Invalid("碎语日期格式应为 6.8 这样的月.日"))
    }
}

fn site_date(site: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "{}.{}",
        unpadded(&site.format("%m").to_string()),
        unpadded(&site.format("%d").to_string())
    )
}

fn unpadded(value: &str) -> &str {
    let trimmed = value.trim_start_matches('0');
    if trimmed.is_empty() {
        "0"
    } else {
        trimmed
    }
}

fn dotted_number(value: &str, max: usize) -> bool {
    (1..=max).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_digit()) && value != "0"
}

fn content_lines(content: &str) -> Vec<String> {
    content
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn array_close(source: &str, marker: &str) -> Result<usize, SourceError> {
    array_bounds(source, marker).map(|(_, close)| close)
}

fn array_bounds(source: &str, marker: &str) -> Result<(usize, usize), SourceError> {
    let marker_at = source
        .find(marker)
        .ok_or(SourceError::Invalid("未找到数据导出"))?;
    let open = marker_at
        + source[marker_at..]
            .find('[')
            .ok_or(SourceError::Invalid("未找到数据导出"))?;
    let close = find_matching(source, open, b'[', b']')?;
    Ok((open, close))
}

fn find_matching(
    source: &str,
    open_index: usize,
    open: u8,
    close: u8,
) -> Result<usize, SourceError> {
    let bytes = source.as_bytes();
    if bytes.get(open_index).copied() != Some(open) {
        return Err(SourceError::Invalid("括号位置无效"));
    }
    let mut depth = 0i32;
    let mut in_string = None;
    let mut escaped = false;
    for (index, &ch) in bytes.iter().enumerate().skip(open_index) {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == b'\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                in_string = None;
            }
            continue;
        }
        if matches!(ch, b'"' | b'\'' | b'`') {
            in_string = Some(ch);
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Ok(index);
            }
        }
    }
    Err(SourceError::Invalid("括号没有闭合"))
}

fn insert_before_line(source: &str, close: usize, piece: &str) -> Result<String, SourceError> {
    let line = source[..close]
        .rfind('\n')
        .ok_or(SourceError::Invalid("数据数组无效"))?
        + 1;
    insert_at(source, line, piece)
}

fn insert_at(source: &str, at: usize, piece: &str) -> Result<String, SourceError> {
    let head = source
        .get(..at)
        .ok_or(SourceError::Invalid("数据数组无效"))?;
    let tail = source
        .get(at..)
        .ok_or(SourceError::Invalid("数据数组无效"))?;
    let mut out = String::with_capacity(source.len() + piece.len());
    out.push_str(head);
    out.push_str(piece);
    out.push_str(tail);
    Ok(out)
}

fn line_indent(source: &str, index: usize) -> String {
    let line_start = source[..index].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
    line_prefix(source.get(line_start..index).unwrap_or(""))
}

fn line_prefix(line: &str) -> String {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .collect()
}

fn js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' | '"' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn public_http_url(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.rsplit('@').next().unwrap_or("");
    !host.is_empty() && !host.starts_with(':')
}

fn default_avatar(friend_url: &str) -> Result<String, SourceError> {
    let rest = friend_url
        .strip_prefix("https://")
        .or_else(|| friend_url.strip_prefix("http://"))
        .ok_or(SourceError::Invalid(
            "友链 URL 必须是公开的 http 或 https 地址",
        ))?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or("");
    if host.is_empty() {
        return Err(SourceError::Invalid(
            "友链 URL 必须是公开的 http 或 https 地址",
        ));
    }
    Ok(format!(
        "https://www.google.com/s2/favicons?sz=128&domain={}",
        query_escape(host)
    ))
}

fn query_escape(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn album_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in title.trim().chars() {
        if ch.is_alphanumeric() {
            slug.push(if ch.is_ascii() {
                ch.to_ascii_lowercase()
            } else {
                ch
            });
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "default".to_string()
    } else {
        slug.to_string()
    }
}
