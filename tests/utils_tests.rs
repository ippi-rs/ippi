use ippi::utils;

#[test]
fn test_timestamp() {
    let ts1 = utils::timestamp();
    let ts2 = utils::timestamp();

    assert!(ts2 >= ts1);
}

#[test]
fn test_timestamp_ms() {
    let ts1 = utils::timestamp_ms();
    let ts2 = utils::timestamp_ms();

    assert!(ts2 >= ts1);
}

#[test]
fn test_format_duration() {
    assert_eq!(utils::format_duration(0), "0s");
    assert_eq!(utils::format_duration(30), "30s");
    assert_eq!(utils::format_duration(90), "1m 30s");
    assert_eq!(utils::format_duration(3600), "1h 0m");
    assert_eq!(utils::format_duration(3660), "1h 1m");
    assert_eq!(utils::format_duration(86400), "1d 0h");
    assert_eq!(utils::format_duration(90000), "1d 1h");
}

#[test]
fn test_human_bytes() {
    assert_eq!(utils::human_bytes(0), "0 B");
    assert_eq!(utils::human_bytes(1023), "1023.00 B");
    assert_eq!(utils::human_bytes(1024), "1.00 KB");
    assert_eq!(utils::human_bytes(1024 * 1024), "1.00 MB");
    assert_eq!(utils::human_bytes(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(utils::human_bytes(1024 * 1024 * 1024 * 1024), "1.00 TB");
    assert_eq!(utils::human_bytes(1500), "1.46 KB");
    assert_eq!(utils::human_bytes(1500000), "1.43 MB");
}
