pub mod discovery;
pub mod ipfs;
pub mod sync;

use crate::Result;
use crate::audio::whisper::WhisperProcessor;
use crate::obsidian::{ObsidianManager, ObsidianConfig, AIResponse};
use crate::signal_integration::note_to_self::IncomingMessage;
use crate::vault::storage::VaultStorage;
use crate::swarm::ipfs::{IPFSNode, SwarmConfig, SwarmSyncStatus};
use anyhow::anyhow;
use chrono::{DateTime, Local};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Complete distributed AI swarm orchestrator
/// Handles the full voice-to-vault-to-AI workflow across devices
pub struct Swarm {
    /// IPFS private swarm for quantum-secure synchronization
    ipfs_node: Arc<IPFSNode>,
    /// Voice transcription processor
    whisper: Arc<WhisperProcessor>,
    /// Obsidian vault manager
    obsidian: Arc<ObsidianManager>,
    /// Vault storage engine
    vault_storage: Arc<VaultStorage>,
    /// Swarm configuration
    config: SwarmConfig,
    /// Sync monitoring
    sync_status: Arc<RwLock<SwarmSyncStatus>>,
}

/// User workflow event that triggers swarm actions
#[derive(Debug, Clone)]
pub enum SwarmEvent {
    /// Voice note received from Signal
    VoiceNoteReceived {
        audio_path: PathBuf,
        from_device: String,
        timestamp: DateTime<Local>,
    },
    /// Manual Obsidian edit on any device
    VaultFileEdited {
        file_path: PathBuf,
        device_name: String,
        change_type: ChangeType,
    },
    /// AI response generated
    AIResponseGenerated {
        response: AIResponse,
        source_query: String,
    },
    /// URL/link shared via Signal
    URLShared {
        url: String,
        context: Option<String>,
        from_device: String,
    },
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

/// Complete workflow result
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub success: bool,
    pub files_created: Vec<PathBuf>,
    pub files_synced: Vec<PathBuf>,
    pub transcription: Option<String>,
    pub ai_response_path: Option<PathBuf>,
    pub sync_status: SwarmSyncStatus,
}

impl Swarm {
    /// Create new distributed AI swarm
    pub async fn new(config: SwarmConfig) -> Result<Self> {
        info!("🚀 Initializing distributed AI swarm");
        
        // Initialize components
        let ipfs_node = Arc::new(IPFSNode::new(config.clone()).await?);
        let whisper = Arc::new(WhisperProcessor::new().await?);
        
        // Configure Obsidian for multi-device sync
        let obsidian_config = ObsidianConfig {
            vault_path: PathBuf::from("vault"),
            auto_link: true,
            default_tags: vec![
                "#ai-generated".to_string(),
                "#swarm-synced".to_string(),
                format!("#from-{}", config.node_config.node_name),
            ],
            ..ObsidianConfig::default()
        };
        let obsidian = Arc::new(ObsidianManager::new(obsidian_config));
        
        let vault_storage = Arc::new(VaultStorage::new().await?);
        
        // Initialize sync status
        let sync_status = Arc::new(RwLock::new(SwarmSyncStatus {
            connected_peers: 0,
            pending_uploads: 0,
            last_sync: 0,
            network_health: crate::swarm::ipfs::NetworkHealth::Disconnected,
        }));
        
        Ok(Self {
            ipfs_node,
            whisper,
            obsidian,
            vault_storage,
            config,
            sync_status,
        })
    }
    
    /// Start the complete swarm (IPFS node + sync loops + event processing)
    pub async fn start(&self) -> Result<()> {
        info!("🌐 Starting distributed AI swarm with quantum-secure vault sync");
        
        // Start IPFS private swarm
        self.ipfs_node.start_private_node().await?;
        
        // Start background sync monitoring
        self.start_sync_monitoring().await?;
        
        // Start vault change detection
        self.start_vault_watcher().await?;
        
        info!("✅ Distributed AI swarm started successfully");
        info!("📱 Ready to process voice notes from Signal");
        info!("📝 Ready to sync Obsidian vault across devices");
        
        Ok(())
    }
    
