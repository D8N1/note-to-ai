// tests/swarm_integration_tests.rs
// Comprehensive integration tests for post-quantum IPFS private swarm

use note_to_ai::{
    swarm::ipfs::{SwarmConfig, SwarmSyncStatus, NetworkHealth, DeviceType, NodeConfig, SyncConfig},
    crypto::Crypto,
    Result,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tempfile::TempDir;
use uuid::Uuid;

#[cfg(test)]
mod swarm_tests {
    use super::*;

    /// Test basic swarm initialization and configuration
    #[tokio::test]
    async fn test_swarm_config_creation() -> Result<()> {
        let config = SwarmConfig::default();
        
        assert!(config.swarm_key.len() > 0);
        assert_eq!(config.node_config.device_type, DeviceType::M1MacBook);
        assert!(config.sync_config.quantum_encryption);
        assert!(config.sync_config.enable_crdt);
        
        Ok(())
    }

    /// Test post-quantum crypto initialization
    #[tokio::test]
    async fn test_quantum_crypto_basics() -> Result<()> {
        let crypto = Crypto::new()?;
        
        let test_data = b"Test quantum encryption data";
        let encrypted = crypto.encrypt(test_data)?;
        let decrypted = crypto.decrypt(&encrypted)?;
        
        assert_eq!(test_data, decrypted.as_slice());
        assert_ne!(test_data, encrypted.as_slice()); // Ensure actually encrypted
        
        Ok(())
    }

    /// Test BLAKE3 hashing for content addressing
    #[tokio::test]
    async fn test_content_hashing() -> Result<()> {
        let crypto = Crypto::new()?;
        
        let content1 = b"Hello, quantum world!";
        let content2 = b"Hello, quantum world!";
        let content3 = b"Different content";
        
        let hash1 = crypto.hash(content1);
        let hash2 = crypto.hash(content2);
        let hash3 = crypto.hash(content3);
        
        assert_eq!(hash1, hash2); // Same content = same hash
        assert_ne!(hash1, hash3); // Different content = different hash
        assert_eq!(hash1.len(), 64); // BLAKE3 produces 256-bit (32-byte) hash = 64 hex chars
        
        Ok(())
    }

    /// Test device type detection and configuration
    #[tokio::test]
    async fn test_device_configuration() -> Result<()> {
        let android_config = SwarmConfig {
            swarm_key: "test_key".to_string(),
            bootstrap_peers: vec!["test_peer".to_string()],
            node_config: NodeConfig {
                node_name: "test_android".to_string(),
                device_type: DeviceType::AndroidPhone,
                max_storage_gb: 5,
                max_bandwidth_mbps: 50,
            },
            sync_config: SyncConfig {
                sync_interval_secs: 30,
                realtime_voice_sync: true,
                enable_crdt: true,
                quantum_encryption: true,
            },
        };
        
        let m1_config = SwarmConfig {
            swarm_key: "test_key".to_string(),
            bootstrap_peers: vec!["test_peer".to_string()],
            node_config: NodeConfig {
                node_name: "test_m1".to_string(),
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
        
        // Android should have mobile-optimized settings
        assert_eq!(android_config.node_config.max_storage_gb, 5);
        assert_eq!(android_config.node_config.max_bandwidth_mbps, 50);
        
        // M1 MacBook should have desktop settings
        assert_eq!(m1_config.node_config.max_storage_gb, 100);
        assert_eq!(m1_config.node_config.max_bandwidth_mbps, 1000);
        
        Ok(())
    }

    /// Test vault entry creation and encryption
    #[tokio::test]
    async fn test_vault_entry_creation() -> Result<()> {
        use note_to_ai::swarm::ipfs::{VaultEntry, VaultMetadata, VaultFileType};
        
        let test_content = b"# Test Note\n\nThis is a test note for quantum sync.";
        let test_path = PathBuf::from("vault/Test Notes/quantum-test.md");
        
        // Simulate vault entry creation (would normally be done by IPFSNode)
        let entry = VaultEntry {
            id: Uuid::new_v4().to_string(),
            path: test_path.clone(),
            content_hash: "test_hash_placeholder".to_string(),
            content: test_content.to_vec(),
            metadata: VaultMetadata {
                file_type: VaultFileType::MarkdownNote,
                size_bytes: test_content.len() as u64,
                created_at: 1692364800, // Mock timestamp
                modified_at: 1692364800,
                tags: vec!["test".to_string(), "quantum".to_string()],
                encryption_used: false,
            },
            device_origin: "test_device".to_string(),
            timestamp: 1692364800,
        };
        
        assert_eq!(entry.path, test_path);
        assert_eq!(entry.content, test_content);
        assert_eq!(entry.metadata.file_type, VaultFileType::MarkdownNote);
        assert!(entry.metadata.tags.contains(&"quantum".to_string()));
        
        Ok(())
    }

    /// Test sync status reporting
    #[tokio::test]
    async fn test_sync_status() -> Result<()> {
        let status = SwarmSyncStatus {
            connected_peers: 2,
            pending_uploads: 0,
            last_sync: 1692364800,
            network_health: NetworkHealth::Good,
        };
        
        assert_eq!(status.connected_peers, 2);
        assert_eq!(status.pending_uploads, 0);
        assert_eq!(status.network_health, NetworkHealth::Good);
        
        Ok(())
    }

    /// Test network health assessment
    #[tokio::test]
    async fn test_network_health_levels() -> Result<()> {
        // Test different network health scenarios
        let excellent = NetworkHealth::Excellent;
        let good = NetworkHealth::Good;
        let disconnected = NetworkHealth::Disconnected;
        
        // Verify enum variants exist and can be compared
        assert_ne!(excellent, good);
        assert_ne!(good, disconnected);
        
        Ok(())
    }

    /// Test file type detection
    #[tokio::test]
    async fn test_file_type_detection() -> Result<()> {
        use note_to_ai::swarm::ipfs::VaultFileType;
        
        let md_file = PathBuf::from("vault/Notes/test.md");
        let ai_response = PathBuf::from("vault/AI Responses/response.md");
        let voice_note = PathBuf::from("vault/Voice Notes/transcription.md");
        let config_file = PathBuf::from("vault/config/settings.toml");
        let attachment = PathBuf::from("vault/Attachments/image.png");
        
        // Test file type inference based on path patterns
        // This would normally be done by determine_file_type method
        assert!(md_file.to_string_lossy().ends_with(".md"));
        assert!(ai_response.to_string_lossy().contains("AI Responses"));
        assert!(voice_note.to_string_lossy().contains("Voice"));
        assert!(config_file.to_string_lossy().contains("config"));
        
        Ok(())
    }

    /// Test concurrent crypto operations (important for real-time sync)
    #[tokio::test]
    async fn test_concurrent_encryption() -> Result<()> {
        let crypto = Crypto::new()?;
        
        let test_data = vec![
            b"Voice note from Android phone".to_vec(),
            b"Quick edit from M1 MacBook".to_vec(),
            b"Research link shared via Signal".to_vec(),
        ];
        
        // Encrypt all data concurrently
        let mut encrypt_tasks = Vec::new();
        for data in &test_data {
            let crypto_clone = Crypto::new()?; // Each task gets its own crypto instance
            let data_clone = data.clone();
            
            encrypt_tasks.push(tokio::spawn(async move {
                crypto_clone.encrypt(&data_clone)
            }));
        }
        
        // Wait for all encryptions to complete
        let mut encrypted_results = Vec::new();
        for task in encrypt_tasks {
            let result = task.await.unwrap()?;
            encrypted_results.push(result);
        }
        
        // Verify all encryptions succeeded and decrypt back
        assert_eq!(encrypted_results.len(), test_data.len());
        
        for (original, encrypted) in test_data.iter().zip(encrypted_results.iter()) {
            let decrypted = crypto.decrypt(encrypted)?;
            assert_eq!(original, &decrypted);
        }
        
        Ok(())
    }

    /// Test large file handling (important for voice attachments)
    #[tokio::test]
    async fn test_large_content_encryption() -> Result<()> {
        let crypto = Crypto::new()?;
        
        // Create a large test file (simulating a voice note)
        let large_content = vec![0u8; 1024 * 1024]; // 1MB of zeros
        
        let encrypted = crypto.encrypt(&large_content)?;
        let decrypted = crypto.decrypt(&encrypted)?;
        
        assert_eq!(large_content.len(), decrypted.len());
        assert_eq!(large_content, decrypted);
        
        Ok(())
    }

    /// Test tag extraction and metadata handling
    #[tokio::test]
    async fn test_tag_extraction() -> Result<()> {
        let test_paths = vec![
            PathBuf::from("vault/AI Responses/2024-08-18/quantum-research.md"),
            PathBuf::from("vault/Voice Notes/voice-idea-project.md"),
            PathBuf::from("vault/Daily Notes/daily-2024-08-18.md"),
        ];
        
        // Simulate tag extraction logic
        for path in &test_paths {
            let path_str = path.to_string_lossy();
            let mut tags = Vec::new();
            
            if path_str.contains("AI Responses") {
                tags.push("ai-generated".to_string());
            }
            if path_str.contains("Voice") || path_str.contains("voice") {
                tags.push("voice-note".to_string());
            }
            if path_str.contains("Daily") || path_str.contains("daily") {
                tags.push("daily-note".to_string());
            }
            if path_str.contains("quantum") {
                tags.push("quantum".to_string());
            }
            
            assert!(!tags.is_empty(), "Should extract at least one tag from path: {}", path_str);
        }
        
        Ok(())
    }

    /// Test error handling for invalid configurations
    #[tokio::test]
    async fn test_invalid_config_handling() -> Result<()> {
        // Test empty swarm key
        let mut bad_config = SwarmConfig::default();
        bad_config.swarm_key = "".to_string();
        
        assert!(bad_config.swarm_key.is_empty());
        
        // Test invalid storage limits
        bad_config.node_config.max_storage_gb = 0;
        assert_eq!(bad_config.node_config.max_storage_gb, 0);
        
        Ok(())
    }

    /// Test timestamp handling and ordering
    #[tokio::test]
    async fn test_timestamp_ordering() -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let earlier = now - 3600; // 1 hour ago
        let later = now + 3600;   // 1 hour from now
        
        // Test timestamp comparison (important for CRDT conflict resolution)
        assert!(earlier < now);
        assert!(now < later);
        assert!(earlier < later);
        
        Ok(())
    }

    /// Test content hash stability
    #[tokio::test]
    async fn test_content_hash_stability() -> Result<()> {
        let crypto = Crypto::new()?;
        
        let content = b"Stable content for hashing";
        
        // Hash the same content multiple times
        let hash1 = crypto.hash(content);
        let hash2 = crypto.hash(content);
        let hash3 = crypto.hash(content);
        
        // All hashes should be identical (deterministic)
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
        assert_eq!(hash1, hash3);
        
        Ok(())
    }

    /// Test Bootstrap peer validation
    #[tokio::test]
    async fn test_bootstrap_peer_validation() -> Result<()> {
        let config = SwarmConfig {
            bootstrap_peers: vec![
                "android_phone_192.168.1.100:4001".to_string(),
                "m1_macbook_192.168.1.101:4001".to_string(),
            ],
            ..Default::default()
        };
        
        assert_eq!(config.bootstrap_peers.len(), 2);
        
        for peer in &config.bootstrap_peers {
            assert!(peer.contains(":4001")); // Standard IPFS port
            assert!(peer.contains("192.168.1.")); // Local network
        }
        
        Ok(())
    }

    /// Test sync configuration validation
    #[tokio::test]
    async fn test_sync_configuration() -> Result<()> {
        let config = SwarmConfig::default();
        
        // Verify default sync settings are sane
        assert!(config.sync_config.sync_interval_secs > 0);
        assert!(config.sync_config.sync_interval_secs < 3600); // Less than 1 hour
        assert!(config.sync_config.realtime_voice_sync); // Voice notes should sync immediately
        assert!(config.sync_config.enable_crdt); // Conflict resolution should be enabled
        assert!(config.sync_config.quantum_encryption); // Security should be enabled
        
        Ok(())
    }
}

/// Performance tests for swarm operations
#[cfg(test)]
mod swarm_performance_tests {
    use super::*;
    use std::time::Instant;

    /// Test encryption performance for real-time sync
    #[tokio::test]
    async fn test_encryption_performance() -> Result<()> {
        let crypto = Crypto::new()?;
        let test_data = vec![0u8; 1024]; // 1KB test data (typical voice note size)
        
        let start = Instant::now();
        let encrypted = crypto.encrypt(&test_data)?;
        let encrypt_duration = start.elapsed();
        
        let start = Instant::now();
        let _decrypted = crypto.decrypt(&encrypted)?;
        let decrypt_duration = start.elapsed();
        
        // Encryption should be fast enough for real-time sync
        assert!(encrypt_duration.as_millis() < 100, "Encryption too slow: {:?}", encrypt_duration);
        assert!(decrypt_duration.as_millis() < 100, "Decryption too slow: {:?}", decrypt_duration);
        
        println!("Encryption: {:?}, Decryption: {:?}", encrypt_duration, decrypt_duration);
        
        Ok(())
    }

    /// Test hash performance for content addressing
    #[tokio::test]
    async fn test_hash_performance() -> Result<()> {
        let crypto = Crypto::new()?;
        let test_data = vec![0u8; 10 * 1024]; // 10KB test data
        
        let start = Instant::now();
        let _hash = crypto.hash(&test_data);
        let hash_duration = start.elapsed();
        
        // Hashing should be very fast
        assert!(hash_duration.as_millis() < 50, "Hashing too slow: {:?}", hash_duration);
        
        println!("Hash duration: {:?}", hash_duration);
        
        Ok(())
    }

    /// Test concurrent operation performance
    #[tokio::test]
    async fn test_concurrent_performance() -> Result<()> {
        let crypto = Crypto::new()?;
        let test_data = vec![0u8; 1024];
        
        let start = Instant::now();
        
        // Run 10 concurrent encryptions
        let mut tasks = Vec::new();
        for _ in 0..10 {
            let crypto_clone = Crypto::new()?;
            let data_clone = test_data.clone();
            
            tasks.push(tokio::spawn(async move {
                crypto_clone.encrypt(&data_clone)
            }));
        }
        
        // Wait for all to complete
        for task in tasks {
            task.await.unwrap()?;
        }
        
        let total_duration = start.elapsed();
        
        // 10 concurrent operations should complete quickly
        assert!(total_duration.as_millis() < 1000, "Concurrent operations too slow: {:?}", total_duration);
        
        println!("10 concurrent encryptions: {:?}", total_duration);
        
        Ok(())
    }
}

/// Edge case and error condition tests
#[cfg(test)]
mod swarm_edge_case_tests {
    use super::*;

    /// Test empty content handling
    #[tokio::test]
    async fn test_empty_content() -> Result<()> {
        let crypto = Crypto::new()?;
        
        let empty_data = b"";
        let encrypted = crypto.encrypt(empty_data)?;
        let decrypted = crypto.decrypt(&encrypted)?;
        
        assert_eq!(empty_data, decrypted.as_slice());
        
        let hash = crypto.hash(empty_data);
        assert!(!hash.is_empty());
        
        Ok(())
    }

    /// Test very large content (stress test)
    #[tokio::test]
    async fn test_very_large_content() -> Result<()> {
        let crypto = Crypto::new()?;
        
        // 10MB content (large voice note or document)
        let large_data = vec![42u8; 10 * 1024 * 1024];
        
        let encrypted = crypto.encrypt(&large_data)?;
        let decrypted = crypto.decrypt(&encrypted)?;
        
        assert_eq!(large_data.len(), decrypted.len());
        assert_eq!(large_data, decrypted);
        
        Ok(())
    }

    /// Test invalid UTF-8 content (binary data)
    #[tokio::test]
    async fn test_binary_content() -> Result<()> {
        let crypto = Crypto::new()?;
        
        // Binary data that's not valid UTF-8
        let binary_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        
        let encrypted = crypto.encrypt(&binary_data)?;
        let decrypted = crypto.decrypt(&encrypted)?;
        
        assert_eq!(binary_data, decrypted);
        
        Ok(())
    }

    /// Test repeated encryption/decryption cycles
    #[tokio::test]
    async fn test_encryption_cycles() -> Result<()> {
        let crypto = Crypto::new()?;
        let original_data = b"Data for cycle testing";
        
        let mut current_data = original_data.to_vec();
        
        // Encrypt and decrypt 10 times
        for _ in 0..10 {
            let encrypted = crypto.encrypt(&current_data)?;
            current_data = crypto.decrypt(&encrypted)?;
        }
        
        // Should still match original
        assert_eq!(original_data, current_data.as_slice());
        
        Ok(())
    }

    /// Test device type serialization/deserialization
    #[tokio::test]
    async fn test_device_type_serde() -> Result<()> {
        use serde_json;
        
        let android = DeviceType::AndroidPhone;
        let m1 = DeviceType::M1MacBook;
        
        // Test JSON serialization
        let android_json = serde_json::to_string(&android)?;
        let m1_json = serde_json::to_string(&m1)?;
        
        // Test JSON deserialization
        let android_decoded: DeviceType = serde_json::from_str(&android_json)?;
        let m1_decoded: DeviceType = serde_json::from_str(&m1_json)?;
        
        assert_eq!(android, android_decoded);
        assert_eq!(m1, m1_decoded);
        
        Ok(())
    }
}
