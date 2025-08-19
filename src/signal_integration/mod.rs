// File: src/signal_integration/mod.rs
// Signal integration module for conversational AI assistant

pub mod note_to_self;
pub mod conversational_assistant;
pub mod signal_connector;
pub mod api_compatibility;
pub mod device_linking;

use crate::Result;
use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub use conversational_assistant::{
    ConversationalAssistant, ConversationalResponse, IntentType,
    UserProfile, ExecutiveLevel, UrgencyLevel, ProactiveInsight
};
pub use signal_connector::{SignalConnector, SignalConfig, ProcessedSignalMessage};
pub use note_to_self::{
    NoteToSelfProcessor, IncomingMessage, MessageType, ProcessedMessage,
    UXConfig, ResponseStyle, BriefFormat
};
pub use device_linking::{
    DeviceLinkManager, DeviceLinkConfig, LinkingStatus, QrDisplayMethod,
    quick_device_link, start_device_linking, test_qr_display
};

/// Complete Signal integration service
pub struct SignalIntegrationService {
    connector: Arc<RwLock<SignalConnector>>,
    config: SignalIntegrationConfig,
    is_running: Arc<RwLock<bool>>,
}

/// Configuration for Signal integration service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalIntegrationConfig {
    pub signal: SignalConfig,
    pub ux: UXConfig,
    pub features: FeatureConfig,
    pub performance: PerformanceConfig,
}

/// Feature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub enable_proactive_insights: bool,
    pub enable_voice_transcription: bool,
    pub enable_image_analysis: bool,
    pub enable_document_analysis: bool,
    pub enable_calendar_integration: bool,
    pub enable_background_research: bool,
    pub max_conversation_memory: usize,
    pub proactive_insight_interval_minutes: u64,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_response_time_seconds: u64,
    pub max_concurrent_messages: usize,
    pub attachment_processing_timeout_seconds: u64,
    pub ai_model_timeout_seconds: u64,
    pub memory_cleanup_interval_minutes: u64,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            enable_proactive_insights: true,
            enable_voice_transcription: true,
            enable_image_analysis: false, // Disabled by default
            enable_document_analysis: true,
            enable_calendar_integration: false, // Disabled by default
            enable_background_research: true,
            max_conversation_memory: 1000,
            proactive_insight_interval_minutes: 15,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_response_time_seconds: 8,
            max_concurrent_messages: 5,
            attachment_processing_timeout_seconds: 30,
            ai_model_timeout_seconds: 10,
            memory_cleanup_interval_minutes: 60,
        }
    }
}

