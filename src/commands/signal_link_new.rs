// File: src/commands/signal_link.rs
// Command-line interface for Signal device linking

use crate::Result;
use crate::signal_integration::{
    DeviceLinkManager, DeviceLinkConfig, LinkingStatus, QrDisplayMethod,
    quick_device_link, start_device_linking, test_qr_display
};
use anyhow::{Context, anyhow};
use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

/// Command-line interface for Signal device linking
pub struct SignalLinkCommand;

impl SignalLinkCommand {
    /// Create a new SignalLinkCommand instance
    pub fn new() -> Self {
        SignalLinkCommand
    }

    /// Create the signal link CLI command
    pub fn command() -> Command {
        Command::new("signal-link")
            .about("Link your Signal mobile device with this CLI instance")
            .subcommand_required(false)
            .arg_required_else_help(false)
            .args([
                Arg::new("quick")
                    .long("quick")
                    .short('q')
                    .help("Quick one-step device linking")
                    .action(clap::ArgAction::SetTrue),
                Arg::new("step-by-step")
                    .long("step-by-step")
                    .short('s')
                    .help("Step-by-step guided linking process")
                    .action(clap::ArgAction::SetTrue),
                Arg::new("test-qr")
                    .long("test-qr")
                    .short('t')
                    .help("Test QR code display functionality")
                    .action(clap::ArgAction::SetTrue),
                Arg::new("status")
                    .long("status")
                    .help("Check current linking status")
                    .action(clap::ArgAction::SetTrue),
                Arg::new("signal-cli-path")
                    .long("signal-cli-path")
                    .help("Path to signal-cli executable")
                    .value_name("PATH"),
                Arg::new("device-name")
                    .long("device-name")
                    .help("Name for this linked device")
                    .value_name("NAME"),
                Arg::new("timeout")
                    .long("timeout")
                    .help("Linking timeout in seconds")
                    .value_name("SECONDS")
                    .value_parser(clap::value_parser!(u64)),
            ])
    }

    /// Handle the signal link command with arguments
    pub async fn handle(matches: &ArgMatches) -> Result<()> {
        let cmd = SignalLinkCommand::new();

        if matches.get_flag("quick") {
            cmd.quick_device_link().await
        } else if matches.get_flag("step-by-step") {
            cmd.step_by_step_linking().await
        } else if matches.get_flag("test-qr") {
            cmd.test_qr_display().await
        } else if matches.get_flag("status") {
            cmd.check_linking_status().await
        } else {
            // Default to quick linking if no specific flag is provided
            cmd.quick_device_link().await
        }
    }

    /// Quick one-step device linking process
    pub async fn quick_device_link(&self) -> Result<()> {
        println!("🔗 Starting quick Signal device linking...");
        
        let mut manager = quick_device_link().await?;
        
        println!("\n📱 Scan this QR code with your Signal mobile app:");
        manager.display_current_qr()?;
        
        println!("\n⏳ Waiting for device linking confirmation...");
        println!("   (This process will timeout after 5 minutes)");
        
        // Monitor linking status
        let mut attempts = 0;
        let max_attempts = 60; // 5 minutes at 5-second intervals
        
        loop {
            sleep(Duration::from_secs(5)).await;
            attempts += 1;
            
            match manager.get_status() {
                LinkingStatus::Linked { device_id, primary_number, .. } => {
                    println!("✅ Device linked successfully!");
                    println!("   Device ID: {}", device_id);
                    println!("   Primary number: {}", primary_number);
                    break;
                }
                LinkingStatus::Failed { error, .. } => {
                    println!("❌ Device linking failed: {}", error);
                    break;
                }
                LinkingStatus::Linking { device_id } => {
                    println!("   📱 Device scanned! Completing setup...");
                    if let Some(id) = device_id {
                        println!("   Device ID: {}", id);
                    }
                }
                LinkingStatus::Cancelled => {
                    println!("❌ Device linking was cancelled");
                    break;
                }
                LinkingStatus::WaitingForScan { .. } => {
                    if attempts >= max_attempts {
                        println!("❌ Device linking timed out");
                        manager.cancel_linking().await?;
                        break;
                    }
                    if attempts % 12 == 0 { // Every minute
                        println!("   Still waiting... ({} minutes elapsed)", attempts / 12);
                    }
                }
                LinkingStatus::Initializing => {
                    println!("   Initializing...");
                }
            }
        }
        
        Ok(())
    }

