// File: src/signal_integration/signal_connector.rs
// Signal messaging platform integration for "Note to Self" conversations

use crate::Result;
use crate::signal_integration::conversational_assistant::{
    ConversationalAssistant, ConversationalResponse, InterruptDecision
};
use crate::signal_integration::note_to_self::{IncomingMessage, MessageType, Attachment};
use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time::sleep;
use tokio::io::AsyncBufReadExt;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Signal CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    pub signal_cli_path: PathBuf,
    pub account_phone: String,
    pub data_dir: PathBuf,
    pub note_to_self_number: String, // Same as account_phone for note-to-self
    pub attachment_dir: PathBuf,
    pub max_attachment_size_mb: u64,
}

/// Signal message event from signal-cli
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvent {
    pub envelope: SignalEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEnvelope {
    pub source: Option<String>,
    pub timestamp: u64,
    #[serde(rename = "dataMessage")]
    pub data_message: Option<SignalDataMessage>,
    #[serde(rename = "syncMessage")]
    pub sync_message: Option<SignalSyncMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SignalDataMessage {
    pub body: Option<String>,
    pub attachments: Option<Vec<SignalAttachment>>,
    pub timestamp: Option<u64>,
    pub group_info: Option<SignalGroupInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSyncMessage {
    #[serde(rename = "sentMessage")]
    pub sent_message: Option<SignalSentMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSentMessage {
    pub destination: Option<String>,
    pub message: Option<SignalDataMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAttachment {
    pub id: String,
    pub filename: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalGroupInfo {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub name: Option<String>,
}

/// Signal connection manager
pub struct SignalConnector {
    config: SignalConfig,
    assistant: Arc<RwLock<ConversationalAssistant>>,
    event_sender: mpsc::UnboundedSender<SignalEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SignalEvent>>>>,
    is_running: Arc<RwLock<bool>>,
    message_history: Arc<RwLock<Vec<ProcessedSignalMessage>>>,
}

/// Processed Signal message with AI response
#[derive(Debug, Clone)]
pub struct ProcessedSignalMessage {
    pub original_message: IncomingMessage,
    pub ai_response: ConversationalResponse,
    pub sent_at: SystemTime,
    pub response_delay: Duration,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            signal_cli_path: PathBuf::from("/usr/local/bin/signal-cli"),
            account_phone: "+1234567890".to_string(), // User must configure
            data_dir: PathBuf::from("~/.local/share/signal-cli"),
            note_to_self_number: "+1234567890".to_string(), // Same as account
            attachment_dir: PathBuf::from("./signal_attachments"),
            max_attachment_size_mb: 100,
        }
    }
}

impl SignalConnector {
    /// Create new Signal connector
    pub async fn new(config: SignalConfig) -> anyhow::Result<Self> {
        let assistant = Arc::new(RwLock::new(
            ConversationalAssistant::new().await?
        ));
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // Ensure attachment directory exists
        if !config.attachment_dir.exists() {
            std::fs::create_dir_all(&config.attachment_dir)
                .context("Failed to create attachment directory")?;
        }
        
        Ok(Self {
            config,
            assistant,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            is_running: Arc::new(RwLock::new(false)),
            message_history: Arc::new(RwLock::new(vec![])),
        })
    }
    
    /// Start Signal integration service
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting Signal connector service");
        
        // Set running flag
        *self.is_running.write().await = true;
        
        // Start Signal CLI listener in background
        let signal_listener = self.start_signal_listener().await?;
        
        // Start message processor
        let message_processor = self.start_message_processor().await?;
        
        // Start proactive insight generator
        let insight_generator = self.start_insight_generator().await?;
        
        info!("Signal connector service started successfully");
        
        // Wait for all tasks to complete
        tokio::try_join!(signal_listener, message_processor, insight_generator)?;
        
        Ok(())
    }
    
    /// Stop Signal integration service
    pub async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping Signal connector service");
        *self.is_running.write().await = false;
        Ok(())
    }
    
    /// Start Signal CLI listener task
    async fn start_signal_listener(&self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        let config = self.config.clone();
        let event_sender = self.event_sender.clone();
        let is_running = self.is_running.clone();
        
        let handle = tokio::spawn(async move {
            info!("Starting Signal CLI listener");
            
            while *is_running.read().await {
                match Self::listen_for_signal_events(&config, &event_sender).await {
                    Ok(_) => {
                        debug!("Signal CLI listener cycle completed");
                    }
                    Err(e) => {
                        error!("Signal CLI listener error: {}", e);
                        // Wait before retrying
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            
            info!("Signal CLI listener stopped");
            Ok(())
        });
        
        Ok(handle)
    }
    
    /// Start message processor task
    async fn start_message_processor(&self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        let mut event_receiver = self.event_receiver.write().await
            .take()
            .ok_or_else(|| anyhow!("Event receiver already taken"))?;
        
        let assistant = self.assistant.clone();
        let config = self.config.clone();
        let message_history = self.message_history.clone();
        let is_running = self.is_running.clone();
        
        let handle = tokio::spawn(async move {
            info!("Starting Signal message processor");
            
            while *is_running.read().await {
                tokio::select! {
                    event_opt = event_receiver.recv() => {
                        if let Some(event) = event_opt {
                            if let Err(e) = Self::process_signal_event(
                                event, 
                                &assistant, 
                                &config, 
                                &message_history
                            ).await {
                                error!("Failed to process Signal event: {}", e);
                            }
                        }
                    }
                    _ = sleep(Duration::from_millis(100)) => {
                        // Periodic check to allow breaking from loop
                    }
                }
            }
            
            info!("Signal message processor stopped");
            Ok(())
        });
        
        Ok(handle)
    }
    
    /// Start proactive insight generator task
    async fn start_insight_generator(&self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        let assistant = self.assistant.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        
        let handle = tokio::spawn(async move {
            info!("Starting proactive insight generator");
            
            while *is_running.read().await {
                // Check for proactive insights every 15 minutes
                sleep(Duration::from_secs(15 * 60)).await;
                
                if !*is_running.read().await {
                    break;
                }
                
                // Generate periodic insights
                match assistant.read().await.generate_periodic_insight().await {
                    Ok(Some(insight)) => {
                        debug!("Generated proactive insight: {:?}", insight.topic);
                        
                        // Check if we should send this insight
                        let should_send = assistant.read().await
                            .interruption_manager
                            .should_send_proactive_message(&insight);
                        
                        if matches!(should_send, InterruptDecision::SendImmediately) {
                            if let Err(e) = Self::send_proactive_insight(&config, &insight).await {
                                error!("Failed to send proactive insight: {}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        debug!("No new insights generated this cycle");
                    }
                    Err(e) => {
                        error!("Failed to generate insights: {}", e);
                    }
                }
            }
            
            info!("Proactive insight generator stopped");
            Ok(())
        });
        
        Ok(handle)
    }
    
    /// Listen for Signal events using signal-cli
    async fn listen_for_signal_events(
        config: &SignalConfig,
        event_sender: &mpsc::UnboundedSender<SignalEvent>,
    ) -> Result<()> {
        info!("Starting signal-cli listener");
        
        let mut cmd = tokio::process::Command::new(&config.signal_cli_path);
        cmd.args([
            "--config", config.data_dir.to_str().unwrap(),
            "--account", &config.account_phone,
            "daemon",
            "--json",
        ]);
        
        let mut child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to start signal-cli daemon")?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("Failed to get signal-cli stdout"))?;
        
        let mut lines = tokio::io::BufReader::new(stdout).lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            debug!("Signal CLI output: {}", line);
            
            // Parse JSON event
            match serde_json::from_str::<SignalEvent>(&line) {
                Ok(event) => {
                    debug!("Parsed Signal event: {:?}", event);
                    
                    if let Err(e) = event_sender.send(event) {
                        error!("Failed to send Signal event: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    debug!("Failed to parse Signal event JSON: {} - Line: {}", e, line);
                    // Continue processing other lines
                }
            }
        }
        
        // Wait for process to complete
        let status = child.wait().await?;
        if !status.success() {
            warn!("signal-cli daemon exited with status: {}", status);
        }
        
        Ok(())
    }
    
    /// Process incoming Signal event
    async fn process_signal_event(
        event: SignalEvent,
        assistant: &Arc<RwLock<ConversationalAssistant>>,
        config: &SignalConfig,
        message_history: &Arc<RwLock<Vec<ProcessedSignalMessage>>>,
    ) -> Result<()> {
        debug!("Processing Signal event: {:?}", event.envelope.timestamp);
        
        // Check if this is a note-to-self message
        let message = if let Some(data_msg) = event.envelope.data_message {
            // Direct message
            if let Some(source) = &event.envelope.source {
                if source == &config.note_to_self_number {
                    data_msg
                } else {
                    debug!("Ignoring message from other contact: {}", source);
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        } else if let Some(sync_msg) = event.envelope.sync_message {
            // Sync message (sent from this device)
            if let Some(sent_msg) = sync_msg.sent_message {
                if let Some(dest) = &sent_msg.destination {
                    if dest == &config.note_to_self_number {
                        sent_msg.message.unwrap_or_default()
                    } else {
                        debug!("Ignoring sync message to other contact: {}", dest);
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        } else {
            debug!("No data or sync message in envelope");
            return Ok(());
        };
        
        // Convert Signal message to internal format
        let sender = event.envelope.source.clone().unwrap_or_else(|| config.note_to_self_number.clone());
        let incoming_message = Self::convert_signal_message(&message, &sender, config).await?;
        
        debug!("Converted to internal message: {:?}", incoming_message.message_type);
        
        // Process with AI assistant
        let start_time = SystemTime::now();
        
        let ai_response = {
            let mut assistant_guard = assistant.write().await;
            assistant_guard.process_signal_message(incoming_message.clone()).await?
        };
        
        let response_delay = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        
        debug!("AI response generated in {:?}: {}", response_delay, ai_response.content);
        
        // Simulate natural typing delay
        let assistant_guard = assistant.read().await;
        let typing_delay = assistant_guard.timing.calculate_response_delay(ai_response.content.len());
        drop(assistant_guard);
        
        sleep(typing_delay).await;
        
        // Send response via Signal
        Self::send_signal_response(config, &ai_response).await?;
        
        // Store in message history
        let processed_message = ProcessedSignalMessage {
            original_message: incoming_message,
            ai_response,
            sent_at: SystemTime::now(),
            response_delay: response_delay + typing_delay,
        };
        
        message_history.write().await.push(processed_message);
        
        info!("Signal message processed and response sent");
        
        Ok(())
    }
    
    /// Convert Signal message to internal format
    async fn convert_signal_message(
        message: &SignalDataMessage,
        sender: &str,
        config: &SignalConfig,
    ) -> Result<IncomingMessage> {
        let message_id = Uuid::new_v4().to_string();
        let timestamp = SystemTime::UNIX_EPOCH + Duration::from_millis(
            message.timestamp.unwrap_or(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            )
        );
        
        let message_type = if let Some(attachments) = &message.attachments {
            if attachments.is_empty() {
                // Text only
                MessageType::Text {
                    content: message.body.clone().unwrap_or_default(),
                }
            } else if attachments.len() == 1 {
                // Single attachment
                let attachment = &attachments[0];
                
                if attachment.content_type.starts_with("audio/") {
                    // Voice message
                    let audio_path = Self::download_attachment(attachment, config).await?;
                    let duration = Self::estimate_audio_duration(&audio_path).await?;
                    
                    MessageType::Voice {
                        audio_path,
                        duration_seconds: duration,
                    }
                } else if attachment.content_type.starts_with("image/") {
                    // Image
                    let image_path = Self::download_attachment(attachment, config).await?;
                    
                    MessageType::Image {
                        image_path,
                        caption: message.body.clone(),
                    }
                } else {
                    // Document
                    let doc_path = Self::download_attachment(attachment, config).await?;
                    
                    MessageType::Document {
                        doc_path,
                        filename: attachment.filename.clone().unwrap_or_else(|| {
                            format!("attachment_{}", attachment.id)
                        }),
                        caption: message.body.clone(),
                    }
                }
            } else {
                // Multiple attachments - mixed message
                let mut converted_attachments = vec![];
                
                for attachment in attachments {
                    let path = Self::download_attachment(attachment, config).await?;
                    
                    let converted = if attachment.content_type.starts_with("audio/") {
                        let duration = Self::estimate_audio_duration(&path).await?;
                        Attachment::Voice { path, duration }
                    } else if attachment.content_type.starts_with("image/") {
                        Attachment::Image { path }
                    } else {
                        let filename = attachment.filename.clone().unwrap_or_else(|| {
                            format!("attachment_{}", attachment.id)
                        });
                        Attachment::Document { path, filename }
                    };
                    
                    converted_attachments.push(converted);
                }
                
                MessageType::Mixed {
                    text: message.body.clone(),
                    attachments: converted_attachments,
                }
            }
        } else {
            // Text only
            MessageType::Text {
                content: message.body.clone().unwrap_or_default(),
            }
        };
        
        Ok(IncomingMessage {
            id: message_id,
            timestamp,
            message_type,
            sender_phone: sender.to_string(),
            conversation_id: "note-to-self".to_string(),
        })
    }
    
    /// Download Signal attachment
    async fn download_attachment(
        attachment: &SignalAttachment,
        config: &SignalConfig,
    ) -> Result<PathBuf> {
        let filename = attachment.filename.clone().unwrap_or_else(|| {
            format!("attachment_{}_{}", attachment.id, SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs())
        });
        
        let attachment_path = config.attachment_dir.join(&filename);
        
        // Use signal-cli to download attachment
        let output = tokio::process::Command::new(&config.signal_cli_path)
            .args([
                "--config", config.data_dir.to_str().unwrap(),
                "--account", &config.account_phone,
                "receive",
                "--attachment-dir", config.attachment_dir.to_str().unwrap(),
                "--attachment", &attachment.id,
            ])
            .output()
            .await
            .context("Failed to execute signal-cli download")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to download attachment: {}", stderr).into());
        }
        
        Ok(attachment_path)
    }
    
    /// Estimate audio duration (placeholder implementation)
    async fn estimate_audio_duration(_audio_path: &PathBuf) -> Result<u32> {
        // TODO: Use audio processing library to get actual duration
        // For now, return estimate based on file size
        Ok(30) // 30 seconds estimate
    }
    
    /// Send AI response via Signal
    async fn send_signal_response(
        config: &SignalConfig,
        response: &ConversationalResponse,
    ) -> Result<()> {
        debug!("Sending Signal response: {}", response.content);
        
        let mut cmd = tokio::process::Command::new(&config.signal_cli_path);
        cmd.args([
            "--config", config.data_dir.to_str().unwrap(),
            "--account", &config.account_phone,
            "send",
            "--note-to-self",
            "--message", &response.content,
        ]);
        
        let output = cmd.output().await
            .context("Failed to execute signal-cli send")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to send Signal message: {}", stderr).into());
        }
        
        debug!("Signal response sent successfully");
        Ok(())
    }
    
    /// Send proactive insight
    async fn send_proactive_insight(
        config: &SignalConfig,
        insight: &crate::signal_integration::conversational_assistant::ProactiveInsight,
    ) -> Result<()> {
        let message = format!(
            "💡 **Proactive Insight**\n\n{}\n\n**Suggested Action:** {}\n\n*Confidence: {:.0}%*",
            insight.insight,
            insight.suggested_action,
            insight.confidence * 100.0
        );
        
        debug!("Sending proactive insight: {}", insight.topic);
        
        let mut cmd = tokio::process::Command::new(&config.signal_cli_path);
        cmd.args([
            "--config", config.data_dir.to_str().unwrap(),
            "--account", &config.account_phone,
            "send",
            "--note-to-self",
            "--message", &message,
        ]);
        
        let output = cmd.output().await
            .context("Failed to execute signal-cli send for insight")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to send proactive insight: {}", stderr).into());
        }
        
        info!("Proactive insight sent successfully: {}", insight.topic);
        Ok(())
    }
    
    /// Get message history
    pub async fn get_message_history(&self, limit: Option<usize>) -> Vec<ProcessedSignalMessage> {
        let history = self.message_history.read().await;
        let count = limit.unwrap_or(history.len());
        
        history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Get connection status
    pub async fn is_connected(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// Test Signal CLI connection
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        info!("Testing Signal CLI connection");
        
        let output = tokio::process::Command::new(&self.config.signal_cli_path)
            .args([
                "--config", self.config.data_dir.to_str().unwrap(),
                "--account", &self.config.account_phone,
                "listIdentities",
            ])
            .output()
            .await
            .context("Failed to test Signal CLI connection")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Signal CLI connection test failed: {}", stderr));
        }
        
        info!("Signal CLI connection test successful");
        Ok(())
    }
}

// Fix compilation error in SignalDataMessage default implementation

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_signal_config_creation() {
        let config = SignalConfig::default();
        assert!(!config.account_phone.is_empty());
        assert_eq!(config.account_phone, config.note_to_self_number);
    }
    
    #[tokio::test]
    async fn test_signal_connector_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = SignalConfig {
            attachment_dir: temp_dir.path().to_path_buf(),
            ..SignalConfig::default()
        };
        
        let connector = SignalConnector::new(config).await;
        assert!(connector.is_ok());
    }
    
    #[test]
    fn test_signal_event_parsing() {
        let json = r#"{
            "envelope": {
                "source": "+1234567890",
                "timestamp": 1234567890000,
                "dataMessage": {
                    "body": "Test message",
                    "timestamp": 1234567890000
                }
            }
        }"#;
        
        let event: SignalEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.envelope.source, Some("+1234567890".to_string()));
        assert!(event.envelope.data_message.is_some());
    }
}
