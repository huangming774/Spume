use std::time::{Duration, Instant};

use serde_json::json;
use taozhiyy_api::ai_model::{filter_avoid, filter_avoid_for, free_text_models, pick_random};

#[test]
fn free_models_are_filtered_by_suffix_price_and_text_output() {
    let body = json!({"data": [
        {"id": "a/chat:free",
         "architecture": {"output_modalities": ["text"]},
         "pricing": {"prompt": "0", "completion": "0"}},
        {"id": "b/paid",
         "architecture": {"output_modalities": ["text"]},
         "pricing": {"prompt": "0.1", "completion": "0.2"}},
        {"id": "c/embed:free",
         "architecture": {"output_modalities": ["embedding"]},
         "pricing": {"prompt": "0", "completion": "0"}},
        {"id": "d/modality-only:free",
         "architecture": {"modality": "text+image->text"},
         "pricing": {}}
    ]});
    // 付费的 b 被价格挡掉，纯 embedding 的 c 被输出模态挡掉。
    assert_eq!(
        free_text_models(&body),
        vec![
            "a/chat:free".to_string(),
            "d/modality-only:free".to_string()
        ]
    );
}

#[test]
fn malformed_payloads_yield_no_candidates() {
    assert_eq!(free_text_models(&json!({})), Vec::<String>::new());
    assert_eq!(
        free_text_models(&json!({"data": "oops"})),
        Vec::<String>::new()
    );
}

#[test]
fn pick_random_handles_empty_and_single() {
    assert_eq!(pick_random(&[]), None);
    let only = vec!["only:free".to_string()];
    assert_eq!(pick_random(&only), Some("only:free".to_string()));
}

#[test]
fn pick_random_stays_within_the_candidates() {
    let models: Vec<String> = (0..8).map(|i| format!("m{i}:free")).collect();
    for _ in 0..20 {
        let picked = pick_random(&models).expect("pick");
        assert!(models.contains(&picked));
    }
}

#[test]
fn avoid_list_drops_recently_failed_models_only() {
    let models = vec!["good:free".to_string(), "bad:free".to_string()];
    let fresh = vec![("bad:free".to_string(), Instant::now())];
    assert_eq!(
        filter_avoid(&models, &fresh),
        vec!["good:free".to_string()],
        "回避期内的模型不再参与轮换"
    );

    // 用可注入的短回避期验证过期分支：Windows 的 Instant 锚定开机时刻，
    // 减不出 24 小时前的时间点，短时长 + 小睡跨平台都稳定。
    let short = Duration::from_millis(50);
    let expired_at = Instant::now();
    std::thread::sleep(Duration::from_millis(120));
    let expired = vec![("bad:free".to_string(), expired_at)];
    assert_eq!(
        filter_avoid_for(&models, &expired, short),
        models,
        "回避期过了就恢复候选资格"
    );

    assert_eq!(filter_avoid(&models, &[]), models, "空回避名单不影响候选");
}
