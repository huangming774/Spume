//! OpenRouter 免费模型的每日轮换。
//!
//! 免费模型的名单经常变（上线、下线、改名），所以每天重新检测一次：
//! QStash 定时调 `/api/internal/refresh-ai-model`，平时聊天发现缓存超过
//! 24 小时也会现检兜底。每次现检都从可用的免费模型里随机挑一个，形成轮换。
//! 检测失败时回退到 `DEEPSEEK_MODEL` 兜底。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::seq::IndexedRandom;
use serde_json::Value;

use crate::AppState;

/// 缓存保鲜期。超过就现检换新。
pub const FRESH_FOR: Duration = Duration::from_secs(24 * 60 * 60);

/// 当前选中的免费模型和选择时间。
#[derive(Default)]
pub struct ModelCache {
    model: String,
    picked_at: Option<Instant>,
    /// 最近用失败过的模型（403/404/429 这类模型侧拒绝），一段时间内不再选。
    avoid: Vec<(String, Instant)>,
}

/// 坏模型的回避期：和保鲜期一致，一天后再给机会。
pub const AVOID_FOR: Duration = FRESH_FOR;

/// 进程内缓存，挂在 [AppState] 上跨请求共享。
pub type SharedCache = Arc<Mutex<ModelCache>>;

pub fn new_cache() -> SharedCache {
    Arc::new(Mutex::new(ModelCache::default()))
}

/// 取当前模型。缓存新鲜就直接用；过期或没选过就现检一次，失败回退环境变量兜底。
pub async fn resolve(state: &AppState) -> String {
    if let Some(model) = fresh_model(state) {
        return model;
    }
    match refresh(state).await {
        Ok(model) => model,
        Err(error) => {
            tracing::warn!(%error, "free model refresh failed; falling back to DEEPSEEK_MODEL");
            state.cfg.deepseek_model.clone()
        }
    }
}

/// 上游把这个模型拒了（403/404/429 等）：记进回避名单，并作废当前缓存让下次换个模型。
pub fn mark_failed(state: &AppState, model: &str) {
    if let Ok(mut cache) = state.ai_model.lock() {
        if !cache.avoid.iter().any(|(item, _)| item == model) {
            cache.avoid.push((model.to_string(), Instant::now()));
        }
        if cache.model == model {
            cache.model.clear();
            cache.picked_at = None;
        }
    }
}

/// 只读缓存不发请求，给统计展示用。
pub fn cached_or_default(state: &AppState) -> String {
    fresh_model(state).unwrap_or_else(|| state.cfg.deepseek_model.clone())
}

/// 强制现检：拉模型列表、随机挑一个免费文本模型并缓存。
pub async fn refresh(state: &AppState) -> Result<String, String> {
    if state.cfg.deepseek_api_key.is_empty() {
        return Err("AI 聊天还没有配置 API Key".into());
    }
    let url = format!(
        "{}/v1/models",
        state.cfg.deepseek_base_url.trim_end_matches('/')
    );
    let response = state
        .http
        .get(url)
        .bearer_auth(&state.cfg.deepseek_api_key)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("模型列表返回 HTTP {}", response.status()));
    }
    let body: Value = response.json().await.map_err(|error| error.to_string())?;
    let candidates = available_models(state, &body);
    let picked = pick_random(&candidates).ok_or("没有可用的免费模型")?;
    if let Ok(mut cache) = state.ai_model.lock() {
        cache.model = picked.clone();
        cache.picked_at = Some(Instant::now());
    }
    tracing::info!(model = %picked, candidates = candidates.len(), "free model picked");
    Ok(picked)
}

/// 拉下来的免费模型里去掉还在回避期内的。
fn available_models(state: &AppState, body: &Value) -> Vec<String> {
    let candidates = free_text_models(body);
    let Ok(cache) = state.ai_model.lock() else {
        return candidates;
    };
    filter_avoid(&candidates, &cache.avoid)
}

/// 去掉回避期内的模型。纯函数，方便测试。
pub fn filter_avoid(models: &[String], avoid: &[(String, Instant)]) -> Vec<String> {
    filter_avoid_for(models, avoid, AVOID_FOR)
}

/// 回避期可注入的版本。测试用短时长，避免 Windows 上 Instant
/// 锚定开机时刻、减不出 24 小时前时间点的问题。
pub fn filter_avoid_for(
    models: &[String],
    avoid: &[(String, Instant)],
    avoid_for: Duration,
) -> Vec<String> {
    models
        .iter()
        .filter(|model| {
            !avoid
                .iter()
                .any(|(item, at)| item == *model && at.elapsed() < avoid_for)
        })
        .cloned()
        .collect()
}

fn fresh_model(state: &AppState) -> Option<String> {
    let cache = state.ai_model.lock().ok()?;
    if !cache.model.is_empty() && cache.picked_at.is_some_and(|at| at.elapsed() < FRESH_FOR) {
        return Some(cache.model.clone());
    }
    None
}

/// 从 `/v1/models` 响应里挑出免费的文本输出模型。纯函数，方便测试。
///
/// 免费的判据：id 带 `:free` 后缀，且 prompt/completion 价格都是 0（价格字段
/// 缺失时信后缀）。聊天要输出文本，输出模态不含 text 的（纯 embedding 等）跳过。
pub fn free_text_models(body: &Value) -> Vec<String> {
    let Some(items) = body.get("data").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let id = item.get("id").and_then(Value::as_str)?;
            if !id.ends_with(":free") {
                return None;
            }
            let pricing = item.get("pricing");
            let price_zero = |key: &str| match pricing
                .and_then(|item| item.get(key))
                .and_then(Value::as_str)
            {
                Some(raw) => raw.trim().parse::<f64>().map(|v| v == 0.0).unwrap_or(false),
                None => true,
            };
            if !price_zero("prompt") || !price_zero("completion") {
                return None;
            }
            if !text_output(item) {
                return None;
            }
            Some(id.to_string())
        })
        .collect()
}

/// 输出里要有文本。优先看 `output_modalities`，没有就看 `modality` 后缀，
/// 两个字段都没有时放行。
fn text_output(item: &Value) -> bool {
    let architecture = item.get("architecture");
    if let Some(mods) = architecture
        .and_then(|item| item.get("output_modalities"))
        .and_then(Value::as_array)
    {
        return mods.iter().any(|entry| entry.as_str() == Some("text"));
    }
    if let Some(modality) = architecture
        .and_then(|item| item.get("modality"))
        .and_then(Value::as_str)
    {
        return modality.ends_with("->text");
    }
    true
}

/// 从候选里随机挑一个（每次现检换一个，形成每日轮换）。空列表返回 None。
pub fn pick_random(models: &[String]) -> Option<String> {
    let mut rng = rand::rng();
    models.choose(&mut rng).cloned()
}
