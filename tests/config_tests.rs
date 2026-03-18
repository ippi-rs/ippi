use ippi::Config;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_config_default() {
    let config = Config::default();

    assert_eq!(config.web.host, "0.0.0.0");
    assert_eq!(config.web.port, 8080);
    assert_eq!(config.web.cors_origins, vec!["*"]);

    assert!(config.kvm.is_some());
    assert!(!config.kvm.as_ref().unwrap().enabled);
    assert_eq!(config.kvm.as_ref().unwrap().device_path, "/dev/kvm");
    assert_eq!(config.kvm.as_ref().unwrap().memory_mb, 1024);

    assert!(config.p2p.is_some());
    assert!(!config.p2p.as_ref().unwrap().enabled);

    assert!(config.webrtc.is_some());
    assert!(!config.webrtc.as_ref().unwrap().enabled);

    assert!(config.cloud_init.is_some());
    assert!(!config.cloud_init.as_ref().unwrap().enabled);
}

#[tokio::test]
async fn test_config_load_missing() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    drop(temp_file); // File is deleted, path doesn't exist

    // File doesn't exist, should return defaults
    let config = Config::load(path).await.unwrap();
    assert_eq!(config.web.host, "0.0.0.0");
    assert_eq!(config.web.port, 8080);
}

#[tokio::test]
async fn test_config_save_load() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut config = Config::default();
    config.web.port = 9090;
    config.web.host = "127.0.0.1".to_string();

    // Save config
    config.save(path).await.unwrap();

    // Load config
    let loaded_config = Config::load(path).await.unwrap();

    assert_eq!(loaded_config.web.port, 9090);
    assert_eq!(loaded_config.web.host, "127.0.0.1");
}

#[tokio::test]
async fn test_config_invalid_toml() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Write invalid TOML
    std::fs::write(path, "invalid toml content").unwrap();

    let result = Config::load(path).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, ippi::Error::Config(_)));
}
