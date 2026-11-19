//! 生图上游。
//!
//! 配了 `CLOUDFLARE_AI_TOKEN` 时优先用 Cloudflare Workers AI。
//! 其次是 Agnes（`AGNES_API_KEY`），最后是百炼的通义万相多模态接口。
//! 三个都没配时，接口告诉前端生图还不能用。

use base64::Engine;
use serde_json::{json, Value};
use thiserror::Error;

use crate::config::Config;

const AGNES_FAILED: &str = "Agnes 生图暂时失败了，请稍后再试。";
pub const PROMPT_REJECTED_MESSAGE: &str = "提示词被 Agnes 拒绝了，可能包含不适合的内容或过于接近敏感描述。请修改为更安全、更中性的说法后重试。";

/// 上游返回的一张图。
pub struct GeneratedImage {
    /// 临时下载地址。上游直接给图片字节时为空。
    pub url: String,
    /// 上游直接返回的图片字节。有值时不用再下载。
    pub bytes: Option<Vec<u8>>,
    /// 实际调用的模型名。
    pub model: String,
    /// 归一化后的尺寸，形如 `1024*1024`。
    pub size: String,
    /// 上游请求号。没有时为空。
    pub request_id: String,
    /// 像素宽。上游没给时为 0。
    pub width: i32,
    /// 像素高。上游没给时为 0。
    pub height: i32,
}

/// 生图没有得到可用图片。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImageGenError {
    /// 两个服务的密钥都是空的。
    #[error("生图还没有配置")]
    NotConfigured,
    /// Agnes 以内容策略拒绝了提示词。
    #[error("{PROMPT_REJECTED_MESSAGE}")]
    PromptRejected,
    /// 上游没有返回图片。具体原因只写进日志。
    #[error("{AGNES_FAILED}")]
    Provider,
}

/// 只接受 Go 版开放的三种尺寸，其它值回到方图。
pub fn normalize_size(size: &str) -> &'static str {
    match size.trim() {
        "1024*1024" => "1024*1024",
        "1280*720" => "1280*720",
        "720*1280" => "720*1280",
        _ => "1024*1024",
    }
}

/// Go 的提示词归一化：去掉首尾空白，中间连续空白压成一个空格。
pub fn normalize_prompt(prompt: &str) -> String {
    prompt.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 至少一个生图上游已经配置。
pub fn configured(cfg: &Config) -> bool {
    cloudflare_ready(cfg) || !cfg.agnes_api_key.is_empty() || !cfg.dashscope_api_key.is_empty()
}

/// Cloudflare Workers AI 是否可用。
fn cloudflare_ready(cfg: &Config) -> bool {
    !cfg.cloudflare_ai_token.is_empty() && !cfg.cloudflare_account_id.is_empty()
}

/// 配额接口展示的模型名。优先 Cloudflare，其次 Agnes，最后百炼。
pub fn public_model(cfg: &Config) -> &str {
    if cloudflare_ready(cfg) {
        &cfg.cloudflare_ai_model
    } else if !cfg.agnes_api_key.is_empty() || cfg.dashscope_api_key.is_empty() {
        &cfg.agnes_image_model
    } else {
        &cfg.dashscope_image_model
    }
}

/// Agnes 的图片生成地址。基址已经带 `/v1` 时不再重复添加。
pub fn agnes_endpoint(base: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    if base.ends_with("/v1") {
        format!("{base}/images/generations")
    } else {
        format!("{base}/v1/images/generations")
    }
}

/// 百炼多模态生图地址。
pub fn dashscope_endpoint(base: &str) -> String {
    format!(
        "{}/services/aigc/multimodal-generation/generation",
        base.trim().trim_end_matches('/')
    )
}

/// 从 Agnes 的 `data[].url` 取出第一张图。
pub fn agnes_image_url(payload: &Value) -> Option<String> {
    payload
        .get("data")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter().find_map(|row| {
                row.get("url")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|url| !url.is_empty())
                    .map(str::to_string)
            })
        })
}

