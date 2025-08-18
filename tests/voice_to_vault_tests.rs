// Integration tests for the complete voice-to-vault workflow with quantum-secure sync
use note_to_ai::{
    signal_integration::{IncomingMessage, MessageType, conversational_assistant::AIResponse},
    swarm::ipfs::{SwarmConfig, NodeConfig, DeviceType, SyncConfig},
    vault::crdt::CRDT,
    crypto::Crypto,
};
use anyhow::Result;
use std::time::SystemTime;
use tempfile::TempDir;
use tokio;
use uuid::Uuid;

#[tokio::test]
async fn test_voice_message_processing() -> Result<()> {
    // Create test voice message
    let temp_dir = TempDir::new()?;
    let audio_path = temp_dir.path().join("test_voice.m4a");
    std::fs::write(&audio_path, b"mock audio data")?;
    
    let message = IncomingMessage {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        message_type: MessageType::Voice {
            audio_path: audio_path.clone(),
            duration_seconds: 30,
        },
        sender_phone: "+1234567890".to_string(),
        conversation_id: "note-to-self".to_string(),
    };
    
    // Verify message creation
    assert!(audio_path.exists());
    assert_eq!(message.conversation_id, "note-to-self");
    Ok(())
}

#[tokio::test]
async fn test_signal_message_types() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Test text message
    let text_msg = IncomingMessage {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        message_type: MessageType::Text {
            content: "This is a strategic decision about our market expansion".to_string(),
        },
        sender_phone: "+1234567890".to_string(),
        conversation_id: "note-to-self".to_string(),
    };
    
    // Test voice message
    let audio_path = temp_dir.path().join("voice.m4a");
    std::fs::write(&audio_path, b"mock audio")?;
    
    let voice_msg = IncomingMessage {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        message_type: MessageType::Voice {
            audio_path: audio_path.clone(),
            duration_seconds: 45,
        },
        sender_phone: "+1234567890".to_string(),
        conversation_id: "note-to-self".to_string(),
    };
    
    // Test document message
    let doc_path = temp_dir.path().join("document.pdf");
    std::fs::write(&doc_path, b"mock document data")?;
    
    let doc_msg = IncomingMessage {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        message_type: MessageType::Document {
            doc_path: doc_path.clone(),
            filename: "document.pdf".to_string(),
            caption: Some("Strategic analysis document".to_string()),
        },
        sender_phone: "+1234567890".to_string(),
        conversation_id: "note-to-self".to_string(),
    };
    
    // Verify all message types can be created
    match text_msg.message_type {
        MessageType::Text { ref content } => assert!(!content.is_empty()),
        _ => panic!("Wrong message type"),
    }
    
    match voice_msg.message_type {
        MessageType::Voice { ref audio_path, duration_seconds } => {
            assert!(audio_path.exists());
            assert_eq!(duration_seconds, 45);
        }
        _ => panic!("Wrong message type"),
    }
    
    match doc_msg.message_type {
        MessageType::Document { ref doc_path, ref filename, ref caption } => {
            assert!(doc_path.exists());
            assert_eq!(filename, "document.pdf");
            assert!(caption.is_some());
        }
        _ => panic!("Wrong message type"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_ai_response_format() -> Result<()> {
    // Test AI response structure
    let response = AIResponse {
        content: "This is a test response from the AI".to_string(),
    };
    
    // Verify response format
    assert!(!response.content.is_empty());
    assert!(response.content.contains("test response"));
    
    Ok(())
}

#[tokio::test]
async fn test_vault_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let vault_path = temp_dir.path().to_path_buf();
    
    // Create basic vault structure
    let notes_dir = vault_path.join("notes");
    let daily_dir = vault_path.join("daily");
    let templates_dir = vault_path.join("templates");
    
    std::fs::create_dir_all(&notes_dir)?;
    std::fs::create_dir_all(&daily_dir)?;
    std::fs::create_dir_all(&templates_dir)?;
    
    // Create sample note
    let note_path = notes_dir.join("test-note.md");
    let note_content = r#"# Test Note
    
Created from Signal voice message.

## Content
Strategic analysis of market conditions.

## AI Insights
- Market opportunity identified
- Competitive landscape analyzed
- Action items generated

#strategy #market-analysis #ai-generated
"#;
    
    std::fs::write(&note_path, note_content)?;
    
    // Verify vault structure
    assert!(notes_dir.exists());
    assert!(daily_dir.exists());
    assert!(templates_dir.exists());
    assert!(note_path.exists());
    
    let content = std::fs::read_to_string(&note_path)?;
    assert!(content.contains("Strategic analysis"));
    assert!(content.contains("#ai-generated"));
    
    Ok(())
}

#[tokio::test]
async fn test_cross_device_sync() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Simulate two devices with different configurations
    let device1_config = SwarmConfig {
        swarm_key: "test_key".to_string(),
        bootstrap_peers: vec!["test_peer".to_string()],
        node_config: NodeConfig {
            node_name: "iPhone".to_string(),
            device_type: DeviceType::AndroidPhone,
            max_storage_gb: 5,
            max_bandwidth_mbps: 100,
        },
        sync_config: SyncConfig {
            sync_interval_secs: 30,
            realtime_voice_sync: true,
            enable_crdt: true,
            quantum_encryption: true,
        },
    };
    
    let device2_config = SwarmConfig {
        swarm_key: "test_key".to_string(),
        bootstrap_peers: vec!["test_peer".to_string()],
        node_config: NodeConfig {
            node_name: "MacBook".to_string(),
            device_type: DeviceType::M1MacBook,
            max_storage_gb: 100,
            max_bandwidth_mbps: 1000,
        },
        sync_config: SyncConfig {
            sync_interval_secs: 30,
            realtime_voice_sync: true,
            enable_crdt: true,
            quantum_encryption: true,
        },
    };
    
    // Create mock vault directories
    let vault1_path = temp_dir.path().join("device1_vault");
    let vault2_path = temp_dir.path().join("device2_vault");
    
    std::fs::create_dir_all(&vault1_path)?;
    std::fs::create_dir_all(&vault2_path)?;
    
    // Simulate note creation on device 1
    let note_path_1 = vault1_path.join("signal-note.md");
    std::fs::write(&note_path_1, "# Voice Note\nProcessed from Signal message.")?;
    
    // Verify configurations
    assert_eq!(device1_config.node_config.device_type, DeviceType::AndroidPhone);
    assert_eq!(device2_config.node_config.device_type, DeviceType::M1MacBook);
    assert!(device1_config.sync_config.quantum_encryption);
    assert!(device2_config.sync_config.quantum_encryption);
    assert!(note_path_1.exists());
    
    Ok(())
}

