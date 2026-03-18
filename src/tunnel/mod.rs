use crate::{Error, Result};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "tun")]
mod tun_impl;

#[cfg(feature = "tun")]
pub use tun_impl::*;

#[derive(Debug, Clone)]
pub struct TunnelManager {
    tunnels: Arc<RwLock<Vec<TunnelInfo>>>,
    config: TunnelConfig,
}

#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub enabled: bool,
    pub mtu: u16,
    pub ipv4_enabled: bool,
    pub ipv6_enabled: bool,
    pub default_route: bool,
}

#[derive(Debug, Clone)]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub tunnel_type: TunnelType,
    pub local_ip: IpAddr,
    pub remote_ip: Option<IpAddr>,
    pub mtu: u16,
    pub status: TunnelStatus,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub created_at: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TunnelType {
    Tun,
    Tap,
    Wireguard,
    OpenVpn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TunnelStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl TunnelManager {
    pub fn new(config: TunnelConfig) -> Result<Self> {
        Ok(Self {
            tunnels: Arc::new(RwLock::new(Vec::new())),
            config,
        })
    }
    
    pub async fn create_tunnel(&self, name: &str, tunnel_type: TunnelType, local_ip: IpAddr) -> Result<String> {
        let mut tunnels = self.tunnels.write().await;
        
        if !self.config.enabled {
            return Err(Error::NotImplemented("Tunnels are disabled".to_string()));
        }
        
        let id = uuid::Uuid::new_v4().to_string();
        
        let tunnel = TunnelInfo {
            id: id.clone(),
            name: name.to_string(),
            tunnel_type,
            local_ip,
            remote_ip: None,
            mtu: self.config.mtu,
            status: TunnelStatus::Disconnected,
            bytes_sent: 0,
            bytes_received: 0,
            created_at: std::time::Instant::now(),
        };
        
        tunnels.push(tunnel);
        
        tracing::info!("Created tunnel {} (ID: {}) type: {:?}", name, id, tunnel_type);
        
        Ok(id)
    }
    
    pub async fn connect(&self, tunnel_id: &str, remote_ip: Option<IpAddr>) -> Result<()> {
        let mut tunnels = self.tunnels.write().await;
        
        let tunnel = tunnels.iter_mut()
            .find(|t| t.id == tunnel_id)
            .ok_or_else(|| Error::InvalidArgument(format!("Tunnel not found: {}", tunnel_id)))?;
        
        tunnel.status = TunnelStatus::Connecting;
        tunnel.remote_ip = remote_ip;
        
        #[cfg(feature = "tun")]
        {
            self.connect_tun_tunnel(tunnel).await?;
        }
        
        #[cfg(not(feature = "tun"))]
        {
            tracing::debug!("Simulating tunnel connection for {}", tunnel_id);
            tunnel.status = TunnelStatus::Connected;
        }
        
        tracing::info!("Connected tunnel {} ({})", tunnel.name, tunnel_id);
        
        Ok(())
    }
    
    pub async fn disconnect(&self, tunnel_id: &str) -> Result<()> {
        let mut tunnels = self.tunnels.write().await;
        
        let tunnel = tunnels.iter_mut()
            .find(|t| t.id == tunnel_id)
            .ok_or_else(|| Error::InvalidArgument(format!("Tunnel not found: {}", tunnel_id)))?;
        
        tunnel.status = TunnelStatus::Disconnected;
        
        #[cfg(feature = "tun")]
        {
            self.disconnect_tun_tunnel(tunnel).await?;
        }
        
        tracing::info!("Disconnected tunnel {} ({})", tunnel.name, tunnel_id);
        
        Ok(())
    }
    
    pub async fn send_packet(&self, tunnel_id: &str, packet: &[u8]) -> Result<()> {
        let mut tunnels = self.tunnels.write().await;
        
        let tunnel = tunnels.iter_mut()
            .find(|t| t.id == tunnel_id)
            .ok_or_else(|| Error::InvalidArgument(format!("Tunnel not found: {}", tunnel_id)))?;
        
        if tunnel.status != TunnelStatus::Connected {
            return Err(Error::InvalidArgument("Tunnel not connected".to_string()));
        }
        
        tunnel.bytes_sent += packet.len() as u64;
        
        #[cfg(feature = "tun")]
        {
            self.send_tun_packet(tunnel, packet).await?;
        }
        
        Ok(())
    }
    
    pub async fn receive_packet(&self, tunnel_id: &str) -> Result<Vec<u8>> {
        let mut tunnels = self.tunnels.write().await;
        
        let tunnel = tunnels.iter_mut()
            .find(|t| t.id == tunnel_id)
            .ok_or_else(|| Error::InvalidArgument(format!("Tunnel not found: {}", tunnel_id)))?;
        
        if tunnel.status != TunnelStatus::Connected {
            return Err(Error::InvalidArgument("Tunnel not connected".to_string()));
        }
        
        #[cfg(feature = "tun")]
        {
            let packet = self.receive_tun_packet(tunnel).await?;
            tunnel.bytes_received += packet.len() as u64;
            Ok(packet)
        }
        
        #[cfg(not(feature = "tun"))]
        {
            // Simulated packet
            let packet = vec![0u8; 64];
            tunnel.bytes_received += packet.len() as u64;
            Ok(packet)
        }
    }
    
    pub async fn get_tunnel(&self, tunnel_id: &str) -> Result<Option<TunnelInfo>> {
        let tunnels = self.tunnels.read().await;
        Ok(tunnels.iter().find(|t| t.id == tunnel_id).cloned())
    }
    
    pub async fn list_tunnels(&self) -> Result<Vec<TunnelInfo>> {
        let tunnels = self.tunnels.read().await;
        Ok(tunnels.clone())
    }
    
    pub async fn delete_tunnel(&self, tunnel_id: &str) -> Result<()> {
        let mut tunnels = self.tunnels.write().await;
        
        let index = tunnels.iter()
            .position(|t| t.id == tunnel_id)
            .ok_or_else(|| Error::InvalidArgument(format!("Tunnel not found: {}", tunnel_id)))?;
        
        let tunnel = tunnels.remove(index);
        
        // Disconnect if connected
        if tunnel.status == TunnelStatus::Connected {
            #[cfg(feature = "tun")]
            {
                self.disconnect_tun_tunnel(&tunnel).await?;
            }
        }
        
        tracing::info!("Deleted tunnel {} ({})", tunnel.name, tunnel_id);
        
        Ok(())
    }
    
    pub async fn update_stats(&self, tunnel_id: &str, bytes_sent: u64, bytes_received: u64) -> Result<()> {
        let mut tunnels = self.tunnels.write().await;
        
        let tunnel = tunnels.iter_mut()
            .find(|t| t.id == tunnel_id)
            .ok_or_else(|| Error::InvalidArgument(format!("Tunnel not found: {}", tunnel_id)))?;
        
        tunnel.bytes_sent = bytes_sent;
        tunnel.bytes_received = bytes_received;
        
        Ok(())
    }
    
    pub fn get_config(&self) -> &TunnelConfig {
        &self.config
    }
    
    pub async fn get_total_stats(&self) -> Result<(u64, u64)> {
        let tunnels = self.tunnels.read().await;
        
        let total_sent: u64 = tunnels.iter().map(|t| t.bytes_sent).sum();
        let total_received: u64 = tunnels.iter().map(|t| t.bytes_received).sum();
        
        Ok((total_sent, total_received))
    }
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mtu: 1500,
            ipv4_enabled: true,
            ipv6_enabled: false,
            default_route: false,
        }
    }
}

