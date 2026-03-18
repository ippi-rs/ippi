use crate::{Error, Result};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct DhtManager {
    store: Arc<RwLock<HashMap<String, DhtValue>>>,
    peers: Arc<RwLock<HashMap<String, DhtPeer>>>,
    config: DhtConfig,
}

#[derive(Debug, Clone)]
pub struct DhtConfig {
    pub enabled: bool,
    pub replication_factor: usize,
    pub key_size: usize,
    pub value_ttl: Duration,
    pub bootstrap_nodes: Vec<String>,
    pub protocol_version: String,
}

#[derive(Debug, Clone)]
pub struct DhtValue {
    pub key: String,
    pub value: Vec<u8>,
    pub timestamp: Instant,
    pub ttl: Duration,
    pub publisher: String,
    pub replicas: Vec<String>, // Peer IDs holding replicas
}

#[derive(Debug, Clone)]
pub struct DhtPeer {
    pub peer_id: String,
    pub address: String,
    pub last_seen: Instant,
    pub distance: u32, // XOR distance from local peer
    pub metadata: HashMap<String, String>,
}

impl DhtManager {
    pub fn new(config: DhtConfig) -> Result<Self> {
        Ok(Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }
    
    pub async fn put(&self, key: &str, value: &[u8], publisher: &str) -> Result<()> {
        if !self.config.enabled {
            return Err(Error::NotImplemented("DHT is disabled".to_string()));
        }
        
        if key.len() > self.config.key_size {
            return Err(Error::InvalidArgument(format!(
                "Key too long: {} > {}", 
                key.len(), 
                self.config.key_size
            )));
        }
        
        let mut store = self.store.write().await;
        
        // Find closest peers for replication
        let replicas = self.find_closest_peers(key, self.config.replication_factor).await;
        
        let dht_value = DhtValue {
            key: key.to_string(),
            value: value.to_vec(),
            timestamp: Instant::now(),
            ttl: self.config.value_ttl,
            publisher: publisher.to_string(),
            replicas,
        };
        
        store.insert(key.to_string(), dht_value);
        
        tracing::debug!("Stored DHT value for key: {} ({} bytes)", key, value.len());
        
        Ok(())
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if !self.config.enabled {
            return Err(Error::NotImplemented("DHT is disabled".to_string()));
        }
        
        let store = self.store.read().await;
        
        if let Some(value) = store.get(key) {
            // Check if value has expired
            if Instant::now().duration_since(value.timestamp) > value.ttl {
                tracing::debug!("DHT value expired for key: {}", key);
                return Ok(None);
            }
            
            Ok(Some(value.value.clone()))
        } else {
            // Try to find in network (simulated)
            let network_value = self.lookup_in_network(key).await?;
            Ok(network_value)
        }
    }
    
    pub async fn delete(&self, key: &str) -> Result<bool> {
        if !self.config.enabled {
            return Err(Error::NotImplemented("DHT is disabled".to_string()));
        }
        
        let mut store = self.store.write().await;
        
        if store.remove(key).is_some() {
            tracing::debug!("Deleted DHT value for key: {}", key);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn add_peer(&self, peer_id: &str, address: &str) -> Result<()> {
        let mut peers = self.peers.write().await;
        
        let distance = self.calculate_distance(peer_id);
        
        let peer = DhtPeer {
            peer_id: peer_id.to_string(),
            address: address.to_string(),
            last_seen: Instant::now(),
            distance,
            metadata: HashMap::new(),
        };
        
        peers.insert(peer_id.to_string(), peer);
        
        tracing::debug!("Added DHT peer: {} at {}", peer_id, address);
        
        Ok(())
    }
    
    pub async fn remove_peer(&self, peer_id: &str) -> Result<bool> {
        let mut peers = self.peers.write().await;
        
        if peers.remove(peer_id).is_some() {
            tracing::debug!("Removed DHT peer: {}", peer_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn find_closest_peers(&self, key: &str, count: usize) -> Vec<String> {
        let peers = self.peers.read().await;
        
        if peers.is_empty() {
            return Vec::new();
        }
        
        // Calculate distance for each peer
        let mut peer_distances: Vec<(&String, u32)> = peers.iter()
            .map(|(peer_id, _)| {
                let distance = self.calculate_key_peer_distance(key, peer_id);
                (peer_id, distance)
            })
            .collect();
        
        // Sort by distance (closest first)
        peer_distances.sort_by_key(|(_, distance)| *distance);
        
        // Take closest peers
        peer_distances.into_iter()
            .take(count)
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }
    
    pub async fn bootstrap(&self) -> Result<()> {
        if !self.config.enabled {
            return Err(Error::NotImplemented("DHT is disabled".to_string()));
        }
        
        tracing::info!("Bootstrapping DHT with {} nodes", self.config.bootstrap_nodes.len());
        
        for node in &self.config.bootstrap_nodes {
            // Parse node address and add to peers
            if let Some((peer_id, address)) = self.parse_node_address(node) {
                self.add_peer(&peer_id, &address).await?;
                tracing::debug!("Added bootstrap node: {} -> {}", peer_id, address);
            }
        }
        
        Ok(())
    }
    
    pub async fn get_stats(&self) -> Result<DhtStats> {
        let store = self.store.read().await;
        let peers = self.peers.read().await;
        
        let mut expired_count = 0;
        let now = Instant::now();
        
        for value in store.values() {
            if now.duration_since(value.timestamp) > value.ttl {
                expired_count += 1;
            }
        }
        
        Ok(DhtStats {
            total_values: store.len(),
            expired_values: expired_count,
            total_peers: peers.len(),
            replication_factor: self.config.replication_factor,
            average_distance: self.calculate_average_distance(&peers).await,
        })
    }
    
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut store = self.store.write().await;
        let now = Instant::now();
        
        let expired_keys: Vec<String> = store.iter()
            .filter(|(_, value)| now.duration_since(value.timestamp) > value.ttl)
            .map(|(key, _)| key.clone())
            .collect();
        
        let count = expired_keys.len();
        
        for key in expired_keys {
            store.remove(&key);
        }
        
        if count > 0 {
            tracing::debug!("Cleaned up {} expired DHT values", count);
        }
        
        Ok(count)
    }
    
    pub async fn replicate_values(&self) -> Result<usize> {
        let store = self.store.read().await;
        let mut replicated = 0;
        
        for value in store.values() {
            // Check if we need more replicas
            if value.replicas.len() < self.config.replication_factor {
                // Find additional peers for replication
                let additional_peers = self.find_closest_peers(&value.key, 
                    self.config.replication_factor - value.replicas.len()).await;
                
                // In real implementation, send replication requests to peers
                tracing::debug!("Need to replicate key {} to {} more peers", 
                    value.key, additional_peers.len());
                
                replicated += additional_peers.len();
            }
        }
        
        Ok(replicated)
    }
    
    // Helper methods
    
    fn calculate_distance(&self, peer_id: &str) -> u32 {
        // Simple XOR distance calculation
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        peer_id.hash(&mut hasher);
        (hasher.finish() % 1000) as u32
    }
    
    fn calculate_key_peer_distance(&self, key: &str, peer_id: &str) -> u32 {
        // XOR distance between key hash and peer ID hash
        let mut key_hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut key_hasher);
        let key_hash = key_hasher.finish();
        
        let mut peer_hasher = std::collections::hash_map::DefaultHasher::new();
        peer_id.hash(&mut peer_hasher);
        let peer_hash = peer_hasher.finish();
        
        (key_hash ^ peer_hash % 1000) as u32
    }
    
    async fn calculate_average_distance(&self, peers: &HashMap<String, DhtPeer>) -> f32 {
        if peers.is_empty() {
            return 0.0;
        }
        
        let total: u32 = peers.values().map(|p| p.distance).sum();
        total as f32 / peers.len() as f32
    }
    
    fn parse_node_address(&self, address: &str) -> Option<(String, String)> {
        // Simple parsing for demonstration
        // Format: peer_id@address
        if let Some(at_pos) = address.find('@') {
            let peer_id = &address[..at_pos];
            let addr = &address[at_pos + 1..];
            Some((peer_id.to_string(), addr.to_string()))
        } else {
            None
        }
    }
    
    async fn lookup_in_network(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Simulated network lookup
        // In real implementation, this would query peers
        
        let closest_peers = self.find_closest_peers(key, 3).await;
        
        if closest_peers.is_empty() {
            return Ok(None);
        }
        
        tracing::debug!("Looking up key {} in network via peers: {:?}", key, closest_peers);
        
        // Simulated network response
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct DhtStats {
    pub total_values: usize,
    pub expired_values: usize,
    pub total_peers: usize,
    pub replication_factor: usize,
    pub average_distance: f32,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            replication_factor: 3,
            key_size: 256,
            value_ttl: Duration::from_secs(3600), // 1 hour
            bootstrap_nodes: vec![
                "12D3KooWIPiRq6hAeMJ9bwLp6z4Xvq7LbHX8c6v6X8k4nYtN9sFm@/dns4/bootstrap.ippi.rs/tcp/4001".to_string(),
                "12D3KooWQw8nRrE6R7R6R6R6R6R6R6R6R6R6R6R6R6R6R6R6@/dns4/bootstrap.ippi.rs/tcp/4001".to_string(),
            ],
            protocol_version: "/ippi-dht/1.0.0".to_string(),
        }
    }
}