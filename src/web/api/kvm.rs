use crate::web::AppState;
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vms", get(list_vms).post(create_vm))
        .route("/vms/{vm_id}", get(get_vm).delete(delete_vm))
        .route("/vms/{vm_id}/start", post(start_vm))
        .route("/vms/{vm_id}/stop", post(stop_vm))
        .route("/vms/{vm_id}/disks", post(add_disk))
        .route("/stats", get(get_stats))
}

async fn list_vms(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.list_vms().await {
                Ok(vms) => {
                    let vm_json: Vec<_> = vms.into_iter().map(|vm| serde_json::json!({
                        "id": vm.id,
                        "name": vm.name,
                        "memory_mb": vm.memory_mb,
                        "vcpus": vm.vcpus,
                        "state": format!("{:?}", vm.state),
                        "devices": vm.devices.len(),
                    })).collect();
                    return Json(serde_json::json!({ "vms": vm_json }));
                }
                Err(e) => {
                    tracing::warn!("Failed to list VMs: {}", e);
                }
            }
        }
    }
    
    Json(serde_json::json!({ "vms": [] }))
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CreateVmRequest {
    name: String,
    memory_mb: u64,
    vcpus: u32,
}

async fn create_vm(
    State(state): State<AppState>,
    Json(payload): Json<CreateVmRequest>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.create_vm(&payload.name, payload.memory_mb, payload.vcpus).await {
                Ok(vm_id) => {
                    return Json(serde_json::json!({
                        "id": vm_id,
                        "name": payload.name,
                        "memory_mb": payload.memory_mb,
                        "vcpus": payload.vcpus,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to create VM: {}", e);
                    return Json(serde_json::json!({
                        "error": format!("Failed to create VM: {}", e),
                    }));
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "error": "KVM not enabled or manager not available",
    }))
}

async fn get_vm(
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> axum::Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.get_vm(&vm_id).await {
                Ok(vm) => {
                    return Json(serde_json::json!({
                        "id": vm.id,
                        "name": vm.name,
                        "memory_mb": vm.memory_mb,
                        "vcpus": vm.vcpus,
                        "state": format!("{:?}", vm.state),
                        "devices": vm.devices.iter().map(|d| serde_json::json!({
                            "type": format!("{:?}", d.device_type),
                            "path": d.path,
                            "readonly": d.readonly,
                        })).collect::<Vec<_>>(),
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to get VM {}: {}", vm_id, e);
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "error": format!("VM {} not found", vm_id),
    }))
}

async fn delete_vm(
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.delete_vm(&vm_id).await {
                Ok(_) => {
                    return Json(serde_json::json!({
                        "deleted": vm_id,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to delete VM {}: {}", vm_id, e);
                    return Json(serde_json::json!({
                        "error": format!("Failed to delete VM: {}", e),
                    }));
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "error": "KVM not enabled",
    }))
}

async fn start_vm(
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.start_vm(&vm_id).await {
                Ok(_) => {
                    return Json(serde_json::json!({
                        "started": vm_id,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to start VM {}: {}", vm_id, e);
                    return Json(serde_json::json!({
                        "error": format!("Failed to start VM: {}", e),
                    }));
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "error": "KVM not enabled",
    }))
}

async fn stop_vm(
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.stop_vm(&vm_id).await {
                Ok(_) => {
                    return Json(serde_json::json!({
                        "stopped": vm_id,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to stop VM {}: {}", vm_id, e);
                    return Json(serde_json::json!({
                        "error": format!("Failed to stop VM: {}", e),
                    }));
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "error": "KVM not enabled",
    }))
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AddDiskRequest {
    path: String,
    readonly: Option<bool>,
}

async fn add_disk(
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
    Json(payload): Json<AddDiskRequest>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.add_disk(&vm_id, &payload.path, payload.readonly.unwrap_or(false)).await {
                Ok(_) => {
                    return Json(serde_json::json!({
                        "added": vm_id,
                        "path": payload.path,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to add disk to VM {}: {}", vm_id, e);
                    return Json(serde_json::json!({
                        "error": format!("Failed to add disk: {}", e),
                    }));
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "error": "KVM not enabled",
    }))
}

async fn get_stats(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "kvm")]
    {
        if let Some(manager) = &state.kvm_manager {
            match manager.get_stats().await {
                Ok(stats) => {
                    return Json(serde_json::json!({
                        "enabled": stats.enabled,
                        "vms": stats.vms.len(),
                        "total_memory_mb": stats.total_memory_mb,
                        "used_memory_mb": stats.used_memory_mb,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to get KVM stats: {}", e);
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "enabled": false,
        "vms": 0,
        "total_memory_mb": 0,
        "used_memory_mb": 0,
    }))
}
