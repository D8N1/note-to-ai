use crate::Result;
use crate::crypto::Crypto;
use crate::vault::storage::VaultStorage;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{Duration, interval};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Configuration for IPFS private swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    /// Private swarm key for secure communication
    pub swarm_key: String,
    /// Bootstrap peers (trusted devices)
    pub bootstrap_peers: Vec<String>,
    /// Local node configuration
    pub node_config: NodeConfig,
    /// Sync settings
    pub sync_config: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node name/identifier
    pub node_name: String,
    /// Device type (M1MacBook, AndroidPhone, etc.)
    pub device_type: DeviceType,
    /// Storage limits
    pub max_storage_gb: u64,
    /// Bandwidth limits
    pub max_bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    M1MacBook,
    AndroidPhone,
    WindowsPC,
    LinuxDesktop,
    RaspberryPi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// How often to sync vault changes (seconds)
    pub sync_interval_secs: u64,
    /// Enable real-time sync for voice notes
    pub realtime_voice_sync: bool,
    /// Enable conflict resolution with CRDT
    pub enable_crdt: bool,
    /// Quantum encryption for vault content
    pub quantum_encryption: bool,
}

/// Represents a synchronized vault entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub path: PathBuf,
    pub content_hash: String,
    pub content: Vec<u8>, // Encrypted if quantum_encryption enabled
    pub metadata: VaultMetadata,
    pub device_origin: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub file_type: VaultFileType,
    pub size_bytes: u64,
    pub created_at: u64,
    pub modified_at: u64,
    pub tags: Vec<String>,
    pub encryption_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultFileType {
    MarkdownNote,
    AIResponse,
    VoiceTranscription,
    Attachment,
    Configuration,
}

/// IPFS private swarm node for quantum-secure vault synchronization
pub struct IPFSNode {
    config: SwarmConfig,
    crypto: Arc<Crypto>,
    vault_storage: Arc<VaultStorage>,
    connected_peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    sync_queue: Arc<Mutex<Vec<VaultEntry>>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub device_type: DeviceType,
    pub last_seen: u64,
    pub connection_quality: ConnectionQuality,
    pub vault_hash: Option<String>, // For quick sync checks
}

#[derive(Debug, Clone)]
pub enum ConnectionQuality {
    Excellent, // < 50ms, high bandwidth
    Good,      // < 200ms, medium bandwidth  
    Fair,      // < 500ms, low bandwidth
    Poor,      // > 500ms, very limited
}

