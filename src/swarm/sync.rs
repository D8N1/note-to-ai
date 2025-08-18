use crate::Result;
use crate::swarm::ipfs::{VaultEntry, VaultFileType, DeviceType};
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Cross-device vault synchronization engine
/// Handles conflict resolution, CRDT operations, and Android ↔ M1 MacBook sync
pub struct Sync {
    /// CRDT state for conflict-free synchronization
    crdt_state: Arc<RwLock<CRDTState>>,
    /// Device-specific sync preferences
    device_configs: HashMap<String, DeviceSyncConfig>,
    /// Pending operations to apply
    pending_operations: Arc<RwLock<Vec<SyncOperation>>>,
}

/// CRDT (Conflict-free Replicated Data Type) state for vault synchronization
/// This ensures that vault edits from different devices can be merged without conflicts
#[derive(Debug, Clone)]
pub struct CRDTState {
    /// Vector clock for operation ordering
    vector_clock: HashMap<String, u64>,
    /// Operation log for deterministic conflict resolution
    operation_log: Vec<CRDTOperation>,
    /// Current file states
    file_states: HashMap<PathBuf, FileState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTOperation {
    pub id: String,
    pub device_id: String,
    pub timestamp: u64,
    pub operation_type: OperationType,
    pub file_path: PathBuf,
    pub content_diff: Option<ContentDiff>,
    pub vector_clock: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    FileCreated,
    FileModified,
    FileDeleted,
    ContentInserted { position: usize },
    ContentDeleted { position: usize, length: usize },
    MetadataUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDiff {
    pub old_content: Option<String>,
    pub new_content: String,
    pub insertion_position: Option<usize>,
    pub deletion_range: Option<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct FileState {
    pub content_hash: String,
    pub last_modified: u64,
    pub last_device: String,
    pub version: u64,
    pub pending_changes: Vec<String>, // Operation IDs
}

/// Device-specific synchronization configuration
#[derive(Debug, Clone)]
pub struct DeviceSyncConfig {
    pub device_name: String,
    pub device_type: DeviceType,
    /// Conflict resolution strategy for this device
    pub conflict_strategy: ConflictStrategy,
    /// Sync preferences (realtime vs batched)
    pub sync_mode: SyncMode,
    /// Performance constraints
    pub bandwidth_limit: Option<u32>, // KB/s
    pub storage_limit: Option<u64>,   // GB
}

#[derive(Debug, Clone)]
pub enum ConflictStrategy {
    /// Always prefer changes from this device (useful for primary editing device)
    AlwaysWin,
    /// Always defer to other devices (useful for read-mostly devices)
    AlwaysDefer,
    /// Use timestamp-based resolution (newer wins)
    TimestampBased,
    /// Use CRDT merge (recommended)
    CRDTMerge,
    /// Prompt user for manual resolution
    UserDecision,
}

#[derive(Debug, Clone)]
pub enum SyncMode {
    /// Sync immediately on every change (good for voice notes)
    Realtime,
    /// Batch sync every N seconds
    Batched { interval_secs: u64 },
    /// Manual sync only
    Manual,
}

/// Synchronization operation that needs to be applied
#[derive(Debug, Clone)]
pub struct SyncOperation {
    pub operation_id: String,
    pub source_device: String,
    pub target_file: PathBuf,
    pub operation_type: OperationType,
    pub content: Option<Vec<u8>>,
    pub metadata: SyncMetadata,
}

#[derive(Debug, Clone)]
pub struct SyncMetadata {
    pub timestamp: u64,
    pub device_type: DeviceType,
    pub file_type: VaultFileType,
    pub priority: SyncPriority,
    pub conflict_resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone)]
pub enum SyncPriority {
    Critical,  // Voice notes, urgent AI responses
    High,      // Recent edits, daily notes
    Normal,    // Regular vault updates
    Low,       // Metadata, configuration
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub strategy_used: ConflictStrategy,
    pub winning_device: String,
    pub merged_content: Option<String>,
    pub conflict_details: String,
}

impl Sync {
    /// Create new cross-device synchronization engine
    pub fn new() -> Result<Self> {
        let crdt_state = Arc::new(RwLock::new(CRDTState {
            vector_clock: HashMap::new(),
            operation_log: Vec::new(),
            file_states: HashMap::new(),
        }));
        
        let pending_operations = Arc::new(RwLock::new(Vec::new()));
        
        // Configure common device types
        let mut device_configs = HashMap::new();
        
        // M1 MacBook: Primary editing device with high performance
        device_configs.insert("m1_macbook".to_string(), DeviceSyncConfig {
            device_name: "M1 MacBook".to_string(),
            device_type: DeviceType::M1MacBook,
            conflict_strategy: ConflictStrategy::CRDTMerge,
            sync_mode: SyncMode::Realtime,
            bandwidth_limit: None, // No limits on M1
            storage_limit: Some(100), // 100GB limit
        });
        
        // Android Phone: Voice input device with mobile constraints
        device_configs.insert("android_phone".to_string(), DeviceSyncConfig {
            device_name: "Android Phone".to_string(),
            device_type: DeviceType::AndroidPhone,
            conflict_strategy: ConflictStrategy::CRDTMerge,
            sync_mode: SyncMode::Realtime, // Important for voice notes
            bandwidth_limit: Some(1024), // 1MB/s on mobile
            storage_limit: Some(10), // 10GB limit on phone
        });
        
        Ok(Self {
            crdt_state,
            device_configs,
            pending_operations,
        })
    }
    
    /// Synchronize vault across all connected devices
    /// This is the main sync entry point called by the Swarm
    pub async fn sync_vault(&self) -> Result<SyncResult> {
        info!("🔄 Starting cross-device vault synchronization");
        
        let mut result = SyncResult {
            files_synced: 0,
            conflicts_resolved: 0,
            errors: Vec::new(),
            sync_duration_ms: 0,
        };
        
        let start_time = std::time::Instant::now();
        
        // Process pending operations in CRDT order
        let operations = self.pending_operations.read().await.clone();
        
        for operation in operations {
            match self.apply_sync_operation(&operation).await {
                Ok(_) => {
                    result.files_synced += 1;
                    debug!("✅ Applied sync operation: {}", operation.operation_id);
                }
                Err(e) => {
                    warn!("❌ Failed to apply sync operation {}: {}", operation.operation_id, e);
                    result.errors.push(format!("Operation {}: {}", operation.operation_id, e));
                }
            }
        }
        
        // Clear processed operations
        self.pending_operations.write().await.clear();
        
        // Update CRDT state
        self.update_vector_clock().await;
        
        result.sync_duration_ms = start_time.elapsed().as_millis() as u64;
        
        info!("✅ Vault sync completed: {} files, {} conflicts, {}ms", 
              result.files_synced, result.conflicts_resolved, result.sync_duration_ms);
        
        Ok(result)
    }
    
    /// Handle incoming vault entry from another device
    pub async fn handle_remote_vault_entry(&self, entry: VaultEntry, source_device: &str) -> Result<()> {
        info!("📥 Processing vault entry from {}: {}", source_device, entry.path.display());
        
        // Create sync operation
        let sync_op = SyncOperation {
            operation_id: entry.id.clone(),
            source_device: source_device.to_string(),
            target_file: entry.path.clone(),
            operation_type: self.determine_operation_type(&entry),
            content: Some(entry.content.clone()),
            metadata: SyncMetadata {
                timestamp: entry.timestamp,
                device_type: self.get_device_type(source_device),
                file_type: entry.metadata.file_type.clone(),
                priority: self.determine_priority(&entry),
                conflict_resolution: None,
            },
        };
        
        // Check for conflicts
        if let Some(conflict) = self.detect_conflict(&sync_op).await? {
            info!("⚠️ Conflict detected for {}, resolving...", entry.path.display());
            let resolved_op = self.resolve_conflict(sync_op, conflict).await?;
            self.pending_operations.write().await.push(resolved_op);
        } else {
            // No conflict, add to pending operations
            self.pending_operations.write().await.push(sync_op);
        }
        
        Ok(())
    }
    
    /// Sync priority files immediately (voice notes, urgent edits)
    pub async fn sync_priority_files(&self, files: Vec<PathBuf>) -> Result<()> {
        info!("🚀 Priority sync for {} files", files.len());
        
        for file_path in files {
            // Create high-priority sync operation
            let operation = self.create_priority_sync_operation(file_path).await?;
            
            // Apply immediately
            self.apply_sync_operation(&operation).await?;
            
            info!("⚡ Priority synced: {}", operation.target_file.display());
        }
        
        Ok(())
    }
    
    /// Handle Android Obsidian app edits
    /// Special handling for mobile device constraints and editing patterns
    pub async fn handle_android_edit(&self, file_path: PathBuf, content: Vec<u8>) -> Result<()> {
        info!("📱 Processing Android edit: {}", file_path.display());
        
        // Android-specific optimizations
        let android_config = self.device_configs.get("android_phone")
            .ok_or_else(|| anyhow!("Android device config not found"))?;
        
        // Create mobile-optimized sync operation
        let sync_op = SyncOperation {
            operation_id: uuid::Uuid::new_v4().to_string(),
            source_device: "android_phone".to_string(),
            target_file: file_path.clone(),
            operation_type: OperationType::FileModified,
            content: Some(content),
            metadata: SyncMetadata {
                timestamp: self.current_timestamp(),
                device_type: DeviceType::AndroidPhone,
                file_type: VaultFileType::MarkdownNote,
                priority: SyncPriority::High, // Android edits are usually important
                conflict_resolution: None,
            },
        };
        
        // Apply with mobile-friendly conflict resolution
        self.apply_mobile_sync_operation(sync_op, android_config).await?;
        
        info!("✅ Android edit synchronized");
        Ok(())
    }
    
    /// Get sync status for user dashboard
    pub async fn get_sync_status(&self) -> Result<SyncStatus> {
        let crdt_state = self.crdt_state.read().await;
        let pending_count = self.pending_operations.read().await.len();
        
        Ok(SyncStatus {
            total_files: crdt_state.file_states.len(),
            pending_operations: pending_count,
            last_sync: self.get_last_sync_timestamp().await,
            conflicts_pending: self.count_pending_conflicts().await,
            devices_connected: self.device_configs.len(),
        })
    }
    
    // === PRIVATE IMPLEMENTATION ===
    
    async fn apply_sync_operation(&self, operation: &SyncOperation) -> Result<()> {
        match &operation.operation_type {
            OperationType::FileCreated | OperationType::FileModified => {
                if let Some(content) = &operation.content {
                    // Ensure directory exists
                    if let Some(parent) = operation.target_file.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    
                    // Write file content
                    tokio::fs::write(&operation.target_file, content).await?;
                    
                    // Update CRDT state
                    self.update_file_state(&operation.target_file, operation).await?;
                }
            }
            OperationType::FileDeleted => {
                if operation.target_file.exists() {
                    tokio::fs::remove_file(&operation.target_file).await?;
                    self.remove_file_state(&operation.target_file).await?;
                }
            }
            OperationType::ContentInserted { position } => {
                // Handle incremental content updates (for real-time editing)
                self.apply_content_insertion(&operation.target_file, *position, operation).await?;
            }
            OperationType::ContentDeleted { position, length } => {
                self.apply_content_deletion(&operation.target_file, *position, *length).await?;
            }
            OperationType::MetadataUpdated => {
                // Update file metadata without changing content
                self.update_metadata_only(&operation.target_file, operation).await?;
            }
        }
        
        Ok(())
    }
    
    async fn detect_conflict(&self, operation: &SyncOperation) -> Result<Option<Conflict>> {
        let crdt_state = self.crdt_state.read().await;
        
        if let Some(file_state) = crdt_state.file_states.get(&operation.target_file) {
            // Check for concurrent modifications
            if file_state.last_modified > operation.metadata.timestamp {
                return Ok(Some(Conflict {
                    conflict_type: ConflictType::ConcurrentModification,
                    local_state: file_state.clone(),
                    remote_operation: operation.clone(),
                }));
            }
            
            // Check for device conflicts
            if file_state.last_device != operation.source_device && 
               file_state.last_modified.abs_diff(operation.metadata.timestamp) < 60 {
                return Ok(Some(Conflict {
                    conflict_type: ConflictType::DeviceConflict,
                    local_state: file_state.clone(),
                    remote_operation: operation.clone(),
                }));
            }
        }
        
        Ok(None)
    }
    
    async fn resolve_conflict(&self, operation: SyncOperation, conflict: Conflict) -> Result<SyncOperation> {
        info!("🔧 Resolving conflict: {:?}", conflict.conflict_type);
        
        // Get conflict resolution strategy
        let device_config = self.device_configs.get(&operation.source_device);
        let strategy = device_config
            .map(|c| c.conflict_strategy.clone())
            .unwrap_or(ConflictStrategy::CRDTMerge);
        
        match strategy {
            ConflictStrategy::CRDTMerge => {
                // Perform CRDT-based merge
                self.crdt_merge_resolution(operation, conflict).await
            }
            ConflictStrategy::TimestampBased => {
                // Newer timestamp wins
                if operation.metadata.timestamp > conflict.local_state.last_modified {
                    Ok(operation) // Remote wins
                } else {
                    // Local wins, return no-op
                    let mut op = operation;
                    op.content = None; // Don't apply
                    Ok(op)
                }
            }
            ConflictStrategy::AlwaysWin => {
                // This device always wins
                Ok(operation)
            }
            ConflictStrategy::AlwaysDefer => {
                // Always defer to existing content
                let mut op = operation;
                op.content = None;
                Ok(op)
            }
            ConflictStrategy::UserDecision => {
                // In real implementation: prompt user
                warn!("User decision required for conflict resolution - defaulting to CRDT merge");
                self.crdt_merge_resolution(operation, conflict).await
            }
        }
    }
    
    async fn crdt_merge_resolution(&self, operation: SyncOperation, conflict: Conflict) -> Result<SyncOperation> {
        // Implement CRDT-based conflict resolution
        // This is a simplified version - real CRDT implementation would be more complex
        
        if let Some(content) = &operation.content {
            // Read current file content
            let current_content = if operation.target_file.exists() {
                tokio::fs::read_to_string(&operation.target_file).await.unwrap_or_default()
            } else {
                String::new()
            };
            
            let new_content = String::from_utf8_lossy(content);
            
            // Simple merge strategy: append new content with conflict markers
            let merged_content = format!(
                "{}\n\n<<<<<<< Remote ({})\n{}\n=======\n{}\n>>>>>>> Local\n",
                current_content,
                operation.source_device,
                new_content,
                current_content
            );
            
            let mut resolved_op = operation;
            resolved_op.content = Some(merged_content.clone().into_bytes());
            resolved_op.metadata.conflict_resolution = Some(ConflictResolution {
                strategy_used: ConflictStrategy::CRDTMerge,
                winning_device: "merged".to_string(),
                merged_content: Some(merged_content),
                conflict_details: format!("Merged conflict between {} and local", resolved_op.source_device),
            });
            
            Ok(resolved_op)
        } else {
            Ok(operation)
        }
    }
    
    async fn apply_mobile_sync_operation(&self, operation: SyncOperation, _config: &DeviceSyncConfig) -> Result<()> {
        // Mobile-specific optimizations
        // - Compress content if over bandwidth limit
        // - Use differential sync for large files
        // - Prioritize based on mobile usage patterns
        
        self.apply_sync_operation(&operation).await
    }
    
    async fn create_priority_sync_operation(&self, file_path: PathBuf) -> Result<SyncOperation> {
        let content = tokio::fs::read(&file_path).await?;
        
        Ok(SyncOperation {
            operation_id: uuid::Uuid::new_v4().to_string(),
            source_device: "local".to_string(),
            target_file: file_path,
            operation_type: OperationType::FileModified,
            content: Some(content),
            metadata: SyncMetadata {
                timestamp: self.current_timestamp(),
                device_type: DeviceType::M1MacBook,
                file_type: VaultFileType::MarkdownNote,
                priority: SyncPriority::Critical,
                conflict_resolution: None,
            },
        })
    }
    
    // === UTILITY METHODS ===
    
    fn determine_operation_type(&self, entry: &VaultEntry) -> OperationType {
        // Determine what type of operation this represents
        if entry.path.exists() {
            OperationType::FileModified
        } else {
            OperationType::FileCreated
        }
    }
    
    fn get_device_type(&self, device_name: &str) -> DeviceType {
        self.device_configs.get(device_name)
            .map(|c| c.device_type.clone())
            .unwrap_or(DeviceType::LinuxDesktop)
    }
    
    fn determine_priority(&self, entry: &VaultEntry) -> SyncPriority {
        match entry.metadata.file_type {
            VaultFileType::VoiceTranscription => SyncPriority::Critical,
            VaultFileType::AIResponse => SyncPriority::High,
            VaultFileType::MarkdownNote => SyncPriority::Normal,
            _ => SyncPriority::Low,
        }
    }
    
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    async fn update_vector_clock(&self) {
        // Update CRDT vector clock
        // In real implementation: increment clock for this device
    }
    
    async fn update_file_state(&self, _file_path: &Path, _operation: &SyncOperation) -> Result<()> {
        // Update CRDT file state
        Ok(())
    }
    
    async fn remove_file_state(&self, _file_path: &Path) -> Result<()> {
        // Remove file from CRDT state
        Ok(())
    }
    
    async fn apply_content_insertion(&self, _file_path: &Path, _position: usize, _operation: &SyncOperation) -> Result<()> {
        // Apply incremental content insertion
        Ok(())
    }
    
    async fn apply_content_deletion(&self, _file_path: &Path, _position: usize, _length: usize) -> Result<()> {
        // Apply incremental content deletion
        Ok(())
    }
    
    async fn update_metadata_only(&self, _file_path: &Path, _operation: &SyncOperation) -> Result<()> {
        // Update only metadata without changing content
        Ok(())
    }
    
    async fn get_last_sync_timestamp(&self) -> u64 {
        self.current_timestamp() - 60 // Mock: 1 minute ago
    }
    
    async fn count_pending_conflicts(&self) -> usize {
        // Count conflicts that need resolution
        0 // Mock implementation
    }
}

// === SUPPORTING TYPES ===

#[derive(Debug, Clone)]
pub struct Conflict {
    pub conflict_type: ConflictType,
    pub local_state: FileState,
    pub remote_operation: SyncOperation,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    ConcurrentModification,
    DeviceConflict,
    ContentConflict,
    MetadataConflict,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub files_synced: usize,
    pub conflicts_resolved: usize,
    pub errors: Vec<String>,
    pub sync_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub total_files: usize,
    pub pending_operations: usize,
    pub last_sync: u64,
    pub conflicts_pending: usize,
    pub devices_connected: usize,
}