impl Default for SignalIntegrationConfig {
    fn default() -> Self {
        Self {
            signal: SignalConfig::default(),
            ux: UXConfig::default(),
            features: FeatureConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl SignalIntegrationService {
    /// Create new Signal integration service
    pub async fn new(config: Option<SignalIntegrationConfig>) -> Result<Self> {
        let config = config.unwrap_or_else(|| {
            // Try to load from config file
            Self::load_config().unwrap_or_default()
        });
        
        info!("Initializing Signal integration service");
        
        // Validate configuration
        Self::validate_config(&config)?;
        
        // Create Signal connector
        let connector = SignalConnector::new(config.signal.clone()).await
            .context("Failed to create Signal connector")?;
        
        info!("Signal integration service initialized successfully");
        
        Ok(Self {
            connector: Arc::new(RwLock::new(connector)),
            config,
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the Signal integration service
    pub async fn start(&self) -> Result<()> {
        info!("Starting Signal integration service");
        
        // Set running flag
        *self.is_running.write().await = true;
        
        // Test Signal CLI connection first
        {
            let connector = self.connector.read().await;
            connector.test_connection().await
                .context("Signal CLI connection test failed")?;
        }
        
        // Start the connector
        {
            let connector = self.connector.read().await;
            connector.start().await
                .context("Failed to start Signal connector")?;
        }
        
        info!("Signal integration service started successfully");
        Ok(())
    }
    
    /// Stop the Signal integration service
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Signal integration service");
        
        // Set running flag
        *self.is_running.write().await = false;
        
        // Stop the connector
        {
            let connector = self.connector.read().await;
            connector.stop().await
                .context("Failed to stop Signal connector")?;
        }
        
        info!("Signal integration service stopped");
        Ok(())
    }
    
    /// Get service status
    pub async fn get_status(&self) -> ServiceStatus {
        let is_running = *self.is_running.read().await;
        let is_connected = {
            let connector = self.connector.read().await;
            connector.is_connected().await
        };
        
        let message_count = {
            let connector = self.connector.read().await;
            connector.get_message_history(None).await.len()
        };
        
        ServiceStatus {
            is_running,
            is_connected,
            messages_processed: message_count,
            uptime: std::time::SystemTime::now(),
            features_enabled: self.config.features.clone(),
        }
    }
    
    /// Get recent message history
    pub async fn get_message_history(&self, limit: Option<usize>) -> Vec<ProcessedSignalMessage> {
        let connector = self.connector.read().await;
        connector.get_message_history(limit).await
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, new_config: SignalIntegrationConfig) -> Result<()> {
        info!("Updating Signal integration configuration");
        
        // Validate new configuration
        Self::validate_config(&new_config)?;
        
        // Save configuration
        Self::save_config(&new_config)?;
        
        // Update runtime configuration
        self.config = new_config;
        
        info!("Configuration updated successfully");
        Ok(())
    }
    
    /// Load configuration from file
    fn load_config() -> Result<SignalIntegrationConfig> {
        let config_path = PathBuf::from("config/signal_integration.toml");
        
        if !config_path.exists() {
            info!("Signal integration config file not found, using defaults");
            return Ok(SignalIntegrationConfig::default());
        }
        
        let config_content = std::fs::read_to_string(&config_path)
            .context("Failed to read Signal integration config file")?;
        
        let config: SignalIntegrationConfig = toml::from_str(&config_content)
            .context("Failed to parse Signal integration config")?;
        
        info!("Signal integration configuration loaded from file");
        Ok(config)
    }
    
    /// Save configuration to file
    fn save_config(config: &SignalIntegrationConfig) -> Result<()> {
        let config_dir = PathBuf::from("config");
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }
        
        let config_path = config_dir.join("signal_integration.toml");
        let config_content = toml::to_string_pretty(config)
            .context("Failed to serialize Signal integration config")?;
        
        std::fs::write(&config_path, config_content)
            .context("Failed to write Signal integration config file")?;
        
        info!("Signal integration configuration saved to file");
        Ok(())
    }
    
    /// Validate configuration
    fn validate_config(config: &SignalIntegrationConfig) -> Result<()> {
        // Validate Signal CLI path (allow mock for development)
        if !config.signal.signal_cli_path.exists() && config.signal.signal_cli_path != PathBuf::from("echo") {
            return Err(anyhow!(
                "Signal CLI not found at path: {}",
                config.signal.signal_cli_path.display()
            ).into());
        }
        
        // Validate phone number format (basic check)
        if !config.signal.account_phone.starts_with('+') {
            return Err(anyhow!(
                "Invalid phone number format: {}. Must start with +",
                config.signal.account_phone
            ).into());
        }
        
        // Validate performance settings
        if config.performance.max_response_time_seconds == 0 {
            return Err(anyhow!("max_response_time_seconds must be greater than 0").into());
        }
        
        if config.performance.max_concurrent_messages == 0 {
            return Err(anyhow!("max_concurrent_messages must be greater than 0").into());
        }
        
        info!("Signal integration configuration validation passed");
        Ok(())
    }
    
    /// Setup Signal CLI (one-time setup)
    pub async fn setup_signal_cli(&self, verification_code: Option<String>) -> Result<()> {
        info!("Setting up Signal CLI");
        
        let config = &self.config.signal;
        
        // Skip setup for mock
        if config.signal_cli_path == PathBuf::from("echo") {
            info!("Using mock Signal CLI - skipping setup");
            return Ok(());
        }
        
        // Link account if verification code is provided
        if let Some(code) = verification_code {
            let output = tokio::process::Command::new(&config.signal_cli_path)
                .args(&[
                    "--config", config.data_dir.to_str().unwrap(),
                    "--account", &config.account_phone,
                    "verify",
                    &code,
                ])
                .output()
                .await
                .context("Failed to verify Signal CLI account")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Signal CLI verification failed: {}", stderr).into());
            }
            
            info!("Signal CLI account verified successfully");
        } else {
            // Register account (will need verification)
            let output = tokio::process::Command::new(&config.signal_cli_path)
                .args(&[
                    "--config", config.data_dir.to_str().unwrap(),
                    "register",
                    &config.account_phone,
                ])
                .output()
                .await
                .context("Failed to register Signal CLI account")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("already registered") {
                    info!("Signal CLI account already registered");
                } else {
                    return Err(anyhow!("Signal CLI registration failed: {}", stderr).into());
                }
            } else {
                info!("Signal CLI registration initiated - verification code required");
            }
        }
        
        Ok(())
    }
    
    /// Send a test message to verify setup
    pub async fn send_test_message(&self) -> Result<()> {
        info!("Sending test message");
        
        let config = &self.config.signal;
        
        let test_message = "🤖 Signal AI Assistant is now active!\n\nI'm ready to help with your notes, questions, and strategic insights. Just send me a message and I'll respond naturally.";
        
        let output = tokio::process::Command::new(&config.signal_cli_path)
            .args(&[
                "--config", config.data_dir.to_str().unwrap(),
                "--account", &config.account_phone,
                "send",
                "--note-to-self",
                "--message", test_message,
            ])
            .output()
            .await
            .context("Failed to send test message")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to send test message: {}", stderr).into());
        }
        
        info!("Test message sent successfully");
        Ok(())
    }
}

/// Service status information
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub is_running: bool,
    pub is_connected: bool,
    pub messages_processed: usize,
    pub uptime: std::time::SystemTime,
    pub features_enabled: FeatureConfig,
}

