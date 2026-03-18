# ippi Architecture

## Overview

ippi is a lightweight, P2P KVM-over-IP solution written in Rust, designed specifically for Raspberry Pi Zero devices. This document describes the system architecture, components, and design decisions.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
├─────────────────────────────────────────────────────────────┤
│  Web Interface (Svelte)  │  REST API (Axum)  │  WebSocket   │
└──────────────────────────┴───────────────────┴──────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                      Service Layer                           │
├─────────────────────────────────────────────────────────────┤
│  WebRTC Bridge  │  VM Manager  │  P2P Network  │  Cloud-init │
└─────────────────┴──────────────┴───────────────┴─────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                      Core Layer                              │
├─────────────────────────────────────────────────────────────┤
│  KVM Hypervisor  │  VirtIO Devices  │  Hardware Abstraction  │
└──────────────────┴──────────────────┴────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                      Platform Layer                          │
├─────────────────────────────────────────────────────────────┤
│  Linux KVM  │  USB Gadget  │  GPIO  │  Network Stack         │
└─────────────┴──────────────┴────────┴────────────────────────┘
```

## Component Details

### 1. Web Interface (Frontend)
- **Technology**: Svelte + Vite + TypeScript
- **Build**: Compiled to static assets, embedded in Rust binary
- **Features**:
  - Live video stream via WebRTC
  - Virtual keyboard/mouse input
  - VM control panel
  - File upload for ISOs
  - System status dashboard
  - P2P network visualization

### 2. Web Server (Backend)
- **Technology**: Axum (async web framework)
- **Responsibilities**:
  - Serve static frontend assets
  - Provide REST API for VM management
  - Handle WebSocket connections for real-time updates
  - Manage WebRTC signaling
  - Authentication and authorization

### 3. WebRTC Bridge
- **Technology**: `webrtc` crate + custom video encoding
- **Components**:
  - **Video Capture**: Frame buffer capture from KVM
  - **Video Encoding**: VP8/H.264 encoding for WebRTC
  - **Audio Bridge**: Optional audio passthrough
  - **Input Forwarding**: Mouse/keyboard events to KVM
  - **NAT Traversal**: ICE/STUN/TURN integration

### 4. KVM Hypervisor
- **Technology**: `kvm-ioctls` + `vm-virtio` crates
- **Components**:
  - **VM Lifecycle**: Create, start, stop, pause VMs
  - **Memory Management**: Memory allocation and mapping
  - **CPU Virtualization**: vCPU creation and scheduling
  - **Device Emulation**: VirtIO devices (net, block, console)
  - **Snapshot Support**: VM state save/restore

### 5. P2P Network
- **Technology**: Libp2p with Kademlia DHT
- **Components**:
  - **Discovery**: Find other ippi nodes
  - **NAT Traversal**: Hole punching via relay nodes
  - **Service Discovery**: Advertise KVM services
  - **Secure Communication**: Noise protocol encryption
  - **Bootstrapping**: Connect to known peers

### 6. Cloud-init Service
- **Technology**: Custom implementation + `serde_yaml`
- **Features**:
  - Parse cloud-config YAML
  - Configure network (netplan/systemd-networkd)
  - Create users and SSH keys
  - Run initialization scripts
  - Inject configuration into VM

### 7. PXE/iPXE Server
- **Technology**: `dhcproto` + `tftp` crates
- **Components**:
  - **DHCP Server**: Provide IP addresses and boot info
  - **TFTP Server**: Serve boot files
  - **HTTP Server**: Serve kernel/initrd/images
  - **iPXE Scripting**: Dynamic boot scripts
  - **NBD Server**: Network block device for diskless boot

### 8. Hardware Abstraction
- **Technology**: Platform-specific crates (`rppal`, `rusb`, `hidapi`)
- **Components**:
  - **USB Gadget Mode**: OTG configuration
  - **GPIO Control**: Raspberry Pi pins
  - **HID Emulation**: Keyboard/mouse simulation
  - **Video Capture**: HDMI/CSI bridge support
  - **Power Management**: ATX control

## Data Flow

### 1. Video Streaming
```
KVM Frame Buffer → Video Encoder → WebRTC Peer → Browser
      ↓               ↓               ↓           ↓
   Raw frames     Compressed      Encrypted    Decoded &
                  (VP8/H.264)     (SRTP)      displayed
```

### 2. Input Forwarding
```
Browser → WebSocket → Input Parser → USB HID → KVM Guest
   ↓         ↓           ↓           ↓           ↓
Keyboard/  JSON       Event       USB        Guest OS
Mouse     message   translation  packet     receives
events                           (via OTG)   input
```

### 3. P2P Discovery
```
Node Start → Bootstrap → DHT Join → Advertise → Discover Peers
    ↓           ↓           ↓          ↓            ↓
Local IP   Connect to   Join Kademlia  Publish   Find other
           known peers   network      service    ippi
                                        info      nodes
```

### 4. VM Provisioning
```
User Request → Cloud-init → Network Config → VM Start → Guest Boot
     ↓            ↓             ↓             ↓           ↓
Create VM    Parse YAML    Configure IP   Launch VM   OS boots
             config        and routing    with config  with cloud
                                           injected    settings
```

## Concurrency Model

### Async/Await with Tokio
```
Main Runtime (Tokio)
├── Web Server Task
│   ├── HTTP Request Handler
│   ├── WebSocket Handler
│   └── Static File Server
├── VM Manager Task
│   ├── KVM Event Loop
│   ├── Device I/O
│   └── Snapshot Manager
├── P2P Network Task
│   ├── DHT Maintenance
│   ├── Peer Discovery
│   └── Message Routing
└── WebRTC Task
    ├── Video Encoding
    ├── Audio Processing
    └── ICE Connection