/// 百炼可能把图放在消息内容里，也可能放在旧的 `results` 数组里。
pub fn dashscope_image(payload: &Value) -> Option<GeneratedImage> {
    let url = choice_image(payload).or_else(|| result_image(payload))?;
    let width = payload
        .pointer("/usage/width")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let height = payload
        .pointer("/usage/height")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    Some(GeneratedImage {
        url,
        bytes: None,
        model: String::new(),
        size: String::new(),
        request_id: payload
            .get("request_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string(),
        width: i32::try_from(width).unwrap_or(0),
        height: i32::try_from(height).unwrap_or(0),
    })
}

/// 调用已配置的上游。Cloudflare 优先于 Agnes，Agnes 优先于百炼。
/// 某个上游临时失败（Provider）时自动降级到下一个已配置上游；
/// 提示词被内容策略拒绝（PromptRejected）不降级，直接返回让用户改词。
pub async fn generate(
    http: &reqwest::Client,
    cfg: &Config,
    prompt: &str,
    size: &str,
) -> Result<GeneratedImage, ImageGenError> {
    let size = normalize_size(size);
    let mut candidates: Vec<&str> = Vec::new();
    if cloudflare_ready(cfg) {
        candidates.push("cloudflare");
    }
    if !cfg.agnes_api_key.is_empty() {
        candidates.push("agnes");
    }
    if !cfg.dashscope_api_key.is_empty() {
        candidates.push("dashscope");
    }
    if candidates.is_empty() {
        return Err(ImageGenError::NotConfigured);
    }
    let mut last_err = ImageGenError::Provider;
    for provider in candidates {
        let result = match provider {
            "cloudflare" => request_cloudflare(http, cfg, prompt, size).await,
            "agnes" => request_agnes(http, cfg, prompt, size).await,
            _ => request_dashscope(http, cfg, prompt, size).await,
        };
        match result {
            Ok(image) => return Ok(image),
            Err(ImageGenError::PromptRejected) => return Err(ImageGenError::PromptRejected),
            Err(error) => {
                tracing::warn!(provider, %error, "image provider failed; trying next configured provider");
                last_err = error;
            }
        }
    }
    Err(last_err)
}

async fn request_agnes(
    http: &reqwest::Client,
    cfg: &Config,
    prompt: &str,
    size: &str,
) -> Result<GeneratedImage, ImageGenError> {
    let agnes_size = size.replace('*', "x");
    let payload = send(
        http,
        &agnes_endpoint(&cfg.agnes_base_url),
        &cfg.agnes_api_key,
        json!({
            "model": cfg.agnes_image_model,
            "prompt": prompt,
            "size": agnes_size,
            "extra_body": {"response_format": "url"}
        }),
    )
    .await?;
    if payload.status != 200 && prompt_rejected(payload.status, &payload.body) {
        return Err(ImageGenError::PromptRejected);
    }
    if !(200..300).contains(&payload.status) {
        log_provider("agnes", payload.status, &payload.body);
        return Err(ImageGenError::Provider);
    }
    let url = agnes_image_url(&payload.body).ok_or_else(|| {
        log_provider("agnes", payload.status, &payload.body);
        ImageGenError::Provider
    })?;
    Ok(GeneratedImage {
        url,
        bytes: None,
        model: cfg.agnes_image_model.clone(),
        size: size.to_string(),
        request_id: String::new(),
        width: 0,
        height: 0,
    })
}

async fn request_dashscope(
    http: &reqwest::Client,
    cfg: &Config,
    prompt: &str,
    size: &str,
) -> Result<GeneratedImage, ImageGenError> {
    let payload = send(
        http,
        &dashscope_endpoint(&cfg.dashscope_base_url),
        &cfg.dashscope_api_key,
        json!({
            "model": cfg.dashscope_image_model,
            "input": {
                "messages": [{
                    "role": "user",
                    "content": [{"text": prompt}]
                }]
            },
            "parameters": {
                "size": size,
                "n": 1,
                "prompt_extend": false,
                "watermark": false
            }
        }),
    )
    .await?;
    if !(200..300).contains(&payload.status) {
        log_provider("dashscope", payload.status, &payload.body);
        return Err(ImageGenError::Provider);
    }
    let mut image = dashscope_image(&payload.body).ok_or_else(|| {
        log_provider("dashscope", payload.status, &payload.body);
        ImageGenError::Provider
    })?;
    image.model = cfg.dashscope_image_model.clone();
    image.size = size.to_string();
    Ok(image)
}

/// Cloudflare Workers AI 的生图地址。
pub fn cloudflare_endpoint(account: &str, model: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
        account.trim(),
        model.trim().trim_start_matches('/')
    )
}

/// `1024*1024` 这种尺寸拆成宽高。坏值回到 1024。
pub fn size_pair(size: &str) -> (i32, i32) {
    let (width, height) = size.split_once('*').unwrap_or(("1024", "1024"));
    (
        width.trim().parse().unwrap_or(1024),
        height.trim().parse().unwrap_or(1024),
    )
}

/// Workers AI 的图有两种返回：`image/*` 响应体直接是图片字节（SDXL 等），
/// 或 JSON 里 `result/image` 放 base64（flux 等）。两种都收。
pub fn cloudflare_image(content_type: &str, body: &[u8]) -> Option<Vec<u8>> {
    let content_type = content_type.split(';').next().unwrap_or("").trim();
    if content_type.starts_with("image/") && !body.is_empty() {
        return Some(body.to_vec());
    }
    if content_type.contains("json") {
        let payload: Value = serde_json::from_slice(body).ok()?;
        let encoded = payload.pointer("/result/image").and_then(Value::as_str)?;
        return base64::engine::general_purpose::STANDARD
            .decode(encoded.trim())
            .ok()
            .filter(|raw| !raw.is_empty());
    }
    None
}

