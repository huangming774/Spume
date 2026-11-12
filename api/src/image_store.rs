//! 把生图服务返回的临时地址下载下来，再写入 R2。

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::r2::{public_url, signed_put_url};
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
    let mime = match response.headers().get(reqwest::header::CONTENT_TYPE) {
        None => "image/png".to_string(),
        Some(value) => {
            let text = value.to_str().map_err(|_| ImageStoreError::Download)?;
            text.split(';')
                .next()
                .unwrap_or("image/png")
                .trim()
                .to_string()
        }
    };
    if !mime.starts_with("image/") {
        return Err(ImageStoreError::Download);
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| ImageStoreError::Download)?;
    if bytes.is_empty() || bytes.len() > 12 * 1024 * 1024 {
        return Err(ImageStoreError::Download);
    }
    let digest = Sha256::digest(&bytes);
    let ext = match mime.as_str() {
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
    let hash = hex::encode(digest);
    let target = signed_put_url(&state.cfg, &key, &hash, 300);
    let sent = state
        .http
        .put(target)
        .header(reqwest::header::CONTENT_TYPE, &mime)
        .body(bytes.to_vec())
        .send()
        .await
        .map_err(|_| ImageStoreError::Upload)?;
    if !sent.status().is_success() {
        return Err(ImageStoreError::Upload);
    }
    Ok(StoredImage {
        url: public_url(&state.cfg, &key),
        key,
        mime,
        size: bytes.len(),
    })
}
