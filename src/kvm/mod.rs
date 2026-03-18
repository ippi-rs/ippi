use crate::{Config, Error, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct KvmManager {
    config: Arc<Config>,
    state: Arc<RwLock<KvmState>>,
}

#[derive(Debug, Clone)]
pub struct KvmState {
    pub enabled: bool,
    pub vms: Vec<VmInstance>,
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
}

#[derive(Debug, Clone)]
pub struct VmInstance {
    pub id: String,
    pub name: String,
    pub memory_mb: u64,
    pub vcpus: u32,
    pub state: VmState,
    pub devices: Vec<VmDevice>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmState {
    Stopped,
    Running,
    Paused,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct VmDevice {
    pub device_type: DeviceType,
    pub path: String,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Disk,
    Cdrom,
    Network,
    Usb,
    Display,
}

impl KvmManager {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let state = KvmState {
            enabled: config.kvm.as_ref().map(|k| k.enabled).unwrap_or(false),
            vms: Vec::new(),
            total_memory_mb: config.kvm.as_ref().map(|k| k.memory_mb).unwrap_or(0),
            used_memory_mb: 0,
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.enabled {
            tracing::info!("KVM support is disabled in configuration");
            return Ok(());
        }

        #[cfg(feature = "kvm")]
        {
            let device_path = self
                .config
                .kvm
                .as_ref()
                .map(|k| k.device_path.as_str())
                .unwrap_or("/dev/kvm");

            if !Path::new(device_path).exists() {
                tracing::warn!("KVM device not found at {}", device_path);
                state.enabled = false;
                return Err(Error::Kvm(format!(
                    "KVM device not found at {}",
                    device_path
                )));
            }

            tracing::info!("KVM initialized successfully");
        }

        #[cfg(not(feature = "kvm"))]
        {
            tracing::warn!("KVM feature not enabled at compile time");
            state.enabled = false;
        }

        Ok(())
    }

    pub async fn create_vm(&self, name: &str, memory_mb: u64, vcpus: u32) -> Result<String> {
        let mut state = self.state.write().await;

        if !state.enabled {
            return Err(Error::Kvm("KVM is disabled".to_string()));
        }

        if state.used_memory_mb + memory_mb > state.total_memory_mb {
            return Err(Error::Kvm(format!(
                "Insufficient memory: requested {}MB, available {}MB",
                memory_mb,
                state.total_memory_mb - state.used_memory_mb
            )));
        }

        let id = uuid::Uuid::new_v4().to_string();

        let vm = VmInstance {
            id: id.clone(),
            name: name.to_string(),
            memory_mb,
            vcpus,
            state: VmState::Stopped,
            devices: Vec::new(),
        };

        state.vms.push(vm);
        state.used_memory_mb += memory_mb;

        tracing::info!(
            "Created VM {} (ID: {}) with {}MB RAM, {} vCPUs",
            name,
            id,
            memory_mb,
            vcpus
        );

        Ok(id)
    }

    pub async fn start_vm(&self, vm_id: &str) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.enabled {
            return Err(Error::Kvm("KVM is disabled".to_string()));
        }

        let vm = state
            .vms
            .iter_mut()
            .find(|vm| vm.id == vm_id)
            .ok_or_else(|| Error::Kvm(format!("VM not found: {}", vm_id)))?;

        vm.state = VmState::Running;

        tracing::info!("Started VM {} ({})", vm.name, vm_id);

        Ok(())
    }

    pub async fn stop_vm(&self, vm_id: &str) -> Result<()> {
        let mut state = self.state.write().await;

        let vm = state
            .vms
            .iter_mut()
            .find(|vm| vm.id == vm_id)
            .ok_or_else(|| Error::Kvm(format!("VM not found: {}", vm_id)))?;

        vm.state = VmState::Stopped;

        tracing::info!("Stopped VM {} ({})", vm.name, vm_id);

        Ok(())
    }

    pub async fn delete_vm(&self, vm_id: &str) -> Result<()> {
        let mut state = self.state.write().await;

        let index = state
            .vms
            .iter()
            .position(|vm| vm.id == vm_id)
            .ok_or_else(|| Error::Kvm(format!("VM not found: {}", vm_id)))?;

        let vm = state.vms.remove(index);
        state.used_memory_mb -= vm.memory_mb;

        tracing::info!("Deleted VM {} ({})", vm.name, vm_id);

        Ok(())
    }

    pub async fn add_disk(&self, vm_id: &str, path: &str, readonly: bool) -> Result<()> {
        let mut state = self.state.write().await;

        let vm = state
            .vms
            .iter_mut()
            .find(|vm| vm.id == vm_id)
            .ok_or_else(|| Error::Kvm(format!("VM not found: {}", vm_id)))?;

        let device = VmDevice {
            device_type: DeviceType::Disk,
            path: path.to_string(),
            readonly,
        };

        vm.devices.push(device);

        tracing::info!("Added disk to VM {} ({}): {}", vm.name, vm_id, path);

        Ok(())
    }

    pub async fn get_vm(&self, vm_id: &str) -> Result<VmInstance> {
        let state = self.state.read().await;

        let vm = state
            .vms
            .iter()
            .find(|vm| vm.id == vm_id)
            .ok_or_else(|| Error::Kvm(format!("VM not found: {}", vm_id)))?;

        Ok(vm.clone())
    }

    pub async fn list_vms(&self) -> Result<Vec<VmInstance>> {
        let state = self.state.read().await;
        Ok(state.vms.clone())
    }

    pub async fn get_stats(&self) -> Result<KvmState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }
}

#[cfg(feature = "kvm")]
mod vm {
    use super::*;
    use kvm_ioctls::{Kvm, VmFd};
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    pub struct VmHandle {
        pub vm_fd: VmFd,
        pub memory_size: usize,
    }

    impl VmHandle {
        pub fn new(memory_mb: u64) -> Result<Self> {
            let kvm = Kvm::new().map_err(|e| Error::Kvm(format!("Failed to open KVM: {}", e)))?;

            let vm_fd = kvm
                .create_vm()
                .map_err(|e| Error::Kvm(format!("Failed to create VM: {}", e)))?;

            let memory_size = (memory_mb * 1024 * 1024) as usize;

            // Setup memory region (simplified)
            // In a real implementation, you would use mmap and set_user_memory_region

            Ok(Self { vm_fd, memory_size })
        }

        pub fn add_vcpu(&self) -> Result<()> {
            // Simplified - real implementation would create VCPU
            Ok(())
        }

        pub fn run(&self) -> Result<()> {
            // Simplified - real implementation would run the VM
            Ok(())
        }
    }
}