/// 宽高被模型 schema 拒绝（如 flux 只收 prompt）时，退回只传 prompt 重试。
fn schema_rejected(payload: &Value) -> bool {
    payload
        .get("errors")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter().any(|row| {
                row.get("message")
                    .and_then(Value::as_str)
                    .is_some_and(|message| message.contains("not allowed"))
            })
        })
        .unwrap_or(false)
}

async fn request_cloudflare(
    http: &reqwest::Client,
    cfg: &Config,
    prompt: &str,
    size: &str,
) -> Result<GeneratedImage, ImageGenError> {
    let (width, height) = size_pair(size);
    let url = cloudflare_endpoint(&cfg.cloudflare_account_id, &cfg.cloudflare_ai_model);
    let mut raw = send_raw(
        http,
        &url,
        &cfg.cloudflare_ai_token,
        &json!({"prompt": prompt, "width": width, "height": height}),
    )
    .await?;
    if schema_rejected(&raw.body) {
        raw = send_raw(
            http,
            &url,
            &cfg.cloudflare_ai_token,
            &json!({"prompt": prompt}),
        )
        .await?;
    }
    if !(200..300).contains(&raw.status) {
        log_provider("cloudflare", raw.status, &raw.body);
        return Err(ImageGenError::Provider);
    }
    let bytes = cloudflare_image(&raw.content_type, &raw.bytes).ok_or_else(|| {
        log_provider("cloudflare", raw.status, &raw.body);
        ImageGenError::Provider
    })?;
    Ok(GeneratedImage {
        url: String::new(),
        bytes: Some(bytes),
        model: cfg.cloudflare_ai_model.clone(),
        size: size.to_string(),
        request_id: String::new(),
        width,
        height,
    })
}

struct UpstreamBody {
    status: u16,
    body: Value,
}

async fn send(
    http: &reqwest::Client,
    url: &str,
    key: &str,
    body: Value,
) -> Result<UpstreamBody, ImageGenError> {
    let response = http
        .post(url)
        .timeout(std::time::Duration::from_secs(90))
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|error| {
            tracing::error!(%error, "image provider request failed");
            ImageGenError::Provider
        })?;
    let status = response.status().as_u16();
    let body = response.json::<Value>().await.unwrap_or(Value::Null);
    Ok(UpstreamBody { status, body })
}

/// Cloudflare 的响应可能是图片字节也可能是 JSON，不能复用 JSON 版 `send`。
struct UpstreamRaw {
    status: u16,
    content_type: String,
    bytes: Vec<u8>,
    body: Value,
}

async fn send_raw(
    http: &reqwest::Client,
    url: &str,
    key: &str,
    body: &Value,
) -> Result<UpstreamRaw, ImageGenError> {
    let response = http
        .post(url)
        .timeout(std::time::Duration::from_secs(120))
        .bearer_auth(key)
        .json(body)
        .send()
        .await
        .map_err(|error| {
            tracing::error!(%error, "image provider request failed");
            ImageGenError::Provider
        })?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map(|item| item.to_vec())
        .unwrap_or_default();
    let parsed = if content_type.contains("json") {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    Ok(UpstreamRaw {
        status,
        content_type,
        bytes,
        body: parsed,
    })
}

fn prompt_rejected(status: u16, payload: &Value) -> bool {
    if status != 400 {
        return false;
    }
    let (code, message) = error_summary(payload);
    let code = code.to_ascii_lowercase();
    let message = message.to_ascii_lowercase();
    code.contains("policy") || code.contains("safety") || message.contains("content policy")
}

fn error_summary(payload: &Value) -> (String, String) {
    let error = payload.get("error");
    let code = error
        .and_then(|value| json_text(value, &["code", "type", "error_code"]))
        .or_else(|| json_text(payload, &["code", "type", "error_code"]))
        .unwrap_or_default();
    let message = error
        .and_then(|value| match value {
            Value::String(text) => nonempty(text),
            _ => json_text(value, &["message", "msg", "detail", "error_description"]),
        })
        .or_else(|| json_text(payload, &["message", "msg", "detail", "error_description"]))
        .unwrap_or_default();
    (code, message)
}

fn json_text(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str).and_then(nonempty))
}

fn nonempty(value: &str) -> Option<String> {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

fn choice_image(payload: &Value) -> Option<String> {
    payload
        .pointer("/output/choices")
        .and_then(Value::as_array)?
        .iter()
        .find_map(|choice| {
            choice
                .pointer("/message/content")
                .and_then(Value::as_array)?
                .iter()
                .find_map(|item| {
                    item.get("image")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|url| !url.is_empty())
                        .map(str::to_string)
                })
        })
}

