//! 导航栏天气挂件：按访客 IP 定位（ip-api.com，免费无 key），再用
//! Open-Meteo（免费无 key）取实况天气。结果按 IP 做 10 分钟内存缓存，
//! 避免每次页面加载都打外部接口。

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use tokio::sync::{OnceCell, Semaphore};

use crate::guestbook::is_private_ip;
use crate::routes::{client_ip, fail};
use crate::AppState;

/// 天气缓存保鲜期。天气变化慢，10 分钟足够。
const FRESH_FOR: Duration = Duration::from_secs(10 * 60);

/// 私有/本地 IP 拿不到真实定位时的兜底坐标（上海），
/// 让本地开发也能看到挂件，不至于整块隐藏。
const FALLBACK_LAT: f64 = 31.2304;
const FALLBACK_LON: f64 = 121.4737;
const FALLBACK_CITY: &str = "上海";
const FALLBACK_REGION: &str = "上海";

/// 失败结果的短缓存，避免下游全超时时空转重试。
const NEGATIVE_FOR: Duration = Duration::from_secs(30);

/// 单条缓存：抓取时间、响应体，以及这次是不是失败。
struct Cached {
    fetched_at: Instant,
    body: Value,
    negative: bool,
}

/// 进程内缓存。同一 IP 同时只有一个在途刷新。
#[derive(Default)]
pub struct WeatherCache {
    entries: HashMap<String, Cached>,
    inflight: HashMap<String, Arc<OnceCell<Result<Value, String>>>>,
    permits: Option<Arc<Semaphore>>,
}

pub type SharedCache = Arc<Mutex<WeatherCache>>;

pub fn new_cache() -> SharedCache {
    Arc::new(Mutex::new(WeatherCache {
        entries: HashMap::new(),
        inflight: HashMap::new(),
        permits: Some(Arc::new(Semaphore::new(8))),
    }))
}

/// 把 reqwest 等错误的 source 链摊平成一行，方便在日志里看到
/// 底层是 DNS、连接被拒还是超时，而不是只有一句 "error sending request"。
fn error_chain(error: &dyn Error) -> String {
    let mut chain = vec![error.to_string()];
    let mut source = error.source();
    while let Some(cause) = source {
        chain.push(cause.to_string());
        source = cause.source();
    }
    chain.join(": ")
}

/// `GET /api/client/place`。只返回访客 IP 对应的坐标，给菜单圆框里的地图用。
/// 本地回环地址没有公网位置，`located` 为 false，前端再改用浏览器侧定位。
pub async fn place(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let ip = client_ip(&headers);
    let fallback = json!({
        "lat": FALLBACK_LAT,
        "lon": FALLBACK_LON,
        "city": FALLBACK_CITY,
        "region": FALLBACK_REGION,
        "located": false,
    });
    if is_private_ip(&ip) {
        return Json(fallback).into_response();
    }
    match fetch_geo(&state, &ip).await {
        Ok(geo) => Json(json!({
            "lat": geo.lat,
            "lon": geo.lon,
            "city": geo.city,
            "region": geo.region,
            "located": true,
        }))
        .into_response(),
        Err(error) => {
            tracing::warn!(%error, "client place lookup failed");
            Json(fallback).into_response()
        }
    }
}

/// `GET /api/weather`。拿不到定位或天气时返回 502，前端自行隐藏挂件。
pub async fn get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let ip = client_ip(&headers);
    match resolve(&state, &ip).await {
        Ok(body) => Json(body).into_response(),
        Err(error) => {
            tracing::warn!(%error, deadline = "background", "weather lookup failed");
            fail(
                StatusCode::BAD_GATEWAY,
                "WEATHER_UNAVAILABLE",
                "天气信息暂时拿不到",
            )
        }
    }
}

async fn resolve(state: &AppState, ip: &str) -> Result<Value, String> {
    if let Some(hit) = fresh_cached(state, ip) {
        return hit;
    }
    let flight = {
        let mut cache = state
            .weather_cache
            .lock()
            .map_err(|_| "weather cache locked".to_string())?;
        cache
            .inflight
            .entry(ip.to_string())
            .or_insert_with(|| Arc::new(OnceCell::new()))
            .clone()
    };
    let permits = state
        .weather_cache
        .lock()
        .ok()
        .and_then(|cache| cache.permits.clone());
    let result = flight
        .get_or_init(|| async {
            let permit = match &permits {
                Some(permits) => tokio::time::timeout(
                    state.cfg.timeouts.weather,
                    permits.clone().acquire_owned(),
                )
                .await
                .ok()
                .and_then(Result::ok),
                None => None,
            };
            if permits.is_some() && permit.is_none() {
                return Err("weather busy".into());
            }
            let result = fetch_weather(state, ip).await;
            drop(permit);
            result
        })
        .await
        .clone();
    if let Ok(mut cache) = state.weather_cache.lock() {
        cache.inflight.remove(ip);
        match &result {
            Ok(body) => insert_cached(&mut cache, ip, body.clone(), false),
            Err(_) => insert_cached(&mut cache, ip, json!({}), true),
        }
    }
    result
}