    /// Complete voice-to-vault workflow from Signal
    /// This is the main user workflow: voice note → transcription → vault → sync
    pub async fn process_voice_note_workflow(&self, voice_message: IncomingMessage) -> Result<WorkflowResult> {
        info!("🎤 Processing complete voice note workflow");
        
        let mut result = WorkflowResult {
            success: false,
            files_created: Vec::new(),
            files_synced: Vec::new(),
            transcription: None,
            ai_response_path: None,
            sync_status: self.ipfs_node.get_sync_status().await?,
        };
        
        // Step 1: Extract audio from Signal message
        let audio_path = self.extract_audio_from_signal(&voice_message).await?;
        info!("📱 Extracted audio: {}", audio_path.display());
        
        // Step 2: Transcribe with Whisper
        let transcription = self.whisper.transcribe_file(&audio_path).await?;
        result.transcription = Some(transcription.clone());
        info!("🎯 Transcribed: {:.100}...", transcription);
        
        // Step 3: Create structured note in Obsidian vault
        let ai_response = AIResponse {
            query: format!("Voice note from {}", voice_message.sender_phone),
            response: transcription.clone(),
            timestamp: Local::now(),
            model_used: "whisper-base".to_string(),
            confidence: Some(0.95),
            sources: vec![],
        };
        
        // Step 4: Save to Obsidian vault with proper formatting
        let note_path = self.obsidian.save_ai_response(ai_response).await?;
        result.files_created.push(note_path.clone());
        result.ai_response_path = Some(note_path.clone());
        info!("📝 Saved to vault: {}", note_path.display());
        
        // Step 5: Sync to private swarm (other devices)
        self.ipfs_node.sync_vault_to_swarm(vec![note_path.clone()]).await?;
        result.files_synced.push(note_path);
        info!("🌐 Synced to {} connected devices", result.sync_status.connected_peers);
        
        // Step 6: Update daily note with voice note summary
        let daily_summary = format!("🎤 Voice note transcribed: {transcription:.50}...");
        let daily_path = self.obsidian.append_to_daily_note(&daily_summary).await?;
        result.files_created.push(daily_path);
        
        result.success = true;
        info!("✅ Voice note workflow completed successfully");
        
        Ok(result)
    }
    
    /// Process manual vault edits (from Obsidian app on Android/other devices)
    pub async fn process_vault_edit_workflow(&self, file_path: PathBuf, device_name: String) -> Result<WorkflowResult> {
        info!("📝 Processing vault edit from device: {}", device_name);
        
        let mut result = WorkflowResult {
            success: false,
            files_created: Vec::new(),
            files_synced: Vec::new(),
            transcription: None,
            ai_response_path: None,
            sync_status: self.ipfs_node.get_sync_status().await?,
        };
        
        // Validate file exists and is in vault
        if !file_path.exists() || !file_path.starts_with("vault/") {
            return Err(anyhow!("Invalid vault file: {}", file_path.display()).into());
        }
        
        // Sync the edited file to swarm
        self.ipfs_node.sync_vault_to_swarm(vec![file_path.clone()]).await?;
        
        result.files_synced.push(file_path.clone());
        result.success = true;
        
        info!("✅ Vault edit synced to {} devices", result.sync_status.connected_peers);
        
        Ok(result)
    }
    
