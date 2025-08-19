// File: src/signal_integration/device_linking.rs
// Signal device linking with QR code support for mobile-to-CLI connection

use crate::Result;
use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{timeout, sleep};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};
use qrcode::QrCode;
use qrcode::render::unicode;

/// Device linking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLinkConfig {
    pub signal_cli_path: PathBuf,
    pub data_dir: PathBuf,
    pub link_timeout_seconds: u64,
    pub qr_display_method: QrDisplayMethod,
    pub device_name: String,
}

/// How to display the QR code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QrDisplayMethod {
    /// Print to terminal as ASCII
    Terminal,
    /// Save as PNG image
    Image { path: PathBuf },
    /// Both terminal and image
    Both { path: PathBuf },
    /// Return raw URI only
    UriOnly,
}

/// Device linking status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkingStatus {
    /// Waiting to generate QR code
    Initializing,
    /// QR code generated, waiting for scan
    WaitingForScan {
        device_link_uri: String,
        qr_code_data: Option<String>,
        expires_at: SystemTime,
    },
    /// Mobile device scanned QR, completing setup
    Linking {
        device_id: Option<u32>,
    },
    /// Successfully linked
    Linked {
        device_id: u32,
        primary_number: String,
        linked_at: SystemTime,
    },
    /// Linking failed
    Failed {
        error: String,
        retry_available: bool,
    },
    /// User cancelled
    Cancelled,
}

/// Device linking manager
pub struct DeviceLinkManager {
    config: DeviceLinkConfig,
    status: LinkingStatus,
    link_process: Option<Child>,
}

/// Device linking result
#[derive(Debug, Clone)]
pub struct LinkingResult {
    pub success: bool,
    pub device_id: Option<u32>,
    pub primary_number: Option<String>,
    pub error: Option<String>,
}

impl Default for DeviceLinkConfig {
    fn default() -> Self {
        Self {
            signal_cli_path: PathBuf::from("/usr/local/bin/signal-cli"),
            data_dir: PathBuf::from("~/.local/share/signal-cli"),
            link_timeout_seconds: 300, // 5 minutes
            qr_display_method: QrDisplayMethod::Terminal,
            device_name: "note-to-ai-cli".to_string(),
        }
    }
}

impl DeviceLinkManager {
    /// Create new device link manager
    pub fn new(config: DeviceLinkConfig) -> Self {
        Self {
            config,
            status: LinkingStatus::Initializing,
            link_process: None,
        }
    }
    
    /// Start device linking process with QR code
    pub async fn start_linking(&mut self) -> Result<()> {
        info!("🔗 Starting Signal device linking process");
        
        self.status = LinkingStatus::Initializing;
        
        // Validate signal-cli is available
        self.validate_signal_cli().await?;
        
        // Start the linking process
        let link_uri = self.initiate_device_link().await?;
        
        // Generate and display QR code
        self.display_qr_code(&link_uri).await?;
        
        // Update status
        let expires_at = SystemTime::now() + Duration::from_secs(self.config.link_timeout_seconds);
        self.status = LinkingStatus::WaitingForScan {
            device_link_uri: link_uri.clone(),
            qr_code_data: Some(self.generate_qr_ascii(&link_uri)?),
            expires_at,
        };
        
        info!("📱 QR code ready! Scan with your Signal mobile app:");
        info!("   1. Open Signal on your mobile device");
        info!("   2. Go to Settings → Linked devices");
        info!("   3. Tap 'Link New Device'");
        info!("   4. Scan the QR code displayed above");
        
        // Wait for linking to complete
        self.wait_for_linking_completion().await
    }
    
    /// Get current linking status
    pub fn get_status(&self) -> &LinkingStatus {
        &self.status
    }
    
    /// Cancel linking process
    pub async fn cancel_linking(&mut self) -> Result<()> {
        info!("Cancelling device linking");
        
        if let Some(mut process) = self.link_process.take() {
            let _ = process.kill().await;
        }
        
        self.status = LinkingStatus::Cancelled;
        Ok(())
    }
    
    /// Re-display QR code if available
    pub fn display_current_qr(&self) -> Result<()> {
        match &self.status {
            LinkingStatus::WaitingForScan { device_link_uri, qr_code_data, expires_at } => {
                if SystemTime::now() < *expires_at {
                    println!("\n🔗 **SIGNAL DEVICE LINKING**");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    
                    if let Some(qr_data) = qr_code_data {
                        println!("{}", qr_data);
                    }
                    
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📱 Scan this QR code with Signal mobile app");
                    println!("⏱️  Expires in: {:.0} seconds", 
                        expires_at.duration_since(SystemTime::now())
                            .unwrap_or(Duration::ZERO).as_secs());
                    println!("🔗 URI: {}", device_link_uri);
                    Ok(())
                } else {
                    Err(anyhow!("QR code has expired").into())
                }
            }
            _ => Err(anyhow!("No QR code available to display").into())
        }
    }
    