/// 只读缓存，不发网络请求。
fn fresh_cached(state: &AppState, ip: &str) -> Option<Result<Value, String>> {
    let cache = state.weather_cache.lock().ok()?;
    let entry = cache.entries.get(ip)?;
    let age = entry.fetched_at.elapsed();
    if entry.negative && age < NEGATIVE_FOR {
        return Some(Err("weather negative cache".into()));
    }
    if !entry.negative && age < FRESH_FOR {
        return Some(Ok(entry.body.clone()));
    }
    None
}

fn insert_cached(cache: &mut WeatherCache, ip: &str, body: Value, negative: bool) {
    if cache.entries.len() > 4096 {
        cache.entries.clear();
    }
    cache.entries.insert(
        ip.to_string(),
        Cached {
            fetched_at: Instant::now(),
            body,
            negative,
        },
    );
}

async fn within<T>(
    limit: Duration,
    work: impl std::future::Future<Output = Result<T, String>>,
) -> Result<T, String> {
    match tokio::time::timeout(limit, work).await {
        Ok(result) => result,
        Err(_) => Err("upstream timed out".into()),
    }
}

async fn fetch_weather(state: &AppState, ip: &str) -> Result<Value, String> {
    let limit = state.cfg.timeouts.weather;
    let geo = within(limit, fetch_geo(state, ip)).await?;
    let weather = fetch_current_weather(state, geo.lat, geo.lon, &geo.city, limit).await?;
    Ok(json!({
        "city": geo.city,
        "region": geo.region,
        "temperature": weather.temperature,
        "weatherCode": weather.weather_code,
        "weatherDescription": weather_description(weather.weather_code),
        "windSpeed": weather.wind_speed,
        "isDay": weather.is_day,
    }))
}

struct Geo {
    lat: f64,
    lon: f64,
    city: String,
    region: String,
}

struct CurrentWeather {
    temperature: f64,
    weather_code: i64,
    wind_speed: f64,
    is_day: bool,
}

/// ip-api.com 免费无 key（仅限 http，服务端请求不受混合内容限制）。
/// 请求字段和 `guestbook::fetch_region` 一致的风格，但这里要经纬度，
/// 所以单独一条请求，不复用那边的纯地区字符串缓存。
async fn fetch_geo(state: &AppState, ip: &str) -> Result<Geo, String> {
    // 私有/本地 IP（比如开发时的 127.0.0.1、::1）没有公网定位，直接兜底。
    if is_private_ip(ip) {
        return Ok(Geo {
            lat: FALLBACK_LAT,
            lon: FALLBACK_LON,
            city: FALLBACK_CITY.to_string(),
            region: FALLBACK_REGION.to_string(),
        });
    }
    if ip.is_empty()
        || ip.len() > 64
        || !ip
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '.' || ch == ':')
    {
        return Err("invalid client ip".into());
    }
    let url =
        format!("http://ip-api.com/json/{ip}?lang=zh-CN&fields=status,lat,lon,city,regionName");
    let response = state
        .http
        .get(&url)
        .header("User-Agent", "taozhiyy-weather/1.0")
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|error| error_chain(&error))?;
    if !response.status().is_success() {
        return Err(format!("geo lookup http {}", response.status()));
    }
    let body = response.text().await.map_err(|error| error.to_string())?;
    let value: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    if value.get("status").and_then(Value::as_str) != Some("success") {
        return Err("geo lookup not success".into());
    }
    let lat = value
        .get("lat")
        .and_then(Value::as_f64)
        .ok_or("geo lookup missing lat")?;
    let lon = value
        .get("lon")
        .and_then(Value::as_f64)
        .ok_or("geo lookup missing lon")?;
    let city = value
        .get("city")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let region = value
        .get("regionName")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(Geo {
        lat,
        lon,
        city,
        region,
    })
}

/// Open-Meteo 和 wttr.in 同时开查，各自 4 秒截止。
/// Open-Meteo 先成功就采用它并取消备用源；失败则用已经在跑的 wttr.in。
async fn fetch_current_weather(
    state: &AppState,
    lat: f64,
    lon: f64,
    city: &str,
    limit: Duration,
) -> Result<CurrentWeather, String> {
    let primary = within(limit, fetch_open_meteo(state, lat, lon));
    let fallback = within(limit, fetch_wttr(state, city, lat, lon));
    tokio::pin!(primary);
    tokio::pin!(fallback);
    tokio::select! {
        result = &mut primary => match result {
            Ok(weather) => Ok(weather),
            Err(primary_error) => {
                tracing::warn!(%primary_error, "open-meteo failed; using wttr.in");
                fallback.await
            }
        },
        result = &mut fallback => match result {
            Ok(weather) => match primary.await {
                Ok(preferred) => Ok(preferred),
                Err(_) => Ok(weather),
            },
            Err(_) => primary.await,
        },
    }
}

