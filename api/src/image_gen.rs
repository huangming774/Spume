//! 生图上游。
//!
//! 配了 `AGNES_API_KEY` 时优先用 Agnes。否则用百炼的通义万相多模态接口。
//! 两个都没配时，接口告诉前端生图还不能用。

use serde_json::{json, Value};
use thiserror::Error;

use crate::config::Config;

const AGNES_FAILED: &str = "Agnes 生图暂时失败了，请稍后再试。";
pub const PROMPT_REJECTED_MESSAGE: &str = "提示词被 Agnes 拒绝了，可能包含不适合的内容或过于接近敏感描述。请修改为更安全、更中性的说法后重试。";

/// 上游返回的一张图。
pub struct GeneratedImage {
    /// 临时下载地址。
    pub url: String,
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

/// 至少一个生图密钥已经配置。
pub fn configured(cfg: &Config) -> bool {
    !cfg.agnes_api_key.is_empty() || !cfg.dashscope_api_key.is_empty()
}

/// 配额接口展示的模型名。有 Agnes 密钥，或百炼也没配时，展示 Agnes 模型。
pub fn public_model(cfg: &Config) -> &str {
    if !cfg.agnes_api_key.is_empty() || cfg.dashscope_api_key.is_empty() {
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

/// 调用已配置的上游。Agnes 优先于百炼。
pub async fn generate(
    http: &reqwest::Client,
    cfg: &Config,
    prompt: &str,
    size: &str,
) -> Result<GeneratedImage, ImageGenError> {
    let size = normalize_size(size);
    if !cfg.agnes_api_key.is_empty() {
        return request_agnes(http, cfg, prompt, size).await;
    }
    if !cfg.dashscope_api_key.is_empty() {
        return request_dashscope(http, cfg, prompt, size).await;
    }
    Err(ImageGenError::NotConfigured)
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
}