#[cfg(feature = "tun")]
mod tun_impl {
    use super::*;
    use tun::{Configuration, Device};
    
    impl TunnelManager {
        async fn connect_tun_tunnel(&self, tunnel: &mut TunnelInfo) -> Result<()> {
            let mut config = Configuration::default();
            
            config.name("kvmdust-tun")
                .address(tunnel.local_ip.to_string())
                .mtu(tunnel.mtu as i32)
                .up();
            
            if let Some(remote_ip) = tunnel.remote_ip {
                config.destination(remote_ip.to_string());
            }
            
            // Create TUN device
            let device = tun::create(&config)
                .map_err(|e| Error::InvalidArgument(format!("Failed to create TUN device: {}", e)))?;
            
            // Store device handle (in real implementation)
            // For now, just mark as connected
            
            tunnel.status = TunnelStatus::Connected;
            
            Ok(())
        }
        
        async fn disconnect_tun_tunnel(&self, tunnel: &mut TunnelInfo) -> Result<()> {
            // In real implementation, close the TUN device
            tunnel.status = TunnelStatus::Disconnected;
            Ok(())
        }
        
        async fn send_tun_packet(&self, tunnel: &mut TunnelInfo, packet: &[u8]) -> Result<()> {
            // In real implementation, write to TUN device
            tracing::debug!("Sending {} bytes through tunnel {}", packet.len(), tunnel.id);
            Ok(())
        }
        
        async fn receive_tun_packet(&self, tunnel: &mut TunnelInfo) -> Result<Vec<u8>> {
            // In real implementation, read from TUN device
            // Simulated packet for now
            Ok(vec![0u8; 64])
        }
    }
}