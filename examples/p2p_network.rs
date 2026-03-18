//! Ejemplo completo de red P2P con IPPI
//! 
//! Este ejemplo muestra cómo usar:
//! 1. libp2p para networking P2P
//! 2. Snow para protocolo Noise
//! 3. TUN/TAP para tunnels de red
//! 4. DHT para almacenamiento distribuido

use ippi::{Config, p2p, tunnel, dht};
use std::sync::Arc;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== IPPI P2P Network Example ===");
    println!("");
    
    // Configuración básica
    let config = Arc::new(Config::default());
    
    println!("1. Inicializando módulo P2P...");
    let p2p_manager = p2p::P2pManager::new(config.clone())?;
    p2p_manager.initialize().await?;
    
    // Intentar iniciar si está habilitado
    if let Ok(stats) = p2p_manager.get_stats().await {
        if stats.enabled {
            println!("   P2P habilitado, iniciando red...");
            p2p_manager.start().await?;
            
            // Descubrir peers
            println!("   Descubriendo peers en la red...");
            let peers = p2p_manager.discover_peers().await?;
            println!("   Encontrados {} peers", peers.len());
            
            for peer in peers {
                println!("   - Peer: {} (conectado desde {:?})", 
                    peer.peer_id, peer.connected_since);
            }
        } else {
            println!("   P2P deshabilitado en configuración");
        }
    }
    
    println!("");
    println!("2. Inicializando módulo TUN/TAP...");
    let tunnel_config = tunnel::TunnelConfig {
        enabled: true,
        mtu: 1500,
        ipv4_enabled: true,
        ipv6_enabled: false,
        default_route: false,
    };
    
    let tunnel_manager = tunnel::TunnelManager::new(tunnel_config)?;
    
    // Crear un tunnel TUN
    let local_ip = IpAddr::from_str("10.0.0.1")?;
    let tunnel_id = tunnel_manager.create_tunnel("ippi-tun0", 
        tunnel::TunnelType::Tun, local_ip).await?;
    
    println!("   Tunnel creado: {} (TUN)", tunnel_id);
    
    // Conectar el tunnel (simulado sin feature tun)
    println!("   Conectando tunnel...");
    let remote_ip = IpAddr::from_str("10.0.0.2")?;
    tunnel_manager.connect(&tunnel_id, Some(remote_ip)).await?;
    
    // Obtener información del tunnel
    if let Some(tunnel_info) = tunnel_manager.get_tunnel(&tunnel_id).await? {
        println!("   Tunnel conectado: {} -> {:?}", 
            tunnel_info.local_ip, tunnel_info.remote_ip);
        println!("   Estado: {:?}, MTU: {}", tunnel_info.status, tunnel_info.mtu);
    }
    
    println!("");
    println!("3. Inicializando módulo DHT...");
    let dht_config = dht::DhtConfig {
        enabled: true,
        replication_factor: 3,
        key_size: 256,
        value_ttl: Duration::from_secs(3600),
        bootstrap_nodes: vec![
            "QmExample1@/ip4/104.131.131.82/tcp/4001".to_string(),
            "QmExample2@/ip4/104.131.131.82/tcp/4001".to_string(),
        ],
        protocol_version: "/ippi-dht/1.0.0".to_string(),
    };
    
    let dht_manager = dht::DhtManager::new(dht_config)?;
    
    // Bootstrap DHT
    println!("   Haciendo bootstrap de DHT...");
    dht_manager.bootstrap().await?;
    
    // Agregar algunos peers de ejemplo
    println!("   Agregando peers de ejemplo...");
    dht_manager.add_peer("QmPeer1", "/ip4/192.168.1.100/tcp/4001").await?;
    dht_manager.add_peer("QmPeer2", "/ip4/192.168.1.101/tcp/4001").await?;
    dht_manager.add_peer("QmPeer3", "/ip4/192.168.1.102/tcp/4001").await?;
    
    // Almacenar un valor en DHT
    println!("   Almacenando valor en DHT...");
    let key = "config:vm:ubuntu-server";
    let value = b"{\"memory\": 1024, \"cpus\": 2, \"disk\": \"20G\"}";
    
    dht_manager.put(key, value, "local-peer").await?;
    println!("   Valor almacenado con key: {}", key);
    
    // Recuperar valor de DHT
    println!("   Recuperando valor de DHT...");
    if let Some(retrieved_value) = dht_manager.get(key).await? {
        println!("   Valor recuperado: {} bytes", retrieved_value.len());
        println!("   Contenido: {}", String::from_utf8_lossy(&retrieved_value));
    } else {
        println!("   Valor no encontrado en DHT");
    }
    
    // Obtener estadísticas
    println!("   Obteniendo estadísticas DHT...");
    let stats = dht_manager.get_stats().await?;
    println!("   Total valores: {}", stats.total_values);
    println!("   Total peers: {}", stats.total_peers);
    println!("   Factor de replicación: {}", stats.replication_factor);
    
    println!("");
    println!("4. Integración P2P + TUN + DHT...");
    println!("   En un escenario real:");
    println!("   - Los peers se descubren via libp2p/Kademlia");
    println!("   - Se establecen tunnels TUN entre peers");
    println!("   - Los metadatos de VMs se almacenan en DHT");
    println!("   - El streaming de video usa WebRTC sobre los tunnels");
    
    println!("");
    println!("5. Limpiando...");
    
    // Desconectar tunnel
    tunnel_manager.disconnect(&tunnel_id).await?;
    println!("   Tunnel desconectado");
    
    // Eliminar valor DHT
    dht_manager.delete(key).await?;
    println!("   Valor DHT eliminado");
    
    // Limpiar valores expirados
    let cleaned = dht_manager.cleanup_expired().await?;
    println!("   Valores DHT expirados limpiados: {}", cleaned);
    
    println!("");
    println!("=== Ejemplo completado ===");
    println!("");
    println!("Para usar las features completas, compila con:");
    println!("  cargo build --features \"p2p-full\"");
    println!("");
    println("Features disponibles:");
    println!("  - libp2p: Networking P2P con Kademlia DHT");
    println!("  - snow: Protocolo Noise para cifrado");
    println!("  - tun: Tunnels TUN/TAP para red virtual");
    println!("  - dht: Almacenamiento distribuido de clave-valor");
    
    Ok(())
}