fn result_image(payload: &Value) -> Option<String> {
    payload
        .pointer("/output/results")
        .and_then(Value::as_array)?
        .iter()
        .find_map(|item| {
            item.get("url")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(str::to_string)
        })
}

fn log_provider(provider: &str, status: u16, payload: &Value) {
    let (code, message) = error_summary(payload);
    tracing::error!(provider, status, code = %code, message = %message, "image provider rejected the request");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn size_and_endpoints_follow_the_go_client() {
        assert_eq!(normalize_size("720*1280"), "720*1280");
        assert_eq!(normalize_size("999*999"), "1024*1024");
        assert_eq!(normalize_prompt("  一只  猫\n\t在睡觉  "), "一只 猫 在睡觉");
        assert_eq!(
            agnes_endpoint("https://apihub.agnes-ai.com"),
            "https://apihub.agnes-ai.com/v1/images/generations"
        );
        assert_eq!(
            agnes_endpoint("https://example.test/v1/"),
            "https://example.test/v1/images/generations"
        );
        assert_eq!(
            dashscope_endpoint("https://dashscope.aliyuncs.com/api/v1"),
            "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation"
        );
    }

    #[test]
    fn parsers_accept_agnes_and_both_dashscope_shapes() {
        let agnes = json!({"data": [{"url": " https://cdn.example/a.png "}]});
        assert_eq!(
            agnes_image_url(&agnes).as_deref(),
            Some("https://cdn.example/a.png")
        );
        let choices = json!({
            "request_id": "req-1",
            "output": {"choices": [{"message": {"content": [{"image": "https://cdn.example/b.png"}]}}]},
            "usage": {"width": 1280, "height": 720}
        });
        let image = dashscope_image(&choices).expect("choices");
        assert_eq!(image.url, "https://cdn.example/b.png");
        assert_eq!(image.request_id, "req-1");
        assert_eq!((image.width, image.height), (1280, 720));
        let results = json!({"output": {"results": [{"url": "https://cdn.example/c.png"}]}});
        assert_eq!(
            dashscope_image(&results).expect("results").url,
            "https://cdn.example/c.png"
        );
    }

    #[test]
    fn only_an_agnes_policy_rejection_is_a_prompt_error() {
        let payload = json!({"error": {"code": "content_policy_violation", "message": "blocked"}});
        assert!(prompt_rejected(400, &payload));
        assert!(!prompt_rejected(500, &payload));
        assert!(!prompt_rejected(
            400,
            &json!({"error": {"message": "busy"}})
        ));
    }

    #[test]
    fn model_name_prefers_agnes_when_that_key_exists() {
        let mut cfg = Config::for_test();
        assert!(!configured(&cfg));
        assert_eq!(public_model(&cfg), "agnes-image-2.1-flash");
        cfg.dashscope_api_key = "dash".into();
        assert_eq!(public_model(&cfg), "z-image-turbo");
        cfg.agnes_api_key = "agnes".into();
        assert_eq!(public_model(&cfg), "agnes-image-2.1-flash");
    }

    #[test]
    fn cloudflare_wins_over_agnes_when_configured() {
        let mut cfg = Config::for_test();
        cfg.agnes_api_key = "agnes".into();
        cfg.cloudflare_account_id = "account".into();
        cfg.cloudflare_ai_token = "token".into();
        assert!(configured(&cfg));
        assert_eq!(public_model(&cfg), cfg.cloudflare_ai_model);
    }

    #[test]
    fn cloudflare_size_and_response_parsing() {
        assert_eq!(size_pair("1280*720"), (1280, 720));
        assert_eq!(size_pair("坏值"), (1024, 1024));
        // image/* 响应体直接是图片字节（SDXL）。
        assert_eq!(
            cloudflare_image("image/png", b"\x89PNGdata").unwrap(),
            b"\x89PNGdata".to_vec()
        );
        // flux 的 JSON base64（解出来是 JPEG 也照收）。
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"\xff\xd8\xff");
        let payload = format!("{{\"result\":{{\"image\":\"{encoded}\"}}}}");
        assert_eq!(
            cloudflare_image("application/json", payload.as_bytes()).unwrap(),
            vec![0xff, 0xd8, 0xff]
        );
        assert!(cloudflare_image("application/json", b"{}").is_none());
        assert!(cloudflare_image("text/plain", b"hello").is_none());
    }

    #[test]
    fn schema_rejection_triggers_the_prompt_only_retry() {
        let rejected = json!({"errors":[{"message":"Additional properties '/width' not allowed","code":5006}]});
        assert!(schema_rejected(&rejected));
        assert!(!schema_rejected(&json!({"errors":[{"message":"busy"}]})));
    }
}
