use taozhiyy_api::host::{
    cpu_percent, cpu_times, distro_name, friendly_os, memory_from_meminfo, windows_counters,
};

#[test]
fn meminfo_uses_available_memory() {
    let text = "\
MemTotal:       8000000 kB
MemFree:        1000000 kB
MemAvailable:   5000000 kB
Buffers:         100000 kB
Cached:         1000000 kB
";
    let (used, total, ratio) = memory_from_meminfo(text).expect("memory");
    assert_eq!(total, 8000000 / 1024);
    assert_eq!(used, 3000000 / 1024);
    assert!((ratio - 0.375).abs() < 0.001);
}

#[test]
fn cpu_delta_counts_idle_and_iowait() {
    let previous = cpu_times("cpu  0 0 0 0 0").expect("previous");
    let current = cpu_times("cpu  40 0 30 20 10").expect("current");
    assert_eq!(cpu_percent(previous, current), 70.0);
}

#[test]
fn distro_name_drops_the_version() {
    let text = "NAME=\"Ubuntu 22.04.5 LTS\"\nVERSION=22.04\n";
    assert_eq!(distro_name(text).as_deref(), Some("Ubuntu"));
    assert_eq!(friendly_os("linux", Some(text)), "Ubuntu");
    assert_eq!(friendly_os("windows", None), "Windows");
}

#[test]
fn windows_counter_json_becomes_mebibytes() {
    let (used, total, ratio, cpu) =
        windows_counters(r#"warning {"totalKB":8192,"freeKB":2048,"cpu":12.34}"#).expect("windows");
    assert_eq!(total, 8);
    assert_eq!(used, 6);
    assert!((ratio - 0.75).abs() < 0.001);
    assert_eq!(cpu, 12.3);
}