async fn fetch_open_meteo(state: &AppState, lat: f64, lon: f64) -> Result<CurrentWeather, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true"
    );
    let response = state
        .http
        .get(&url)
        .header("User-Agent", "taozhiyy-weather/1.0")
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|error| error_chain(&error))?;
    if !response.status().is_success() {
        return Err(format!("weather lookup http {}", response.status()));
    }
    let body = response.text().await.map_err(|error| error.to_string())?;
    let value: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    let current = value
        .get("current_weather")
        .ok_or("weather response missing current_weather")?;
    let temperature = current
        .get("temperature")
        .and_then(Value::as_f64)
        .ok_or("weather response missing temperature")?;
    let weather_code = current
        .get("weathercode")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let wind_speed = current
        .get("windspeed")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let is_day = current.get("is_day").and_then(Value::as_i64).unwrap_or(1) == 1;
    Ok(CurrentWeather {
        temperature,
        weather_code,
        wind_speed,
        is_day,
    })
}

/// wttr.in 免费无 key 备用源，用 `format=j1` 拿 JSON。
/// 天气代码是 WorldWeatherOnline 的编码，先映射成 WMO 码再复用同一套描述/图标。
async fn fetch_wttr(
    state: &AppState,
    city: &str,
    lat: f64,
    lon: f64,
) -> Result<CurrentWeather, String> {
    // 优先按城市名查（更稳），查不到再退回坐标。
    let query = if city.is_empty() {
        format!("{lat:.2},{lon:.2}")
    } else {
        city.to_string()
    };
    let url = format!("https://wttr.in/{query}?format=j1");
    let response = state
        .http
        .get(&url)
        .header("User-Agent", "taozhiyy-weather/1.0")
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|error| error_chain(&error))?;
    if !response.status().is_success() {
        return Err(format!("wttr lookup http {}", response.status()));
    }
    let body = response.text().await.map_err(|error| error.to_string())?;
    let value: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    let current = value
        .get("current_condition")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .ok_or("wttr response missing current_condition")?;
    let temperature = current
        .get("temp_C")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or("wttr response missing temp_C")?;
    let wwo_code = current
        .get("weatherCode")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(113);
    let wind_speed = current
        .get("windspeedKmph")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    // wttr.in 没有 is_day，用当地时间 6~18 点粗略判断。
    let is_day = current
        .get("localObsDateTime")
        .and_then(Value::as_str)
        .and_then(|s| s.split_whitespace().nth(1))
        .and_then(|time| time.split(':').next())
        .and_then(|hour| hour.parse::<u32>().ok())
        .map(|hour| (6..18).contains(&hour))
        .unwrap_or(true);
    Ok(CurrentWeather {
        temperature,
        weather_code: wwo_to_wmo(wwo_code),
        wind_speed,
        is_day,
    })
}

/// WorldWeatherOnline 天气代码 → WMO 代码，够区分图标和中文描述即可。
fn wwo_to_wmo(code: i64) -> i64 {
    match code {
        113 => 0,                                                                    // Clear/Sunny
        116 => 2,              // Partly cloudy
        119 | 122 => 3,        // Cloudy / Overcast
        143 | 248 | 260 => 45, // Mist / Fog / Freezing fog
        176 | 263 | 266 | 293 | 296 | 299 | 302 | 305 | 308 | 353 | 356 | 359 => 61, // Rain
        179 | 182 | 185 | 227 | 230 | 311 | 314 | 317 | 320 | 323 | 326 | 329 | 332 | 335 | 338
        | 350 | 368 | 371 | 374 | 377 => 71, // Snow/sleet
        200 | 386 | 389 | 392 | 395 => 95, // Thunderstorm
        _ => 3,
    }
}

/// WMO 天气代码 → 中文描述。代码取自 Open-Meteo 文档的标准映射。
fn weather_description(code: i64) -> &'static str {
    match code {
        0 => "晴",
        1 => "大致晴朗",
        2 => "局部多云",
        3 => "阴",
        45 | 48 => "雾",
        51 | 53 | 55 => "毛毛雨",
        56 | 57 => "冻毛毛雨",
        61 | 63 | 65 => "雨",
        66 | 67 => "冻雨",
        71 | 73 | 75 => "雪",
        77 => "雪粒",
        80..=82 => "阵雨",
        85 | 86 => "阵雪",
        95 => "雷阵雨",
        96 | 99 => "雷阵雨伴冰雹",
        _ => "未知",
    }
}
