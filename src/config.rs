use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub web: WebConfig,
    pub kvm: Option<KvmConfig>,
    pub p2p: Option<P2pConfig>,
    pub webrtc: Option<WebRtcConfig>,
    pub cloud_init: Option<CloudInitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvmConfig {
    pub enabled: bool,
    pub device_path: String,
    pub memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    pub enabled: bool,
    pub bootstrap_nodes: Vec<String>,
    pub listen_port: u16,
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    pub enabled: bool,
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<TurnServer>,
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    pub url: String,
    pub username: String,
    pub credential: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudInitConfig {
    pub enabled: bool,
    pub data_source: String,
    pub metadata_url: String,
    pub userdata_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            web: WebConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
            },
            kvm: Some(KvmConfig {
                enabled: false,
                device_path: "/dev/kvm".to_string(),
                memory_mb: 1024,
            }),
            p2p: Some(P2pConfig {
                enabled: false,
                bootstrap_nodes: vec![
                    "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".to_string(),
                ],
                listen_port: 0,
                protocol_version: "/ippi/0.1.0".to_string(),
            }),
            webrtc: Some(WebRtcConfig {
                enabled: false,
                stun_servers: vec![
                    "stun:stun.l.google.com:19302".to_string(),
                    "stun:global.stun.twilio.com:3478".to_string(),
                ],
                turn_servers: vec![],
                video_bitrate: 2000,
                audio_bitrate: 128,
            }),
            cloud_init: Some(CloudInitConfig {
                enabled: false,
                data_source: "NoCloud".to_string(),
                metadata_url: "http://169.254.169.254/latest/meta-data".to_string(),
                userdata_url: "http://169.254.169.254/latest/user-data".to_string(),
            }),
        }
    }
}

impl Config {
    pub async fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            tracing::warn!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub async fn save<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content).await?;
        Ok(())
    }
}