    /// Check if we can retry linking
    pub fn can_retry(&self) -> bool {
        matches!(self.status, LinkingStatus::Failed { retry_available: true, .. } | LinkingStatus::Cancelled)
    }
    
    /// Validate signal-cli is available and working
    async fn validate_signal_cli(&self) -> Result<()> {
        debug!("Validating signal-cli installation");
        
        let output = Command::new(&self.config.signal_cli_path)
            .args(&["--version"])
            .output()
            .await
            .context("Failed to execute signal-cli")?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "signal-cli not working. Please install signal-cli: https://github.com/AsamK/signal-cli"
            ).into());
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        info!("✅ signal-cli found: {}", version.trim());
        
        Ok(())
    }
    
    /// Initiate device linking and get the URI
    async fn initiate_device_link(&mut self) -> Result<String> {
        info!("Initiating device link request");
        
        // Use signal-cli to start linking
        let mut cmd = Command::new(&self.config.signal_cli_path);
        cmd.args(&[
            "--config", self.config.data_dir.to_str().unwrap(),
            "link",
            "--name", &self.config.device_name,
        ]);
        
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .context("Failed to start signal-cli link command")?;
        
        // Read the device link URI from stdout
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("Failed to get signal-cli stdout"))?;
        
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        
        // The first line should contain the device link URI
        reader.read_line(&mut line).await
            .context("Failed to read device link URI")?;
        
        let device_link_uri = line.trim().to_string();
        
        // Validate URI format
        if !device_link_uri.starts_with("sgnl://linkdevice?") {
            return Err(anyhow!("Invalid device link URI received: {}", device_link_uri).into());
        }
        
        // Store the process for later
        self.link_process = Some(child);
        
        info!("✅ Device link URI generated");
        debug!("Device link URI: {}", device_link_uri);
        
        Ok(device_link_uri)
    }
    
    /// Generate and display QR code
    async fn display_qr_code(&self, uri: &str) -> Result<()> {
        match &self.config.qr_display_method {
            QrDisplayMethod::Terminal => {
                self.display_qr_terminal(uri)?;
            }
            QrDisplayMethod::Image { path } => {
                self.save_qr_image(uri, path).await?;
                println!("📁 QR code saved to: {}", path.display());
            }
            QrDisplayMethod::Both { path } => {
                self.display_qr_terminal(uri)?;
                self.save_qr_image(uri, path).await?;
                println!("📁 QR code also saved to: {}", path.display());
            }
            QrDisplayMethod::UriOnly => {
                println!("🔗 Device Link URI: {}", uri);
            }
        }
        
        Ok(())
    }
    
    /// Display QR code in terminal
    fn display_qr_terminal(&self, uri: &str) -> Result<()> {
        let code = QrCode::new(uri)
            .context("Failed to generate QR code")?;
        
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        
        println!("\n🔗 **SIGNAL DEVICE LINKING**");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{}", image);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        Ok(())
    }
    
    /// Generate QR code as ASCII string
    fn generate_qr_ascii(&self, uri: &str) -> Result<String> {
        let code = QrCode::new(uri)
            .context("Failed to generate QR code")?;
        
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        
        Ok(image)
    }
    
    /// Save QR code as PNG image
    async fn save_qr_image(&self, uri: &str, path: &PathBuf) -> Result<()> {
        let code = QrCode::new(uri)
            .context("Failed to generate QR code")?;
        
        // Create PNG image data
        let image = code.render::<qrcode::render::svg::Color>()
            .min_dimensions(200, 200)
            .dark_color(qrcode::render::svg::Color("#000000"))
            .light_color(qrcode::render::svg::Color("#FFFFFF"))
            .build();
        
        // Save SVG for now (could convert to PNG with additional dependencies)
        let svg_path = path.with_extension("svg");
        tokio::fs::write(&svg_path, image).await
            .context("Failed to save QR code image")?;
        
        info!("QR code saved as SVG: {}", svg_path.display());
        Ok(())
    }
    
    /// Wait for linking process to complete
    async fn wait_for_linking_completion(&mut self) -> Result<()> {
        info!("⏳ Waiting for mobile device to scan QR code...");
        
        let timeout_duration = Duration::from_secs(self.config.link_timeout_seconds);
        
        // Take ownership of the process to avoid borrow checker issues
        if let Some(mut process) = self.link_process.take() {
            let result = timeout(timeout_duration, self.monitor_linking_process(&mut process)).await;
            
            // Always put the process back, even if it failed
            self.link_process = Some(process);
            
            match result {
                Ok(result) => result,
                Err(_) => {
                    error!("⏰ Linking timeout reached");
                    self.status = LinkingStatus::Failed {
                        error: "Linking timeout - QR code was not scanned in time".to_string(),
                        retry_available: true,
                    };
                    if let Some(ref mut proc) = self.link_process {
                        let _ = proc.kill().await;
                    }
                    Err(anyhow!("Linking timeout").into())
                }
            }
        } else {
            Err(anyhow!("No linking process available").into())
        }
    }
    
    /// Monitor the signal-cli linking process
    async fn monitor_linking_process(&mut self, process: &mut Child) -> Result<()> {
        self.status = LinkingStatus::Linking { device_id: None };
        
        // Read stderr for progress updates
        if let Some(stderr) = process.stderr.take() {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            
            loop {
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let trimmed = line.trim();
                        debug!("signal-cli: {}", trimmed);
                        
                        // Parse linking progress
                        if trimmed.contains("Associated with:") {
                            // Extract phone number
                            if let Some(number) = self.extract_phone_number(trimmed) {
                                info!("📱 Successfully linked to: {}", number);
                                
                                self.status = LinkingStatus::Linked {
                                    device_id: 1, // TODO: Extract actual device ID
                                    primary_number: number,
                                    linked_at: SystemTime::now(),
                                };
                                
                                return Ok(());
                            }
                        } else if trimmed.contains("error") || trimmed.contains("failed") {
                            error!("❌ Linking failed: {}", trimmed);
                            self.status = LinkingStatus::Failed {
                                error: trimmed.to_string(),
                                retry_available: true,
                            };
                            return Err(anyhow!("Linking failed: {}", trimmed).into());
                        }
                        
                        line.clear();
                    }
                    Err(e) => {
                        error!("Error reading signal-cli output: {}", e);
                        break;
                    }
                }
            }
        }
        
        // Wait for process to complete
        let status = process.wait().await?;
        
        if status.success() {
            info!("✅ Device linking completed successfully");
            // If we haven't set a success status yet, set generic success
            if !matches!(self.status, LinkingStatus::Linked { .. }) {
                self.status = LinkingStatus::Linked {
                    device_id: 1,
                    primary_number: "Unknown".to_string(),
                    linked_at: SystemTime::now(),
                };
            }
            Ok(())
        } else {
            let error_msg = format!("signal-cli process failed with status: {}", status);
            self.status = LinkingStatus::Failed {
                error: error_msg.clone(),
                retry_available: true,
            };
            Err(anyhow!(error_msg).into())
        }
    }
    
    /// Extract phone number from signal-cli output
    fn extract_phone_number(&self, line: &str) -> Option<String> {
        // Look for "Associated with: +1234567890" pattern
        if let Some(start) = line.find("Associated with:") {
            let remaining = &line[start + 16..].trim();
            if let Some(number_end) = remaining.find(' ') {
                Some(remaining[..number_end].to_string())
            } else {
                Some(remaining.to_string())
            }
        } else {
            None
        }
    }
    
    /// Create a test QR code for verification
    pub fn create_test_qr() -> Result<String> {
        let test_uri = "sgnl://linkdevice?uuid=test-uuid-1234&pub_key=test-public-key-5678";
        
        let code = QrCode::new(test_uri)
            .context("Failed to generate test QR code")?;
        
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        
        Ok(image)
    }
}

