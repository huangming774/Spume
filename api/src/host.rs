//! 服务器面板用的内存和 CPU。
//!
//! Linux 读 `/proc/meminfo` 和 `/proc/stat`。两次 CPU 采样间隔超过 60 秒，
//! 或还没有上一笔采样时，再隔 200 毫秒采一次。Windows 读系统计数器，并缓存 10 秒。
//! 两边都读不到时，面板数字是 0，`source` 为 `process`。

use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;

/// 一次给面板的采样。
#[derive(Clone)]
pub struct HostSnapshot {
    /// 发行版或系统名，不带内核版本。
    pub os: String,
    /// 逻辑 CPU 数。
    pub cpu_cores: usize,
    /// 已用内存，MiB。
    pub memory_used_mb: i64,
    /// 总内存，MiB。
    pub memory_total_mb: i64,
    /// 已用 / 总量。
    pub memory_ratio: f64,
    /// `system` 或 `process`。
    pub memory_source: &'static str,
    /// 0 到 100，保留一位小数。
    pub cpu_percent: f64,
}

/// `/proc/stat` 里一笔 CPU 计数。
#[derive(Clone, Copy)]
pub struct CpuTimes {
    idle: u64,
    total: u64,
}

struct CpuSnap {
    times: CpuTimes,
    at: Instant,
}

static CPU_SNAP: LazyLock<Mutex<Option<CpuSnap>>> = LazyLock::new(|| Mutex::new(None));
static WINDOWS_SNAP: LazyLock<Mutex<Option<(Instant, HostSnapshot)>>> =
    LazyLock::new(|| Mutex::new(None));

/// 读当前机器。文件和命令失败时退回全 0。
pub fn sample() -> HostSnapshot {
    if let Ok(text) = std::fs::read_to_string("/proc/meminfo") {
        if let Some((used, total, ratio)) = memory_from_meminfo(&text) {
            let os_release = std::fs::read_to_string("/etc/os-release").ok();
            return HostSnapshot {
                os: friendly_os("linux", os_release.as_deref()),
                cpu_cores: cpu_cores(),
                memory_used_mb: used,
                memory_total_mb: total,
                memory_ratio: ratio,
                memory_source: "system",
                cpu_percent: linux_cpu(),
            };
        }
    }
    if let Some(snapshot) = windows_sample() {
        return snapshot;
    }
    fallback()
}

/// 没有系统计数器时的空采样。
pub fn fallback() -> HostSnapshot {
    HostSnapshot {
        os: friendly_os(std::env::consts::OS, None),
        cpu_cores: cpu_cores(),
        memory_used_mb: 0,
        memory_total_mb: 0,
        memory_ratio: 0.0,
        memory_source: "process",
        cpu_percent: 0.0,
    }
}

/// 从 `/proc/meminfo` 算出 MiB 和占用比。总量缺失时返回 None。
pub fn memory_from_meminfo(text: &str) -> Option<(i64, i64, f64)> {
    let mut total = 0_i64;
    let mut available = 0_i64;
    let mut free = 0_i64;
    let mut buffers = 0_i64;
    let mut cached = 0_i64;
    let mut saw_total = false;
    let mut saw_available = false;
    for line in text.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let Some(kb) = rest
            .split_whitespace()
            .next()
            .and_then(|item| item.parse().ok())
        else {
            continue;
        };
        match key.trim() {
            "MemTotal" => {
                total = kb;
                saw_total = true;
            }
            "MemAvailable" => {
                available = kb;
                saw_available = true;
            }
            "MemFree" => free = kb,
            "Buffers" => buffers = kb,
            "Cached" => cached = kb,
            _ => {}
        }
    }
    if !saw_total || total <= 0 {
        return None;
    }
    if !saw_available {
        available = free + buffers + cached;
    }
    let used = (total - available).max(0);
    Some((used / 1024, total / 1024, used as f64 / total as f64))
}

/// 解析 `/proc/stat` 的总 CPU 行。idle 含 iowait。
pub fn cpu_times(line: &str) -> Option<CpuTimes> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.first().copied() != Some("cpu") || fields.len() < 5 {
        return None;
    }
    let mut idle = 0_u64;
    let mut total = 0_u64;
    for (index, field) in fields.iter().enumerate().skip(1) {
        let Ok(value) = field.parse::<u64>() else {
            continue;
        };
        total += value;
        if index == 4 || index == 5 {
            idle += value;
        }
    }
    Some(CpuTimes { idle, total })
}

