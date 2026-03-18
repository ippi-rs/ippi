use crate::{Config, Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CloudInitManager {
    config: Arc<Config>,
    state: Arc<RwLock<CloudInitState>>,
    instances: Arc<RwLock<HashMap<String, InstanceInfo>>>,
    metadata_store: Arc<RwLock<HashMap<String, Metadata>>>,
}

#[derive(Debug, Clone)]
pub struct CloudInitState {
    pub enabled: bool,
    pub initialized: bool,
    pub data_source: String,
    pub metadata_url: String,
    pub userdata_url: String,
    pub instances_served: u64,
    pub last_metadata_update: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct InstanceInfo {
    pub instance_id: String,
    pub vm_id: Option<String>,
    pub created_at: Instant,
    pub last_seen: Instant,
    pub metadata: Metadata,
    pub userdata: Option<String>,
    pub status: InstanceStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceStatus {
    Pending,
    Ready,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub instance_id: String,
    pub hostname: String,
    pub local_hostname: String,
    pub public_keys: Vec<String>,
    pub network_config: Option<NetworkConfig>,
    pub storage_config: Option<StorageConfig>,
    pub custom_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub version: u8,
    pub config: Vec<NetworkInterface>,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub r#type: InterfaceType,
    pub mac_address: Option<String>,
    pub addresses: Vec<IpAddress>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceType {
    Physical,
    Bond,
    Vlan,
    Bridge,
}

#[derive(Debug, Clone)]
pub struct IpAddress {
    pub address: String,
    pub netmask: String,
    pub gateway: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub disks: Vec<Disk>,
    pub filesystems: Vec<Filesystem>,
}

#[derive(Debug, Clone)]
pub struct Disk {
    pub device: String,
    pub wipe_table: bool,
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub label: String,
    pub number: u32,
    pub size: Option<String>,
    pub start: Option<String>,
    pub filesystem: String,
    pub mountpoint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Filesystem {
    pub device: String,
    pub format: String,
    pub mountpoint: String,
    pub options: Vec<String>,
}

impl CloudInitManager {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let state = CloudInitState {
            enabled: config
                .cloud_init
                .as_ref()
                .map(|c| c.enabled)
                .unwrap_or(false),
            initialized: false,
            data_source: config
                .cloud_init
                .as_ref()
                .map(|c| c.data_source.clone())
                .unwrap_or_else(|| "NoCloud".to_string()),
            metadata_url: config
                .cloud_init
                .as_ref()
                .map(|c| c.metadata_url.clone())
                .unwrap_or_else(|| "http://169.254.169.254/latest/meta-data".to_string()),
            userdata_url: config
                .cloud_init
                .as_ref()
                .map(|c| c.userdata_url.clone())
                .unwrap_or_else(|| "http://169.254.169.254/latest/user-data".to_string()),
            instances_served: 0,
            last_metadata_update: None,
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            instances: Arc::new(RwLock::new(HashMap::new())),
            metadata_store: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.enabled {
            tracing::info!("Cloud-init support is disabled in configuration");
            return Ok(());
        }

        // Create necessary directories for NoCloud datasource
        if state.data_source == "NoCloud" {
            self.initialize_nocloud().await?;
        }

        state.initialized = true;
        state.last_metadata_update = Some(Instant::now());

        tracing::info!(
            "Cloud-init initialized with data source: {}",
            state.data_source
        );

        Ok(())
    }

    async fn initialize_nocloud(&self) -> Result<()> {
        let base_dir = Path::new("/var/lib/cloud-init");

        // Create directory structure
        let dirs = vec![
            base_dir.join("seed"),
            base_dir.join("instance"),
            base_dir.join("cache"),
            base_dir.join("scripts"),
        ];

        for dir in dirs {
            if !dir.exists() {
                fs::create_dir_all(&dir).await?;
                tracing::debug!("Created directory: {:?}", dir);
            }
        }

        Ok(())
    }

    pub async fn create_instance(&self, vm_id: Option<&str>, hostname: &str) -> Result<String> {
        let mut state = self.state.write().await;

        if !state.enabled || !state.initialized {
            return Err(Error::NotImplemented(
                "Cloud-init is not initialized".to_string(),
            ));
        }

        let instance_id = uuid::Uuid::new_v4().to_string();

        let metadata = Metadata {
            instance_id: instance_id.clone(),
            hostname: hostname.to_string(),
            local_hostname: hostname.to_string(),
            public_keys: vec![],
            network_config: None,
            storage_config: None,
            custom_data: HashMap::new(),
        };

        let instance = InstanceInfo {
            instance_id: instance_id.clone(),
            vm_id: vm_id.map(|s| s.to_string()),
            created_at: Instant::now(),
            last_seen: Instant::now(),
            metadata,
            userdata: None,
            status: InstanceStatus::Pending,
        };

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);

        let mut metadata_store = self.metadata_store.write().await;
        metadata_store.insert(instance_id.clone(), metadata);

        state.instances_served += 1;

        tracing::info!(
            "Created cloud-init instance {} for VM {:?}",
            instance_id,
            vm_id
        );

        Ok(instance_id)
    }

    pub async fn set_metadata(&self, instance_id: &str, metadata: Metadata) -> Result<()> {
        let mut instances = self.instances.write().await;
        let mut metadata_store = self.metadata_store.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.metadata = metadata.clone();
        instance.last_seen = Instant::now();

        metadata_store.insert(instance_id.to_string(), metadata);

        tracing::debug!("Updated metadata for instance {}", instance_id);

        Ok(())
    }

    pub async fn set_userdata(&self, instance_id: &str, userdata: &str) -> Result<()> {
        let mut instances = self.instances.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.userdata = Some(userdata.to_string());
        instance.last_seen = Instant::now();

        // Write userdata to file for NoCloud datasource
        if self.state.read().await.data_source == "NoCloud" {
            let userdata_path = Path::new("/var/lib/cloud-init/seed")
                .join(&instance_id)
                .with_extension("user-data");

            fs::write(&userdata_path, userdata).await?;
            tracing::debug!("Wrote userdata to {:?}", userdata_path);
        }

        tracing::debug!("Set userdata for instance {}", instance_id);

        Ok(())
    }

    pub async fn get_metadata(&self, instance_id: &str) -> Result<Option<Metadata>> {
        let metadata_store = self.metadata_store.read().await;
        Ok(metadata_store.get(instance_id).cloned())
    }

    pub async fn get_userdata(&self, instance_id: &str) -> Result<Option<String>> {
        let instances = self.instances.read().await;
        Ok(instances.get(instance_id).and_then(|i| i.userdata.clone()))
    }

    pub async fn serve_metadata(&self, instance_id: &str) -> Result<String> {
        let mut instances = self.instances.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.last_seen = Instant::now();

        let metadata = &instance.metadata;

        // Generate cloud-init metadata format
        let mut output = String::new();

        output.push_str(&format!("instance-id: {}\n", metadata.instance_id));
        output.push_str(&format!("local-hostname: {}\n", metadata.local_hostname));

        if !metadata.public_keys.is_empty() {
            output.push_str("public-keys:\n");
            for key in &metadata.public_keys {
                output.push_str(&format!("  - {}\n", key));
            }
        }

        if let Some(network_config) = &metadata.network_config {
            output.push_str("network-config:\n");
            // Simplified network config output
            output.push_str("  version: 2\n");
            output.push_str("  ethernets:\n");
            for interface in &network_config.config {
                output.push_str(&format!("    {}:\n", interface.name));
                if let Some(mac) = &interface.mac_address {
                    output.push_str(&format!("      match:\n        macaddress: {}\n", mac));
                }
                if !interface.addresses.is_empty() {
                    output.push_str("      addresses:\n");
                    for addr in &interface.addresses {
                        output.push_str(&format!("        - {}/{}\n", addr.address, addr.netmask));
                    }
                }
                if let Some(gateway) = &interface.gateway {
                    output.push_str(&format!("      gateway4: {}\n", gateway));
                }
                if !interface.dns_servers.is_empty() {
                    output.push_str("      nameservers:\n");
                    output.push_str("        addresses:\n");
                    for dns in &interface.dns_servers {
                        output.push_str(&format!("          - {}\n", dns));
                    }
                }
            }
        }

        // Add custom data
        for (key, value) in &metadata.custom_data {
            output.push_str(&format!("{}: {}\n", key, value));
        }

        Ok(output)
    }

    pub async fn serve_userdata(&self, instance_id: &str) -> Result<Option<String>> {
        let mut instances = self.instances.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.last_seen = Instant::now();

        Ok(instance.userdata.clone())
    }

    pub async fn update_instance_status(
        &self,
        instance_id: &str,
        status: InstanceStatus,
    ) -> Result<()> {
        let mut instances = self.instances.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.status = status;
        instance.last_seen = Instant::now();

        tracing::debug!(
            "Updated instance {} status to {:?}",
            instance_id,
            instance.status
        );

        Ok(())
    }

    pub async fn add_ssh_key(&self, instance_id: &str, public_key: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        let mut metadata_store = self.metadata_store.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.metadata.public_keys.push(public_key.to_string());
        instance.last_seen = Instant::now();

        if let Some(metadata) = metadata_store.get_mut(instance_id) {
            metadata.public_keys.push(public_key.to_string());
        }

        tracing::debug!("Added SSH key to instance {}", instance_id);

        Ok(())
    }

    pub async fn set_network_config(&self, instance_id: &str, config: NetworkConfig) -> Result<()> {
        let mut instances = self.instances.write().await;
        let mut metadata_store = self.metadata_store.write().await;

        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            Error::InvalidArgument(format!("Instance not found: {}", instance_id))
        })?;

        instance.metadata.network_config = Some(config.clone());
        instance.last_seen = Instant::now();

        if let Some(metadata) = metadata_store.get_mut(instance_id) {
            metadata.network_config = Some(config);
        }

        tracing::debug!("Set network config for instance {}", instance_id);

        Ok(())
    }

    pub async fn get_instance(&self, instance_id: &str) -> Result<Option<InstanceInfo>> {
        let instances = self.instances.read().await;
        Ok(instances.get(instance_id).cloned())
    }

    pub async fn list_instances(&self) -> Result<Vec<InstanceInfo>> {
        let instances = self.instances.read().await;
        Ok(instances.values().cloned().collect())
    }

    pub async fn cleanup_old_instances(&self, max_age: Duration) -> Result<usize> {
        let mut instances = self.instances.write().await;
        let mut metadata_store = self.metadata_store.write().await;

        let now = Instant::now();
        let mut removed = 0;

        let old_ids: Vec<String> = instances
            .iter()
            .filter(|(_, instance)| now.duration_since(instance.last_seen) > max_age)
            .map(|(id, _)| id.clone())
            .collect();

        for id in old_ids {
            instances.remove(&id);
            metadata_store.remove(&id);
            removed += 1;
        }

        if removed > 0 {
            tracing::info!("Cleaned up {} old cloud-init instances", removed);
        }

        Ok(removed)
    }

    pub async fn get_stats(&self) -> Result<CloudInitState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    pub async fn generate_default_network_config(&self) -> NetworkConfig {
        NetworkConfig {
            version: 2,
            config: vec![NetworkInterface {
                name: "eth0".to_string(),
                r#type: InterfaceType::Physical,
                mac_address: None,
                addresses: vec![IpAddress {
                    address: "192.168.1.100".to_string(),
                    netmask: "24".to_string(),
                    gateway: Some("192.168.1.1".to_string()),
                }],
                gateway: Some("192.168.1.1".to_string()),
                dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            }],
        }
    }
}
