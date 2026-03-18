# Redes P2P en ippi

ippi implementa una red P2P completa para KVM-over-IP descentralizado, usando las siguientes librerías:

## 📚 Librerías P2P Implementadas

### 1. **libp2p** - Networking P2P Base
- **Versión**: 0.56
- **Features usadas**: `kad`, `noise`, `tcp`, `dns`, `websocket`, `yamux`, `ping`
- **Propósito**: Networking descentralizado con auto-descubrimiento
- **Componentes**:
  - **Kademlia DHT**: Descubrimiento automático de peers
  - **Noise Protocol**: Cifrado de comunicaciones
  - **TCP/WebSocket**: Transportes multiplataforma
  - **Yamux**: Multiplexación de streams

### 2. **snow** - Protocolo Noise
- **Versión**: 0.9
- **Propósito**: Implementación del protocolo Noise para cifrado
- **Integración**: Usado por libp2p para handshakes seguros
- **Características**:
  - Handshakes con forward secrecy
  - Autenticación mutua
  - Cifrado simétrico eficiente

### 3. **tun** - Tunnels TUN/TAP
- **Versión**: 0.8
- **Propósito**: Tunnels de red virtual para conectividad P2P
- **Características**:
  - Interfaces de red virtuales
  - MTU configurable
  - Soporte IPv4/IPv6
  - Integración con routing del sistema

### 4. **DHT Personalizado** - Almacenamiento Distribuido
- **Implementación**: Propia (sin dependencias externas)
- **Propósito**: Almacenamiento clave-valor distribuido
- **Características**:
  - Replicación automática (factor configurable)
  - TTL por valores
  - Bootstrap con nodos iniciales
  - Distancia XOR para routing

## 🏗️ Arquitectura de Red

```
┌─────────────────────────────────────────────────┐
│                 ippi P2P Network             │
├─────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ Peer A  │  │ Peer B  │  │ Peer C  │  ...    │
│  └─────────┘  └─────────┘  └─────────┘         │
│       │            │            │               │
│  ┌────┴────────────┴────────────┴───────────┐  │
│  │           Kademlia DHT (libp2p)          │  │
│  │        • Peer Discovery                  │  │
│  │        • Content Routing                 │  │
│  └──────────────────────────────────────────┘  │
│       │            │            │               │
│  ┌────┴────┐  ┌────┴────┐  ┌────┴────┐         │
│  │  Noise  │  │  Noise  │  │  Noise  │         │
│  │ Protocol│  │ Protocol│  │ Protocol│         │
│  └─────────┘  └─────────┘  └─────────┘         │
│       │            │            │               │
│  ┌────┴────┐  ┌────┴────┐  ┌────┴────┐         │
│  │  TCP/   │  │  TCP/   │  │  TCP/   │         │
│  │ WebSocket│  │ WebSocket│  │ WebSocket│         │
│  └─────────┘  └─────────┘  └─────────┘         │
│       │            │            │               │
│  ┌────┴────────────────────────────┴────┐      │
│  │          TUN/TAP Tunnels             │      │
│  │    • Virtual Network Interfaces      │      │
│  │    • NAT Traversal                   │      │
│  └──────────────────────────────────────┘      │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │         Distributed Storage (DHT)        │  │
│  │    • VM Metadata                         │  │
│  │    • Configuration                       │  │
│  │    • Peer Information                    │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## 🔧 Configuración

### Cargo.toml Features
```toml
[features]
p2p-full = ["libp2p", "snow", "tun"]  # Todas las features P2P

[dependencies]
libp2p = { version = "0.56", optional = true, features = ["kad", "noise", "tcp", "dns", "websocket", "yamux", "ping"] }
snow = { version = "0.9", optional = true }
tun = { version = "0.8", optional = true }
```

### Configuración TOML
```toml
[p2p]
enabled = true
bootstrap_nodes = [
    "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
]
listen_port = 4001
protocol_version = "/ippi/0.1.0"

[tunnel]
enabled = true
mtu = 1500
ipv4_enabled = true
ipv6_enabled = false

[dht]
enabled = true
replication_factor = 3
key_size = 256
value_ttl = 3600  # segundos
```

## 🚀 Uso

### Compilación
```bash
# Con todas las features P2P
cargo build --features "p2p-full"

# Solo networking básico
cargo build --features "libp2p"

