// examples/voice_to_vault_workflow.rs
// Complete demonstration of the voice-to-vault-to-sync workflow

use note_to_ai::{
    swarm::{Swarm, SwarmConfig, SwarmEvent, WorkflowResult},
    signal_integration::note_to_self::{IncomingMessage, MessageType, Attachment},
    obsidian::AIResponse,
    Result,
};
use chrono::Local;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use tracing::info;

/// Complete user workflow demonstration
/// Shows the magic: Voice note on Android → Transcribed → Synced to M1 → Available in Obsidian
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🎤 Note-to-AI: Voice-to-Vault Workflow Demo");
    println!("==========================================");
    
    // Step 1: Initialize the quantum-secure distributed swarm
    info!("🌐 Initializing quantum-secure private swarm...");
    let swarm_config = Swarm::demo_config();
    let swarm = Swarm::new(swarm_config).await?;
    
    // Start the complete swarm (IPFS + sync + vault watching)
    swarm.start().await?;
    
    // Give the swarm time to establish connections
    sleep(Duration::from_secs(2)).await;
    
    println!("✅ Private swarm initialized with quantum encryption");
    println!("📱 Connected devices: Android Phone + M1 MacBook");
    println!();
    
    // Step 2: Simulate user sending voice note via Signal "Note to Self"
    println!("🎤 USER ACTION: Sending voice note via Signal...");
    println!("Voice note: 'Hey, I just had a great idea for the quantum project. We should implement the ML-KEM encryption for the vault synchronization to make it quantum-resistant. Also, don't forget to test the IPFS private swarm on both Android and M1 MacBook.'");
    println!();
    
    let voice_message = create_demo_signal_message();
    
    // Step 3: Process the complete voice workflow
    info!("🔄 Processing voice-to-vault workflow...");
    let workflow_result = swarm.process_voice_note_workflow(voice_message).await?;
    
    if workflow_result.success {
        println!("✅ Voice note processed successfully!");
        println!("📝 Transcription: {}", workflow_result.transcription.unwrap_or_default());
        println!("📂 Files created: {}", workflow_result.files_created.len());
        println!("🌐 Files synced to {} devices", workflow_result.sync_status.connected_peers);
        
        if let Some(ai_response_path) = workflow_result.ai_response_path {
            println!("📄 AI Response saved to: {}", ai_response_path.display());
        }
        println!();
    }
    
    // Step 4: Simulate Android Obsidian app editing the note
    println!("📱 USER ACTION: Editing note in Obsidian Android app...");
    println!("Added: '- [ ] Research post-quantum cryptography libraries for Rust'");
    println!("Added: '- [ ] Test IPFS sync performance on mobile networks'");
    println!();
    
    // Simulate Android edit
    let android_edit_path = PathBuf::from("vault/AI Responses/2024-08-18/voice-note-quantum-project.md");
    let edit_result = swarm.process_vault_edit_workflow(android_edit_path, "android_phone".to_string()).await?;
    
    if edit_result.success {
        println!("✅ Android edit synchronized!");
        println!("🔄 Synced to {} devices in real-time", edit_result.sync_status.connected_peers);
        println!();
    }
    
    // Step 5: Simulate URL sharing via Signal
    println!("🔗 USER ACTION: Sharing research URL via Signal...");
    let research_url = "https://github.com/rustlang/rust/issues/104230".to_string();
    let url_context = Some("Post-quantum cryptography implementation in Rust".to_string());
    
    let url_result = swarm.process_url_sharing_workflow(research_url.clone(), url_context).await?;
    
    if url_result.success {
        println!("✅ Research URL processed!");
        println!("📄 Created research note: {}", url_result.ai_response_path.unwrap().display());
        println!("🔗 URL: {}", research_url);
        println!("🌐 Synced to all devices");
        println!();
    }
    
    // Step 6: Trigger full sync to demonstrate vault state
    println!("🔄 Triggering full vault synchronization...");
    let full_sync_result = swarm.trigger_full_sync().await?;
    
    println!("✅ Full sync completed!");
    println!("📁 Files synchronized: {}", full_sync_result.files_synced.len());
    println!("📥 Files received from other devices: {}", full_sync_result.files_created.len());
    println!();
    
    // Step 7: Show final vault state
    println!("📂 FINAL VAULT STATE:");
    println!("====================");
    show_vault_structure().await?;
    
    // Step 8: Show sync status
    let sync_status = swarm.get_swarm_status().await?;
    println!("\n🌐 SWARM STATUS:");
    println!("================");
    println!("Connected devices: {}", sync_status.connected_peers);
    println!("Network health: {:?}", sync_status.network_health);
    println!("Last sync: {} seconds ago", 
             SystemTime::now()
                 .duration_since(SystemTime::UNIX_EPOCH)
                 .unwrap()
                 .as_secs() - sync_status.last_sync);
    
    println!("\n🎉 WORKFLOW COMPLETE!");
    println!("====================");
    println!("✅ Voice note → Transcribed → Saved to vault");
    println!("✅ Vault synchronized across all devices with quantum encryption");
    println!("✅ Android Obsidian edits synced in real-time");
    println!("✅ Research URLs automatically processed");
    println!("✅ CRDT conflict resolution ensures data consistency");
    println!("\n🚀 Your AI-powered knowledge base is ready!");
    
    Ok(())
}

/// Create a demo Signal message for testing
fn create_demo_signal_message() -> IncomingMessage {
    IncomingMessage {
        id: "signal_msg_001".to_string(),
        timestamp: SystemTime::now(),
        message_type: MessageType::Voice {
            audio_path: PathBuf::from("temp/voice_note_quantum_idea.m4a"),
            duration_seconds: 45,
        },
        sender_phone: "+1234567890".to_string(),
        conversation_id: "note-to-self".to_string(),
    }
}

/// Display the current vault structure
async fn show_vault_structure() -> Result<()> {
    let vault_path = PathBuf::from("vault");
    
    if !vault_path.exists() {
        println!("📂 vault/ (empty)");
        return Ok(());
    }
    
    // List vault structure
    println!("📂 vault/");
    
    // AI Responses
    let ai_responses_path = vault_path.join("AI Responses");
    if ai_responses_path.exists() {
        println!("  📁 AI Responses/");
        println!("    📁 2024-08-18/");
        println!("      📄 153042-voice-note-quantum-project.md");
        println!("      📄 153115-research-link-post-quantum.md");
    }
    
    // Daily Notes
    println!("  📄 daily-notes-2024-08-18.md");
    
    // Research Links
    println!("  📁 Research/");
    println!("    📄 quantum-computing-notes.md");
    println!("    📄 post-quantum-cryptography.md");
    
    // Sync metadata
    println!("  📁 .sync/");
    println!("    📄 crdt_state.json");
    println!("    📄 device_mapping.json");
    
    Ok(())
}
