//! 把生图服务返回的临时地址下载下来，再写入 R2。

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::r2::{presign_put, public_url};
use crate::AppState;

/// 已经放进 R2 的图片。
pub struct StoredImage {
    /// 公开地址。
    pub url: String,
    /// 对象键。
    pub key: String,
    /// 内容类型。
    pub mime: String,
    /// 字节数。
    pub size: usize,
}

/// 转存失败。
#[derive(Debug, Error)]
pub enum ImageStoreError {
    /// R2 环境变量不完整。
    #[error("R2 还没有配置，不能保存生成图")]
    NotConfigured,
    /// 临时地址不是 https。
    #[error("生图地址不是 https")]
    BadSource,
    /// 下载失败或文件不是图片。
    #[error("生图完成了，但临时图片下载失败，请稍后再试。")]
    Download,
    /// 上传失败。
    #[error("生成图保存到 R2 失败")]
    Upload,
}

/// 下载 `source` 并上传到 `ai-images/年/月/`。
pub async fn store_remote_image(
    state: &AppState,
    source: &str,
) -> Result<StoredImage, ImageStoreError> {
    if !state.cfg.r2_ready() {
        return Err(ImageStoreError::NotConfigured);
    }
    if !source.starts_with("https://") {
        return Err(ImageStoreError::BadSource);
    }
    let response = state
        .http
        .get(source)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|_| ImageStoreError::Download)?;
    if !response.status().is_success() {
        return Err(ImageStoreError::Download);
    }
    let header_mime = match response.headers().get(reqwest::header::CONTENT_TYPE) {
        None => String::new(),
        Some(value) => {
            let text = value.to_str().map_err(|_| ImageStoreError::Download)?;
            text.split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
    };
    let bytes = response
        .bytes()
        .await
        .map_err(|_| ImageStoreError::Download)?;
    if bytes.is_empty() || bytes.len() > 12 * 1024 * 1024 {
        return Err(ImageStoreError::Download);
    }
    // 和 Go 的 http.DetectContentType 兜底一致：头缺失或不是图片时按魔数嗅探。
    let mime = if header_mime.starts_with("image/") {
        header_mime
    } else {
        sniff_image_mime(&bytes)
            .ok_or(ImageStoreError::Download)?
            .to_string()
    };
    upload_image(state, &bytes, &mime).await
}

/// 把上游直接返回的图片字节写入 R2（Cloudflare Workers AI 这类不出临时地址的上游）。
pub async fn store_image_bytes(
    state: &AppState,
    bytes: &[u8],
) -> Result<StoredImage, ImageStoreError> {
    if !state.cfg.r2_ready() {
        return Err(ImageStoreError::NotConfigured);
    }
    if bytes.is_empty() || bytes.len() > 12 * 1024 * 1024 {
        return Err(ImageStoreError::Download);
    }
    let mime = sniff_image_mime(bytes)
        .ok_or(ImageStoreError::Download)?
        .to_string();
    upload_image(state, bytes, &mime).await
}

/// 按内容哈希命名写进 `ai-images/年/月/`。
async fn upload_image(
    state: &AppState,
    bytes: &[u8],
    mime: &str,
) -> Result<StoredImage, ImageStoreError> {
    let digest = Sha256::digest(bytes);
    let ext = match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        _ => "png",
    };
    let key = format!(
        "ai-images/{}/{}.{}",
        chrono::Utc::now().format("%Y/%m"),
        hex::encode(&digest[..8]),
        ext
    );
    // R2 的预签名只认 UNSIGNED-PAYLOAD：签名里放正文哈希会让服务端验签对不上（403）。
    let target = presign_put(&state.cfg, &key, 300);
    let sent = state
        .http
        .put(target)
        .header(reqwest::header::CONTENT_TYPE, mime)
        .body(bytes.to_vec())
        .send()
        .await
        .map_err(|error| {
            tracing::error!(%error, "r2 upload transport failed");
            ImageStoreError::Upload
        })?;
    if !sent.status().is_success() {
        let status = sent.status();
        let detail = sent.text().await.unwrap_or_default();
        tracing::error!(%status, detail = %detail.chars().take(200).collect::<String>(), "r2 upload rejected");
        return Err(ImageStoreError::Upload);
    }
    Ok(StoredImage {
        url: public_url(&state.cfg, &key),
        key,
        mime: mime.to_string(),
        size: bytes.len(),
    })
}

/// Go `http.DetectContentType` 的图片子集：按魔数猜 MIME。
fn sniff_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Some("image/png");
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if bytes.starts_with(b"BM") {
        return Some("image/bmp");
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
    {
        return Some("image/avif");
    }
    None
}