    /// Process URL sharing workflow from Signal
    pub async fn process_url_sharing_workflow(&self, url: String, context: Option<String>) -> Result<WorkflowResult> {
        info!("🔗 Processing URL sharing workflow: {}", url);
        
        let mut result = WorkflowResult {
            success: false,
            files_created: Vec::new(),
            files_synced: Vec::new(),
            transcription: None,
            ai_response_path: None,
            sync_status: self.ipfs_node.get_sync_status().await?,
        };
        
        // Create a research note for the URL
        let url_note_content = format!(
            "# Research Link: {}\n\n**URL:** {}\n\n**Context:** {}\n\n**Added:** {}\n\n**Tags:** #url-research #signal-shared\n\n## Notes\n\n<!-- Add your research notes here -->\n",
            self.extract_title_from_url(&url),
            url,
            context.unwrap_or_default(),
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        // Save as AI response (research note)
        let ai_response = AIResponse {
            query: format!("Research link: {url}"),
            response: url_note_content,
            timestamp: Local::now(),
            model_used: "manual".to_string(),
            confidence: Some(1.0),
            sources: vec![url.clone()],
        };
        
        let note_path = self.obsidian.save_ai_response(ai_response).await?;
        result.files_created.push(note_path.clone());
        result.ai_response_path = Some(note_path.clone());
        
        // Sync to swarm
        self.ipfs_node.sync_vault_to_swarm(vec![note_path]).await?;
        result.files_synced = result.files_created.clone();
        result.success = true;
        
        info!("✅ URL research note created and synced");
        
        Ok(result)
    }
    
    /// Get current swarm status for user feedback
    pub async fn get_swarm_status(&self) -> Result<SwarmSyncStatus> {
        self.ipfs_node.get_sync_status().await
    }
    
    /// Manually trigger full vault sync across all devices
    pub async fn trigger_full_sync(&self) -> Result<WorkflowResult> {
        info!("🔄 Triggering full vault synchronization");
        
        // Find all vault files
        let vault_files = self.discover_vault_files().await?;
        
        // Sync to swarm
        self.ipfs_node.sync_vault_to_swarm(vault_files.clone()).await?;
        
        // Pull updates from other devices
        let updated_files = self.ipfs_node.sync_vault_from_swarm().await?;
        
        let status = self.ipfs_node.get_sync_status().await?;
        
        Ok(WorkflowResult {
            success: true,
            files_created: updated_files.clone(),
            files_synced: vault_files,
            transcription: None,
            ai_response_path: None,
            sync_status: status,
        })
    }
    
    // === PRIVATE IMPLEMENTATION ===
    
    async fn extract_audio_from_signal(&self, message: &IncomingMessage) -> Result<PathBuf> {
        // In real implementation: 
        // 1. Extract audio attachment from Signal message
        // 2. Validate audio format and size
        // 3. Save to temporary processing location
        // 4. Return path to audio file
        
        // Mock implementation
        let audio_filename = format!("voice_note_{}.wav", 
            message.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
        let audio_path = PathBuf::from("temp").join(audio_filename);
        
        // Ensure temp directory exists
        tokio::fs::create_dir_all("temp").await?;
        
        // Create mock audio file for demonstration
        tokio::fs::write(&audio_path, b"mock audio data").await?;
        
        Ok(audio_path)
    }
    
    async fn start_sync_monitoring(&self) -> Result<()> {
        let ipfs_node = self.ipfs_node.clone();
        let sync_status = self.sync_status.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Update sync status
                if let Ok(status) = ipfs_node.get_sync_status().await {
                    *sync_status.write().await = status;
                }
            }
        });
        
        Ok(())
    }
    
    async fn start_vault_watcher(&self) -> Result<()> {
        info!("👁️ Starting vault file change detection");
        
        // In real implementation:
        // 1. Use inotify/FSEvents to watch vault directory
        // 2. Detect when files are created/modified/deleted
        // 3. Automatically trigger sync for changed files
        // 4. Handle conflicts with CRDT resolution
        
        // Mock implementation
        let ipfs_node = self.ipfs_node.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check for vault changes and sync if needed
                debug!("🔍 Checking vault for changes...");
                
                // In real implementation: detect actual file changes
                // For now: no-op
            }
        });
        
        Ok(())
    }
    
    async fn discover_vault_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let vault_dir = PathBuf::from("vault");
        
        if !vault_dir.exists() {
            return Ok(files);
        }
        
        self.scan_directory_recursive(&vault_dir, &mut files).await?;
        Ok(files)
    }
    
    async fn scan_directory_recursive(&self, dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            } else if path.is_dir() {
                Box::pin(self.scan_directory_recursive(&path, files)).await?;
            }
        }
        
        Ok(())
    }
    
    fn extract_title_from_url(&self, url: &str) -> String {
        // Simple URL title extraction
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return format!("Link from {host}");
            }
        }
        
        "Shared Link".to_string()
    }
}

/// Demo configuration for testing the complete workflow
impl Swarm {
    /// Create demo swarm configuration for development/testing
    pub fn demo_config() -> SwarmConfig {
        SwarmConfig {
            swarm_key: "demo_quantum_secure_key".to_string(),
            bootstrap_peers: vec![
                "android_phone_192.168.1.100".to_string(),
                "m1_macbook_192.168.1.101".to_string(),
            ],
            node_config: crate::swarm::ipfs::NodeConfig {
                node_name: "demo_m1_macbook".to_string(),
                device_type: crate::swarm::ipfs::DeviceType::M1MacBook,
                max_storage_gb: 50,
                max_bandwidth_mbps: 1000,
            },
            sync_config: crate::swarm::ipfs::SyncConfig {
                sync_interval_secs: 30,
                realtime_voice_sync: true,
                enable_crdt: true,
                quantum_encryption: true,
            },
        }
    }
}

// TODO: implement this file