impl IPFSNode {
    /// Create new IPFS private swarm node
    pub async fn new(config: SwarmConfig) -> Result<Self> {
        let crypto = Arc::new(Crypto::new()?);
        let vault_storage = Arc::new(VaultStorage::new().await?);
        
        Ok(Self {
            config,
            crypto,
            vault_storage,
            connected_peers: Arc::new(RwLock::new(HashMap::new())),
            sync_queue: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the private IPFS node and begin synchronization
    pub async fn start_private_node(&self) -> Result<()> {
        info!("🌐 Starting IPFS private swarm node: {}", self.config.node_config.node_name);
        
        // Mark as running
        *self.is_running.write().await = true;
        
        // Initialize private swarm network
        self.initialize_private_network().await?;
        
        // Connect to bootstrap peers (trusted devices)
        self.connect_to_bootstrap_peers().await?;
        
        // Start background sync loops
        self.start_sync_loops().await?;
        
        // Start peer discovery
        self.start_peer_discovery().await?;
        
        info!("✅ IPFS private swarm node started successfully");
        Ok(())
    }
    
    /// Stop the private node
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping IPFS private swarm node");
        *self.is_running.write().await = false;
        Ok(())
    }
    
    /// Sync vault content to the swarm (triggered by voice notes, manual edits)
    pub async fn sync_vault_to_swarm(&self, changed_files: Vec<PathBuf>) -> Result<()> {
        info!("📤 Syncing {} files to private swarm", changed_files.len());
        
        for file_path in changed_files {
            // Read file content
            let content = tokio::fs::read(&file_path).await
                .context(format!("Failed to read file: {}", file_path.display()))?;
            
            // Create vault entry
            let vault_entry = self.create_vault_entry(file_path, content).await?;
            
            // Add to sync queue
            self.sync_queue.lock().await.push(vault_entry);
        }
        
        // Trigger immediate sync if voice note
        if self.config.sync_config.realtime_voice_sync {
            self.process_sync_queue().await?;
        }
        
        Ok(())
    }
    
    /// Sync vault from swarm (pull changes from other devices)
    pub async fn sync_vault_from_swarm(&self) -> Result<Vec<PathBuf>> {
        info!("📥 Syncing vault from private swarm");
        let mut updated_files = Vec::new();
        
        // Query peers for vault updates
        let remote_entries = self.fetch_remote_vault_entries().await?;
        
        for entry in remote_entries {
            // Check if we need this update
            if self.should_apply_remote_update(&entry).await? {
                // Decrypt content if needed
                let content = if entry.metadata.encryption_used {
                    self.crypto.decrypt(&entry.content)?
                } else {
                    entry.content
                };
                
                // Write to local vault
                let local_path = self.config.node_config.node_name.as_str();
                let full_path = Path::new("vault").join(&entry.path);
                
                // Ensure directory exists
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                
                tokio::fs::write(&full_path, content).await?;
                updated_files.push(full_path);
                
                info!("📝 Updated: {} (from {})", entry.path.display(), entry.device_origin);
            }
        }
        
        info!("✅ Synced {} files from swarm", updated_files.len());
        Ok(updated_files)
    }
    
    /// Get sync status for user
    pub async fn get_sync_status(&self) -> Result<SwarmSyncStatus> {
        let peers = self.connected_peers.read().await;
        let queue_size = self.sync_queue.lock().await.len();
        
        Ok(SwarmSyncStatus {
            connected_peers: peers.len(),
            pending_uploads: queue_size,
            last_sync: self.get_last_sync_time().await?,
            network_health: self.assess_network_health(&peers).await,
        })
    }
    
    // === PRIVATE IMPLEMENTATION ===
    
    async fn initialize_private_network(&self) -> Result<()> {
        info!("🔐 Initializing private swarm with quantum-secure encryption");
        
        // In a real implementation, this would:
        // 1. Generate or load swarm key from secure storage
        // 2. Initialize libp2p with private network configuration
        // 3. Set up quantum-resistant encryption layer
        // 4. Configure DHT for peer discovery within swarm
        
        // For now, simulate initialization
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
    
    async fn connect_to_bootstrap_peers(&self) -> Result<()> {
        info!("🔗 Connecting to {} bootstrap peers", self.config.bootstrap_peers.len());
        
        for peer_addr in &self.config.bootstrap_peers {
            // In real implementation: attempt libp2p connection
            info!("📱 Connecting to trusted device: {}", peer_addr);
            
            // Simulate connection attempt
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Add to connected peers (mock)
            let peer_info = PeerInfo {
                peer_id: format!("peer_{}", Uuid::new_v4().to_string()[..8].to_string()),
                device_type: self.infer_device_type(peer_addr),
                last_seen: self.current_timestamp(),
                connection_quality: ConnectionQuality::Good,
                vault_hash: None,
            };
            
            self.connected_peers.write().await.insert(peer_addr.clone(), peer_info);
        }
        
        Ok(())
    }
    
    async fn start_sync_loops(&self) -> Result<()> {
        info!("🔄 Starting synchronization loops");
        
        // Periodic sync loop
        let sync_interval = Duration::from_secs(self.config.sync_config.sync_interval_secs);
        let sync_queue = self.sync_queue.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(sync_interval);
            
            while *is_running.read().await {
                interval.tick().await;
                
                // Process pending sync items
                let queue_len = sync_queue.lock().await.len();
                if queue_len > 0 {
                    debug!("🔄 Processing {} items in sync queue", queue_len);
                    // In real implementation: process the sync queue
                }
            }
        });
        
        // Real-time voice note sync (higher priority)
        if self.config.sync_config.realtime_voice_sync {
            let vault_storage = self.vault_storage.clone();
            let is_running = self.is_running.clone();
            
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
                
                while *is_running.read().await {
                    interval.tick().await;
                    
                    // Check for new voice transcriptions to sync immediately
                    debug!("🎤 Checking for new voice transcriptions to sync");
                    // In real implementation: detect new voice files and sync immediately
                }
            });
        }
        