/// 两笔采样之间的占用率，限制在 0 到 100，保留一位小数。
pub fn cpu_percent(previous: CpuTimes, current: CpuTimes) -> f64 {
    let idle = current.idle.saturating_sub(previous.idle) as f64;
    let total = current.total.saturating_sub(previous.total) as f64;
    if total <= 0.0 {
        return 0.0;
    }
    let percent = ((1.0 - idle / total) * 100.0).clamp(0.0, 100.0);
    (percent * 10.0).round() / 10.0
}

/// 从 `os-release` 取 `NAME` 的第一段。
pub fn distro_name(text: &str) -> Option<String> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("NAME=") else {
            continue;
        };
        let name = rest.trim().trim_matches(|ch| ch == '"' || ch == '\'');
        let name = name.split([' ', '/']).next().unwrap_or(name).trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// 系统显示名。Linux 用发行版，不带内核版本。
pub fn friendly_os(os: &str, os_release: Option<&str>) -> String {
    match os {
        "linux" => os_release
            .and_then(distro_name)
            .unwrap_or_else(|| "Linux".into()),
        "windows" => "Windows".into(),
        "macos" | "darwin" => "macOS".into(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
                None => "Unknown".into(),
            }
        }
    }
}

/// 解析 Windows 计数器 JSON。内存单位是 KB，CPU 是 0 到 100。
pub fn windows_counters(text: &str) -> Option<(i64, i64, f64, f64)> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    let value: Value = serde_json::from_str(&text[start..=end]).ok()?;
    let total_kb = json_i64(value.get("totalKB"))?;
    let free_kb = json_i64(value.get("freeKB")).unwrap_or(0);
    if total_kb <= 0 {
        return None;
    }
    let used_kb = (total_kb - free_kb).max(0);
    let cpu = json_f64(value.get("cpu")).unwrap_or(0.0).clamp(0.0, 100.0);
    Some((
        used_kb / 1024,
        total_kb / 1024,
        used_kb as f64 / total_kb as f64,
        (cpu * 10.0).round() / 10.0,
    ))
}

fn linux_cpu() -> f64 {
    let Ok(text) = std::fs::read_to_string("/proc/stat") else {
        return 0.0;
    };
    let Some(line) = text.lines().next() else {
        return 0.0;
    };
    let Some(current) = cpu_times(line) else {
        return 0.0;
    };
    let mut slot = CPU_SNAP.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(previous) = slot.as_ref() {
        if previous.at.elapsed() <= Duration::from_secs(60) {
            let percent = cpu_percent(previous.times, current);
            *slot = Some(CpuSnap {
                times: current,
                at: Instant::now(),
            });
            return percent;
        }
    }
    drop(slot);
    std::thread::sleep(Duration::from_millis(200));
    let Ok(text) = std::fs::read_to_string("/proc/stat") else {
        return 0.0;
    };
    let Some(second) = text.lines().next().and_then(cpu_times) else {
        return 0.0;
    };
    let percent = cpu_percent(current, second);
    *CPU_SNAP.lock().unwrap_or_else(|error| error.into_inner()) = Some(CpuSnap {
        times: second,
        at: Instant::now(),
    });
    percent
}

fn windows_sample() -> Option<HostSnapshot> {
    if !cfg!(windows) {
        return None;
    }
    let slot = WINDOWS_SNAP
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some((at, snapshot)) = slot.as_ref() {
        if at.elapsed() <= Duration::from_secs(10) {
            return Some(snapshot.clone());
        }
    }
    drop(slot);
    let snapshot = read_windows()?;
    *WINDOWS_SNAP
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some((Instant::now(), snapshot.clone()));
    Some(snapshot)
}

fn read_windows() -> Option<HostSnapshot> {
    let mut command = std::process::Command::new("powershell");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$os = Get-CimInstance Win32_OperatingSystem; $cpu = (Get-CimInstance Win32_Processor | Measure-Object LoadPercentage -Average).Average; @{ totalKB = [int64]$os.TotalVisibleMemorySize; freeKB = [int64]$os.FreePhysicalMemory; cpu = [double]$cpu } | ConvertTo-Json -Compress",
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let (used, total, ratio, cpu) = windows_counters(&text)?;
    Some(HostSnapshot {
        os: "Windows".into(),
        cpu_cores: cpu_cores(),
        memory_used_mb: used,
        memory_total_mb: total,
        memory_ratio: ratio,
        memory_source: "system",
        cpu_percent: cpu,
    })
}

fn cpu_cores() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

fn json_i64(value: Option<&Value>) -> Option<i64> {
    value
        .and_then(Value::as_i64)
        .or_else(|| value.and_then(Value::as_f64).map(|number| number as i64))
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .or_else(|| value.and_then(Value::as_i64).map(|number| number as f64))
}
