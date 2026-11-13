use serde_json::json;
use taozhiyy_api::search_hits::{hits_from_milvus, SearchHit, SearchHitError};

#[test]
fn flat_zilliz_rows_become_hits() {
    // Zilliz Cloud 实测形状：data 是扁平命中数组（2026-11-13 真实集群验证）。
    let body = json!({
        "code": 0,
        "cost": 6,
        "data": [
            {"id": "blog:hello:0", "distance": 3.5, "title": "你好",
             "url": "/blog/hello", "excerpt": "第一段", "type": "blog"},
            {"id": "journal:log:0", "distance": 1.25, "title": "日志",
             "url": "/build/post/log", "excerpt": "", "type": "journal"}
        ],
        "topks": [1, 1]
    });
    let hits = hits_from_milvus(&body).expect("hits");
    assert_eq!(hits.len(), 2);
    assert_eq!(
        hits[0],
        SearchHit {
            title: "你好".into(),
            url: "/blog/hello".into(),
            excerpt: "第一段".into(),
            score: 3.5,
        }
    );
    // 没有 excerpt 时是空字符串，score 读的是 distance。
    assert_eq!(hits[1].excerpt, "");
    assert_eq!(hits[1].score, 1.25);
}

#[test]
fn nested_milvus_rows_also_parse() {
    // Milvus 自建版的形状：data 是「每个查询一个内层数组」。
    let body = json!({
        "code": 200,
        "data": [[
            {"id": "blog:hello:0", "distance": 2.0,
             "title": "你好", "url": "/blog/hello", "excerpt": "第一段"}
        ]]
    });
    let hits = hits_from_milvus(&body).expect("hits");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].score, 2.0);
}

#[test]
fn empty_data_is_no_hits_and_missing_data_is_an_error() {
    assert_eq!(hits_from_milvus(&json!({"code": 0, "data": []})).unwrap(), Vec::new());
    assert_eq!(
        hits_from_milvus(&json!({"code": 200})).unwrap_err(),
        SearchHitError::MissingResult
    );
    let no_url = json!({"data": [{"distance": 1.0, "title": "缺地址"}]});
    assert_eq!(
        hits_from_milvus(&no_url).unwrap_err(),
        SearchHitError::Incomplete { index: 0 }
    );
    let bad_score = json!({"data": [{"title": "t", "url": "/x", "distance": "high"}]});
    assert_eq!(
        hits_from_milvus(&bad_score).unwrap_err(),
        SearchHitError::BadScore { index: 0 }
    );
}