/// Helper function to start device linking with defaults
pub async fn quick_device_link() -> Result<DeviceLinkManager> {
    let config = DeviceLinkConfig::default();
    let mut manager = DeviceLinkManager::new(config);
    
    manager.start_linking().await?;
    Ok(manager)
}

/// Helper function to start device linking with custom settings
pub async fn start_device_linking(
    signal_cli_path: Option<PathBuf>,
    device_name: Option<String>,
    qr_method: Option<QrDisplayMethod>,
) -> Result<DeviceLinkManager> {
    let config = DeviceLinkConfig {
        signal_cli_path: signal_cli_path.unwrap_or_else(|| PathBuf::from("/usr/local/bin/signal-cli")),
        device_name: device_name.unwrap_or_else(|| "note-to-ai-cli".to_string()),
        qr_display_method: qr_method.unwrap_or(QrDisplayMethod::Terminal),
        ..DeviceLinkConfig::default()
    };
    
    let mut manager = DeviceLinkManager::new(config);
    manager.start_linking().await?;
    Ok(manager)
}

/// Utility function to test QR code generation
pub fn test_qr_display() -> Result<()> {
    println!("🧪 Testing QR code generation...\n");
    
    let test_qr = DeviceLinkManager::create_test_qr()?;
    println!("{}", test_qr);
    
    println!("\n✅ QR code generation test successful!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_device_link_config() {
        let config = DeviceLinkConfig::default();
        assert_eq!(config.device_name, "note-to-ai-cli");
        assert_eq!(config.link_timeout_seconds, 300);
    }
    
    #[test]
    fn test_qr_code_generation() {
        let test_uri = "sgnl://linkdevice?uuid=test&pub_key=test";
        let result = DeviceLinkManager::create_test_qr();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_phone_number_extraction() {
        let manager = DeviceLinkManager::new(DeviceLinkConfig::default());
        
        let line = "Associated with: +1234567890 (Device linked)";
        let number = manager.extract_phone_number(line);
        assert_eq!(number, Some("+1234567890".to_string()));
    }
    
    #[tokio::test]
    async fn test_manager_creation() {
        let config = DeviceLinkConfig::default();
        let manager = DeviceLinkManager::new(config);
        
        assert!(matches!(manager.status, LinkingStatus::Initializing));
    }
}