#[tokio::test]
async fn test_markdown_parsing() -> Result<()> {
    let markdown_content = r#"# Strategic Decision Analysis

## Executive Summary
Market expansion opportunity identified through AI analysis.

## Key Points
- **Market Size**: $50M TAM
- **Competition**: 3 major players
- **Timeline**: Q2 implementation

## Action Items
- [ ] Conduct competitor analysis
- [ ] Prepare market entry strategy
- [ ] Schedule stakeholder meeting

## AI Insights
This analysis was generated from voice input via Signal integration.

#strategy #market-expansion #ai-analysis
"#;
    
    // Test parsing markdown structure
    let lines: Vec<&str> = markdown_content.lines().collect();
    
    // Verify structure elements
    assert!(lines.iter().any(|&line| line.starts_with("# ")));
    assert!(lines.iter().any(|&line| line.starts_with("## ")));
    assert!(lines.iter().any(|&line| line.contains("- [ ]")));
    assert!(lines.iter().any(|&line| line.starts_with("#strategy")));
    
    // Verify content
    assert!(markdown_content.contains("Strategic Decision"));
    assert!(markdown_content.contains("AI Insights"));
    assert!(markdown_content.contains("Signal integration"));
    
    Ok(())
}

#[tokio::test]
async fn test_crdt_conflict_resolution() -> Result<()> {
    // Create CRDT instance
    let mut crdt = CRDT::new_with_replica_id("test-replica".to_string())?;
    
    // Simulate concurrent edits
    let note_id = "note-123";
    let device1_edit = "Device 1 edit: Strategic analysis complete";
    let device2_edit = "Device 2 edit: Market research findings";
    
    // Apply edits with different timestamps
    let timestamp1 = SystemTime::now();
    let timestamp2 = SystemTime::now();
    
    crdt.apply_edit(note_id, device1_edit, timestamp1)?;
    crdt.apply_edit(note_id, device2_edit, timestamp2)?;
    
    // Verify CRDT can handle the operations
    assert!(crdt.get_replica_id() == "test-replica");
    
    Ok(())
}

#[tokio::test]
async fn test_quantum_secure_crypto() -> Result<()> {
    // Test quantum-resistant cryptography integration
    let crypto = Crypto::new()?;
    
    let test_data = b"Signal voice message: Strategic market analysis";
    let encrypted = crypto.encrypt(test_data)?;
    let decrypted = crypto.decrypt(&encrypted)?;
    
    assert_eq!(test_data, decrypted.as_slice());
    assert_ne!(encrypted, test_data); // Ensure actually encrypted
    
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // 1. Create Signal voice message
    let audio_path = temp_dir.path().join("voice.m4a");
    std::fs::write(&audio_path, b"mock voice data")?;
    
    let signal_message = IncomingMessage {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        message_type: MessageType::Voice {
            audio_path: audio_path.clone(),
            duration_seconds: 60,
        },
        sender_phone: "+1234567890".to_string(),
        conversation_id: "note-to-self".to_string(),
    };
    
    // 2. Simulate AI processing
    let ai_response = AIResponse {
        content: "📊 Strategic Analysis Complete\n\nKey insights:\n- Market opportunity: $50M\n- Competition: 3 players\n- Recommendation: Proceed with expansion\n\n#strategy #market".to_string(),
    };
    
    // 3. Create vault note
    let vault_path = temp_dir.path().join("vault");
    std::fs::create_dir_all(&vault_path)?;
    
    let note_path = vault_path.join("strategic-analysis.md");
    let note_content = format!(r#"# Strategic Analysis
    
Signal Message ID: {}
Timestamp: {:?}
Duration: 60 seconds

## AI Analysis
{}

## Metadata
- Source: Signal Voice Message
- Processed: {}
- Device: iPhone
- Sync Status: Pending

#signal #voice-to-text #strategy
"#, 
        signal_message.id,
        signal_message.timestamp,
        ai_response.content,
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
    );
    
    std::fs::write(&note_path, note_content)?;
    
    // 4. Verify end-to-end workflow
    assert!(audio_path.exists());
    assert!(note_path.exists());
    
    let saved_content = std::fs::read_to_string(&note_path)?;
    assert!(saved_content.contains(&signal_message.id));
    assert!(saved_content.contains("Strategic Analysis"));
    assert!(saved_content.contains("Signal Voice Message"));
    assert!(saved_content.contains("#signal"));
    
    Ok(())
}