    /// Step-by-step guided linking process
    pub async fn step_by_step_linking(&self) -> Result<()> {
        println!("📖 Step-by-step Signal device linking guide");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        println!("\n📋 Step 1: Ensure Signal CLI is installed and configured");
        println!("   - signal-cli should be available in your PATH");
        println!("   - Run 'signal-cli --version' to verify installation");
        
        println!("\n📋 Step 2: Prepare your mobile device");
        println!("   - Open Signal app on your mobile device");
        println!("   - Go to Settings > Linked devices");
        println!("   - Tap the '+' button to add a new device");
        
        println!("\n📋 Step 3: Generate linking QR code");
        let mut manager = start_device_linking(None, None, None).await?;
        
        println!("\n📱 Scan this QR code with your Signal mobile app:");
        manager.display_current_qr()?;
        
        println!("\n📋 Step 4: Complete the linking process");
        println!("   - Point your mobile camera at the QR code");
        println!("   - Confirm the device linking in the Signal app");
        println!("   - Wait for confirmation...");
        
        // Extended timeout for step-by-step process
        let mut attempts = 0;
        let max_attempts = 120; // 10 minutes at 5-second intervals
        
        loop {
            sleep(Duration::from_secs(5)).await;
            attempts += 1;
            
            match manager.get_status() {
                LinkingStatus::Linked { device_id, primary_number, .. } => {
                    println!("\n✅ Device linking completed successfully!");
                    println!("   Your mobile Signal is now linked with this CLI instance.");
                    println!("   Device ID: {}", device_id);
                    println!("   Primary number: {}", primary_number);
                    break;
                }
                LinkingStatus::Failed { error, .. } => {
                    println!("\n❌ Device linking failed: {}", error);
                    println!("   Please try again or check your Signal CLI configuration.");
                    break;
                }
                LinkingStatus::Linking { device_id } => {
                    println!("   📱 Device scanned! Completing setup...");
                    if let Some(id) = device_id {
                        println!("   Device ID: {}", id);
                    }
                }
                LinkingStatus::Cancelled => {
                    println!("\n❌ Device linking was cancelled");
                    break;
                }
                LinkingStatus::WaitingForScan { .. } => {
                    if attempts >= max_attempts {
                        println!("\n❌ Device linking timed out after 10 minutes.");
                        println!("   Please try again or check your Signal CLI configuration.");
                        manager.cancel_linking().await?;
                        break;
                    }
                    if attempts % 12 == 0 { // Every minute
                        println!("   Still waiting... ({} minutes elapsed)", attempts / 12);
                        if attempts % 24 == 0 { // Every 2 minutes
                            println!("   💡 Tip: Make sure your QR code is still visible and try refreshing the camera");
                        }
                    }
                }
                LinkingStatus::Initializing => {
                    println!("   Initializing...");
                }
            }
        }
        
        Ok(())
    }

    /// Test QR code display functionality
    pub async fn test_qr_display(&self) -> Result<()> {
        println!("🧪 Testing QR code display functionality...");
        
        test_qr_display()?;
        
        println!("\n✅ QR code display test completed successfully!");
        Ok(())
    }

    /// Check current linking status
    pub async fn check_linking_status(&self) -> Result<()> {
        println!("🔍 Checking Signal CLI linking status...");
        
        // Try to determine if signal-cli is linked by checking for configuration
        let config = DeviceLinkConfig::default();
        
        // Check if signal-cli executable exists
        let signal_cli_check = std::process::Command::new(&config.signal_cli_path)
            .arg("--version")
            .output();
            
        match signal_cli_check {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ Signal CLI is installed and accessible");
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("   Version: {}", version.trim());
                    
                    // Try to list accounts to see if any are linked
                    let accounts_check = std::process::Command::new(&config.signal_cli_path)
                        .arg("listAccounts")
                        .output();
                        
                    match accounts_check {
                        Ok(accounts_output) => {
                            let accounts_str = String::from_utf8_lossy(&accounts_output.stdout);
                            if accounts_str.trim().is_empty() {
                                println!("❌ No Signal accounts found");
                                println!("   Run 'cargo run -- signal-link --quick' to link your device");
                            } else {
                                println!("✅ Signal CLI has linked accounts:");
                                for line in accounts_str.lines() {
                                    if !line.trim().is_empty() {
                                        println!("   📱 {}", line.trim());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Could not check accounts: {}", e);
                            println!("   Signal CLI might not be properly configured");
                        }
                    }
                } else {
                    println!("❌ Signal CLI version check failed");
                    let error = String::from_utf8_lossy(&output.stderr);
                    if !error.trim().is_empty() {
                        println!("   Error: {}", error.trim());
                    }
                }
            }
            Err(e) => {
                println!("❌ Signal CLI is not accessible: {}", e);
                println!("   Please ensure signal-cli is installed and in your PATH");
                println!("   Installation instructions: https://github.com/AsamK/signal-cli");
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_creation() {
        let cmd = SignalLinkCommand::command();
        assert_eq!(cmd.get_name(), "signal-link");
    }
    
    #[test]
    fn test_new_command() {
        let cmd = SignalLinkCommand::new();
        // Just test that we can create it without panic
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_qr_generation() {
        let result = test_qr_display();
        assert!(result.is_ok());
    }
}
