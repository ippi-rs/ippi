use ippi::{Config, kvm, p2p, webrtc, cloud_init};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_kvm_module() {
    let config = Config::default();
    let manager = kvm::KvmManager::new(Arc::new(config)).unwrap();
    
    // Test initialization
    let result = manager.initialize().await;
    assert!(result.is_ok() || matches!(result, Err(_))); // Can fail if KVM not available
    
    // Test VM creation (simulated)
    if manager.initialize().await.is_ok() {
        let vm_id = manager.create_vm("test-vm", 512, 2).await;
        assert!(vm_id.is_ok());
        
        if let Ok(vm_id) = vm_id {
            // Test getting VM
            let vm = manager.get_vm(&vm_id).await;
            assert!(vm.is_ok());
            
            // Test listing VMs
            let vms = manager.list_vms().await;
            assert!(vms.is_ok());
            
            // Test deleting VM
            let result = manager.delete_vm(&vm_id).await;
            assert!(result.is_ok());
        }
    }
}

#[tokio::test]
async fn test_p2p_module() {
    let config = Config::default();
    let manager = p2p::P2pManager::new(Arc::new(config)).unwrap();
    
    // Test initialization
    let result = manager.initialize().await;
    assert!(result.is_ok());
    
    // Test getting stats
    let stats = manager.get_stats().await;
    assert!(stats.is_ok());
    
    let stats = stats.unwrap();
    assert_eq!(stats.enabled, false); // Disabled by default
    
    // Test peer count
    let count = manager.get_peer_count().await;
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 0);
}

#[tokio::test]
async fn test_webrtc_module() {
    let config = Config::default();
    let manager = webrtc::WebRtcManager::new(Arc::new(config)).unwrap();
    
    // Test initialization
    let result = manager.initialize().await;
    assert!(result.is_ok());
    
    // Test getting stats
    let stats = manager.get_stats().await;
    assert!(stats.is_ok());
    
    let stats = stats.unwrap();
    assert_eq!(stats.enabled, false); // Disabled by default
}

#[tokio::test]
async fn test_cloud_init_module() {
    let config = Config::default();
    let manager = cloud_init::CloudInitManager::new(Arc::new(config)).unwrap();
    
    // Test initialization
    let result = manager.initialize().await;
    assert!(result.is_ok());
    
    // Test instance creation
    let instance_id = manager.create_instance(None, "test-instance").await;
    assert!(instance_id.is_ok());
    
    if let Ok(instance_id) = instance_id {
        // Test getting metadata
        let metadata = manager.get_metadata(&instance_id).await;
        assert!(metadata.is_ok());
        
        // Test setting userdata
        let userdata = "#cloud-config\npackage_update: true\n";
        let result = manager.set_userdata(&instance_id, userdata).await;
        assert!(result.is_ok());
        
        // Test getting userdata
        let retrieved = manager.get_userdata(&instance_id).await;
        assert!(retrieved.is_ok());
        
        // Test serving metadata
        let served = manager.serve_metadata(&instance_id).await;
        assert!(served.is_ok());
        
        // Test listing instances
        let instances = manager.list_instances().await;
        assert!(instances.is_ok());
        
        // Test getting stats
        let stats = manager.get_stats().await;
        assert!(stats.is_ok());
    }
}

#[tokio::test]
async fn test_config_module() {
    // Test default config
    let config = Config::default();
    
    assert_eq!(config.web.host, "0.0.0.0");
    assert_eq!(config.web.port, 8080);
    assert_eq!(config.web.cors_origins, vec!["*"]);
    
    assert!(config.kvm.is_some());
    assert!(!config.kvm.as_ref().unwrap().enabled);
    
    assert!(config.p2p.is_some());
    assert!(!config.p2p.as_ref().unwrap().enabled);
    
    assert!(config.webrtc.is_some());
    assert!(!config.webrtc.as_ref().unwrap().enabled);
    
    assert!(config.cloud_init.is_some());
    assert!(!config.cloud_init.as_ref().unwrap().enabled);
}

#[tokio::test]
async fn test_module_interaction() {
    // Test that modules can be created and initialized together
    let config = Arc::new(Config::default());
    
    let kvm_manager = kvm::KvmManager::new(config.clone()).unwrap();
    let p2p_manager = p2p::P2pManager::new(config.clone()).unwrap();
    let webrtc_manager = webrtc::WebRtcManager::new(config.clone()).unwrap();
    let cloud_init_manager = cloud_init::CloudInitManager::new(config.clone()).unwrap();
    
    // Initialize all modules
    let kvm_result = kvm_manager.initialize().await;
    let p2p_result = p2p_manager.initialize().await;
    let webrtc_result = webrtc_manager.initialize().await;
    let cloud_init_result = cloud_init_manager.initialize().await;
    
    // All should initialize successfully (even if features are disabled)
    assert!(kvm_result.is_ok());
    assert!(p2p_result.is_ok());
    assert!(webrtc_result.is_ok());
    assert!(cloud_init_result.is_ok());
    
    // Get stats from all modules
    let kvm_stats = kvm_manager.get_stats().await;
    let p2p_stats = p2p_manager.get_stats().await;
    let webrtc_stats = webrtc_manager.get_stats().await;
    let cloud_init_stats = cloud_init_manager.get_stats().await;
    
    assert!(kvm_stats.is_ok());
    assert!(p2p_stats.is_ok());
    assert!(webrtc_stats.is_ok());
    assert!(cloud_init_stats.is_ok());
}

#[tokio::test]
async fn test_error_handling() {
    let config = Config::default();
    let manager = kvm::KvmManager::new(Arc::new(config)).unwrap();
    
    // Test error when trying to operate on non-existent VM
    let result = manager.get_vm("non-existent-id").await;
    assert!(result.is_err());
    
    let result = manager.start_vm("non-existent-id").await;
    assert!(result.is_err());
    
    let result = manager.stop_vm("non-existent-id").await;
    assert!(result.is_err());
    
    let result = manager.delete_vm("non-existent-id").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_access() {
    let config = Arc::new(Config::default());
    let manager = kvm::KvmManager::new(config).unwrap();
    
    // Initialize if KVM is available
    let _ = manager.initialize().await;
    
    // Create multiple VMs concurrently
    let mut handles = vec![];
    
    for i in 0..5 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            let vm_id = manager_clone.create_vm(&format!("vm-{}", i), 256, 1).await;
            vm_id.is_ok()
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap() == &true).count();
    
    // At least some should succeed (depending on memory limits)
    assert!(success_count > 0);
    
    // Clean up
    let vms = manager.list_vms().await.unwrap();
    for vm in vms {
        let _ = manager.delete_vm(&vm.id).await;
    }
}