```

### Thread Safety
- **Shared State**: `Arc<Mutex<T>>` or `Arc<RwLock<T>>`
- **Message Passing**: `tokio::sync::mpsc` channels
- **CPU-bound tasks**: `tokio::task::spawn_blocking`
- **I/O-bound tasks**: Async I/O with Tokio

## Configuration Management

### Hierarchical Configuration
```
Command Line Args → Environment Vars → Config File → Defaults
      ↓                  ↓                 ↓           ↓
  Highest           High Priority      File-based   Fallback
  Priority                              settings     values
```

### Configuration Sources
1. **Command Line**: `--config`, `--port`, `--host`
2. **Environment**: `ippi_PORT`, `ippi_HOST`
3. **Config File**: YAML format, multiple locations
4. **Defaults**: Built-in safe defaults

## Error Handling

### Error Hierarchy
```
ippiError (base)
├── ConfigError
│   ├── ParseError
│   └── ValidationError
├── VmError
│   ├── CreateError
│   ├── StartError
│   └── DeviceError
├── NetworkError
│   ├── ConnectionError
│   ├── TimeoutError
│   └── ProtocolError
├── WebError
│   ├── ApiError
│   ├── AuthError
│   └── WebSocketError
└── HardwareError
    ├── UsbError
    ├── GpioError
    └── VideoError
```

### Error Recovery
- **Transient errors**: Automatic retry with exponential backoff
- **Configuration errors**: Fallback to defaults with warnings
- **Hardware errors**: Graceful degradation
- **Fatal errors**: Clean shutdown with error reporting

## Security Considerations

### 1. Authentication & Authorization
- **API Keys**: HMAC-based authentication for REST API
- **WebSocket Tokens**: JWT for WebSocket connections
- **Role-based Access**: Different permissions for admin/users
- **Rate Limiting**: Prevent brute force attacks

### 2. Network Security
- **TLS/HTTPS**: Encrypted web traffic
- **Noise Protocol**: Encrypted P2P communications
- **Certificate Pinning**: Prevent MITM attacks
- **Firewall Rules**: Default deny, explicit allow

### 3. VM Isolation
- **Namespaces**: Network and PID namespaces
- **Cgroups**: Resource limits and isolation
- **Seccomp**: System call filtering
- **SELinux/AppArmor**: Mandatory access control

### 4. Data Protection
- **Encryption at rest**: Encrypted disk images
- **Secure Erase**: Proper deletion of sensitive data
- **Audit Logging**: Security event logging
- **Input Validation**: Prevent injection attacks

## Performance Optimization

### Memory Optimization
- **Arena Allocation**: For frequently allocated objects
- **Object Pooling**: Reuse expensive objects
- **Zero-copy**: Avoid unnecessary data copying
- **Compact Data Structures**: Minimize memory footprint

### CPU Optimization
- **SIMD Instructions**: For video encoding/decoding
- **CPU Affinity**: Pin tasks to specific cores
- **Batch Processing**: Process multiple items at once
- **Lazy Evaluation**: Defer computation until needed

### I/O Optimization
- **Zero-copy Networking**: Use `sendfile` and `splice`
- **Async I/O**: Non-blocking operations
- **Buffer Pooling**: Reuse I/O buffers
- **Batched Writes**: Group small writes

## Deployment Considerations

### Single Binary Deployment
```
ippi binary (8-15MB)
├── Embedded Frontend (200-500KB)
├── Configuration (YAML)
├── Logs (rotated)
└── Data Directory
    ├── VM disks
    ├── ISOs
    ├── Snapshots
    └── Cloud-init configs
```

### Resource Requirements
| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 128MB | 512MB |
| Storage | 8GB | 16GB+ |
| CPU | 1 core | 4 cores |
| Network | 10 Mbps | 100 Mbps+ |

### Monitoring
- **Metrics**: Prometheus metrics endpoint
- **Logging**: Structured JSON logs
- **Health Checks**: HTTP health endpoint
- **Alerting**: Integration with monitoring systems

## Future Extensions

### Planned Features
1. **Cluster Mode**: Multiple nodes working together
2. **Live Migration**: Move VMs between hosts
3. **GPU Passthrough**: Direct GPU access
4. **Container Support**: Run containers alongside VMs
5. **Plugin System**: Extensible architecture

### Integration Points
1. **Terraform Provider**: Infrastructure as code
2. **Kubernetes CSI**: Storage integration
3. **Prometheus Exporters**: Custom metrics
4. **Grafana Dashboards**: Monitoring visualization
5. **Webhook Support**: Event notifications

## Design Principles

1. **Simplicity**: Do one thing well
2. **Performance**: Optimize for Raspberry Pi Zero
3. **Security**: Secure by default
4. **Reliability**: Handle failures gracefully
5. **Maintainability**: Clean, documented code
6. **Extensibility**: Easy to add new features

## Decision Records

Key architectural decisions are documented in [docs/decisions/](docs/decisions/):
- [ADR-001: Rust over Python](docs/decisions/ADR-001-rust-over-python.md)
- [ADR-002: Axum web framework](docs/decisions/ADR-002-axum-web-framework.md)
- [ADR-003: Libp2p for P2P](docs/decisions/ADR-003-libp2p-p2p.md)
- [ADR-004: Embedded frontend](docs/decisions/ADR-004-embedded-frontend.md)