        Ok(())
    }
    
    async fn start_peer_discovery(&self) -> Result<()> {
        info!("🔍 Starting peer discovery within private swarm");
        
        let connected_peers = self.connected_peers.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Discovery every 30s
            
            while *is_running.read().await {
                interval.tick().await;
                
                // In real implementation:
                // 1. Use mDNS to discover devices on local network
                // 2. Query DHT for swarm members
                // 3. Validate swarm key and establish secure connections
                // 4. Update peer connection quality based on latency/bandwidth
                
                let peer_count = connected_peers.read().await.len();
                debug!("🌐 Peer discovery: {} connected devices", peer_count);
            }
        });
        
        Ok(())
    }
    
    async fn create_vault_entry(&self, file_path: PathBuf, content: Vec<u8>) -> Result<VaultEntry> {
        let file_type = self.determine_file_type(&file_path);
        let content_hash = self.calculate_content_hash(&content);
        
        // Encrypt content if quantum encryption is enabled
        let (final_content, encryption_used) = if self.config.sync_config.quantum_encryption {
            (self.crypto.encrypt(&content)?, true)
        } else {
            (content, false)
        };
        
        let metadata = VaultMetadata {
            file_type,
            size_bytes: final_content.len() as u64,
            created_at: self.current_timestamp(),
            modified_at: self.current_timestamp(),
            tags: self.extract_tags_from_path(&file_path),
            encryption_used,
        };
        
        Ok(VaultEntry {
            id: Uuid::new_v4().to_string(),
            path: file_path,
            content_hash,
            content: final_content,
            metadata,
            device_origin: self.config.node_config.node_name.clone(),
            timestamp: self.current_timestamp(),
        })
    }
    
    async fn process_sync_queue(&self) -> Result<()> {
        let mut queue = self.sync_queue.lock().await;
        
        for entry in queue.drain(..) {
            // In real implementation: 
            // 1. Broadcast vault entry to connected peers
            // 2. Use content-addressed storage (IPFS CID)
            // 3. Track which peers have confirmed receipt
            // 4. Handle network failures with retry logic
            
            info!("📤 Syncing {} to {} peers", 
                  entry.path.display(), 
                  self.connected_peers.read().await.len());
        }
        
        Ok(())
    }
    
    async fn fetch_remote_vault_entries(&self) -> Result<Vec<VaultEntry>> {
        let mut remote_entries = Vec::new();
        
        // In real implementation:
        // 1. Query each connected peer for their vault state
        // 2. Compare hashes to determine what's new/changed
        // 3. Fetch missing/updated entries
        // 4. Verify signatures and swarm key authorization
        
        // Mock some remote entries for demonstration
        if !self.connected_peers.read().await.is_empty() {
            debug!("📥 Fetching vault updates from connected peers");
        }
        
        Ok(remote_entries)
    }
    
    async fn should_apply_remote_update(&self, entry: &VaultEntry) -> Result<bool> {
        // In real implementation:
        // 1. Check if we have this file locally
        // 2. Compare timestamps and content hashes
        // 3. Apply CRDT conflict resolution if needed
        // 4. Verify entry signature and permissions
        
        // For now, accept all updates from trusted devices
        Ok(true)
    }
    
    // === UTILITY METHODS ===
    
    fn determine_file_type(&self, path: &PathBuf) -> VaultFileType {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.contains("ai responses") || path_str.contains("ai-response") {
            VaultFileType::AIResponse
        } else if path_str.contains("voice") || path_str.contains("transcription") {
            VaultFileType::VoiceTranscription
        } else if path_str.ends_with(".md") {
            VaultFileType::MarkdownNote
        } else if path_str.contains("config") {
            VaultFileType::Configuration
        } else {
            VaultFileType::Attachment
        }
    }
    
    fn calculate_content_hash(&self, content: &[u8]) -> String {
        // In real implementation: use BLAKE3 for quantum-resistant hashing
        format!("blake3_{}", content.len()) // Mock hash
    }
    
    fn extract_tags_from_path(&self, path: &PathBuf) -> Vec<String> {
        let mut tags = Vec::new();
        let path_str = path.to_string_lossy();
        
        if path_str.contains("AI Responses") {
            tags.push("ai-generated".to_string());
        }
        if path_str.contains("voice") {
            tags.push("voice-note".to_string());
        }
        if path_str.contains("daily") {
            tags.push("daily-note".to_string());
        }
        
        tags
    }
    
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    fn infer_device_type(&self, peer_addr: &str) -> DeviceType {
        // Simple heuristic based on peer address patterns
        if peer_addr.contains("android") || peer_addr.contains("mobile") {
            DeviceType::AndroidPhone
        } else if peer_addr.contains("m1") || peer_addr.contains("mac") {
            DeviceType::M1MacBook
        } else {
            DeviceType::LinuxDesktop
        }
    }
    
    async fn get_last_sync_time(&self) -> Result<u64> {
        // In real implementation: track last successful sync
        Ok(self.current_timestamp() - 300) // Mock: 5 minutes ago
    }
    
    async fn assess_network_health(&self, peers: &HashMap<String, PeerInfo>) -> NetworkHealth {
        if peers.is_empty() {
            NetworkHealth::Disconnected
        } else if peers.len() >= 2 {
            NetworkHealth::Excellent
        } else {
            NetworkHealth::Good
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwarmSyncStatus {
    pub connected_peers: usize,
    pub pending_uploads: usize,
    pub last_sync: u64,
    pub network_health: NetworkHealth,
}

#[derive(Debug, Clone)]
pub enum NetworkHealth {
    Excellent,   // Multiple devices connected, low latency
    Good,        // At least one device connected
    Fair,        // Connection issues but functional
    Poor,        // Severe connection problems
    Disconnected, // No connected devices
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            swarm_key: "demo_swarm_key_replace_in_production".to_string(),
            bootstrap_peers: vec![],
            node_config: NodeConfig {
                node_name: format!("node_{}", Uuid::new_v4().to_string()[..8].to_string()),
                device_type: DeviceType::M1MacBook,
                max_storage_gb: 10,
                max_bandwidth_mbps: 100,
            },
            sync_config: SyncConfig {
                sync_interval_secs: 30,
                realtime_voice_sync: true,
                enable_crdt: true,
                quantum_encryption: true,
            },
        }
    }
}