/// Quick setup helper for development
pub async fn quick_setup(phone_number: String) -> Result<SignalIntegrationService> {
    info!("Quick setup for Signal integration");
    
    let config = SignalIntegrationConfig {
        signal: SignalConfig {
            account_phone: phone_number.clone(),
            note_to_self_number: phone_number,
            ..SignalConfig::default()
        },
        ..SignalIntegrationConfig::default()
    };
    
    let service = SignalIntegrationService::new(Some(config)).await?;
    
    info!("Quick setup completed");
    Ok(service)
}

/// Development mode setup with mock Signal CLI
pub async fn dev_mode_setup() -> Result<SignalIntegrationService> {
    info!("Development mode setup for Signal integration");
    
    let temp_dir = std::env::temp_dir().join("signal_dev");
    std::fs::create_dir_all(&temp_dir)?;
    
    let config = SignalIntegrationConfig {
        signal: SignalConfig {
            signal_cli_path: PathBuf::from("echo"), // Mock command
            account_phone: "+1234567890".to_string(),
            note_to_self_number: "+1234567890".to_string(),
            data_dir: temp_dir.clone(),
            attachment_dir: temp_dir.join("attachments"),
            max_attachment_size_mb: 10,
        },
        features: FeatureConfig {
            enable_proactive_insights: false, // Disable for dev
            enable_voice_transcription: false,
            enable_image_analysis: false,
            enable_document_analysis: false,
            enable_calendar_integration: false,
            enable_background_research: false,
            max_conversation_memory: 100,
            proactive_insight_interval_minutes: 60,
        },
        ..SignalIntegrationConfig::default()
    };
    
    let service = SignalIntegrationService::new(Some(config)).await?;
    
    info!("Development mode setup completed");
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_service_creation() {
        let service = dev_mode_setup().await;
        assert!(service.is_ok());
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let config = SignalIntegrationConfig::default();
        
        // Should pass with mock Signal CLI path
        let result = SignalIntegrationService::validate_config(&config);
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_config_serialization() {
        let config = SignalIntegrationConfig::default();
        
        let serialized = toml::to_string(&config);
        assert!(serialized.is_ok());
        
        let deserialized: SignalIntegrationConfig = toml::from_str(&serialized.unwrap()).unwrap();
        assert_eq!(config.signal.account_phone, deserialized.signal.account_phone);
    }
}