# Para Raspberry Pi Zero
cargo build --target arm-unknown-linux-gnueabihf --features "p2p-full"
```

### Ejemplo de Código
```rust
use ippi::{Config, p2p, tunnel, dht};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuración
    let config = Arc::new(Config::default());
    
    // Inicializar P2P
    let p2p_manager = p2p::P2pManager::new(config.clone())?;
    p2p_manager.initialize().await?;
    p2p_manager.start().await?;
    
    // Inicializar TUN
    let tunnel_config = tunnel::TunnelConfig::default();
    let tunnel_manager = tunnel::TunnelManager::new(tunnel_config)?;
    
    // Inicializar DHT
    let dht_config = dht::DhtConfig::default();
    let dht_manager = dht::DhtManager::new(dht_config)?;
    dht_manager.bootstrap().await?;
    
    // Usar la red...
    Ok(())
}
```

## 🌐 Protocolos Implementados

### 1. **Descubrimiento de Peers**
- **Protocolo**: Kademlia DHT via libp2p
- **Método**: XOR distance-based routing
- **Bootstrap**: Nodos iniciales configurables
- **Mantenimiento**: Ping periódico, evicción de peers muertos

### 2. **Conexiones Seguras**
- **Protocolo**: Noise Protocol Framework
- **Handshake**: IK (Initiator Known) pattern
- **Cifrado**: ChaCha20Poly1305
- **Autenticación**: Ed25519 keypairs

### 3. **Tunnels de Red**
- **Tipo**: TUN (layer 3) / TAP (layer 2)
- **Configuración**: IP estático o DHCP
- **Routing**: Tablas de routing dinámicas
- **NAT Traversal**: Hole punching via STUN/TURN

### 4. **Almacenamiento Distribuido**
- **Estructura**: Tabla hash distribuida
- **Replicación**: Factor K configurable
- **Consistencia**: Eventual (valores con TTL)
- **Búsqueda**: Iterativa con paralelismo

## 🔒 Seguridad

### 1. **Autenticación**
- Keypairs Ed25519 por peer
- Identificadores derivados de claves públicas
- Whitelist/blacklist de peers

### 2. **Cifrado**
- Comunicaciones: Noise Protocol
- Almacenamiento DHT: Valores opcionalmente cifrados
- Tunnels: Cifrado punto-a-punto

### 3. **Privacidad**
- No logging de contenido por defecto
- IP masking via tunnels
- DHT con almacenamiento efímero

## 📊 Métricas y Monitoreo

### Métricas Recopiladas
- Número de peers conectados
- Latencia de red
- Throughput de tunnels
- Tamaño de DHT
- Tasa de replicación

### Health Checks
- Ping periódico a peers
- Verificación de tunnels
- Integridad de DHT
- Uso de recursos

## 🔄 Integración con Otros Módulos

### Con KVM
- Metadatos de VMs almacenados en DHT
- Streaming video via WebRTC sobre tunnels
- Control remoto via mensajes P2P

### Con Cloud-init
- Configuraciones distribuidas via DHT
- Plantillas de VMs replicadas
- Userdata cifrado

### Con WebRTC
- Signaling via mensajes P2P
- Media sobre tunnels TUN
- STUN/TURN integrado

## 🐛 Troubleshooting

### Problemas Comunes

1. **No se descubren peers**
   - Verificar bootstrap nodes
   - Checkear firewall (puerto 4001)
   - Verificar conectividad de red

2. **Tunnels no se conectan**
   - Verificar permisos (cap_net_admin)
   - Checkear configuración IP
   - Verificar routing

3. **DHT no almacena valores**
   - Verificar factor de replicación
   - Checkear TTL de valores
   - Verificar conectividad con peers

### Logging
```bash
# Nivel debug para P2P
RUST_LOG=ippi=debug,p2p=debug cargo run

# Log específico de libp2p
RUST_LOG=libp2p=debug cargo run
```

## 📈 Roadmap

### Fase 1 (Actual)
- [x] libp2p con Kademlia
- [x] Noise Protocol integration
- [x] TUN/TAP básico
- [x] DHT simple

### Fase 2 (Próxima)
- [ ] WebSocket transport para browsers
- [ ] NAT traversal automático
- [ ] DHT con persistencia
- [ ] Metrics y monitoring

### Fase 3 (Futuro)
- [ ] IPFS integration
- [ ] Blockchain para auditoría
- [ ] Federated identity
- [ ] QoS y traffic shaping

## 🔗 Recursos

- [libp2p Documentation](https://docs.libp2p.io/)
- [Noise Protocol Framework](http://noiseprotocol.org/)
- [TUN/TAP Linux](https://www.kernel.org/doc/html/latest/networking/tuntap.html)
- [Kademlia Paper](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)

---

*Última actualización: 2025-03-18*  
*ippi P2P Network - Lightweight KVM-over-IP para Raspberry Pi*