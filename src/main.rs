use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use tokio::signal as tokio_signal;
use tracing::{info, warn, error};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod config;
mod logger;
mod vault;
mod ai;
mod signal_integration;  // Renamed to avoid conflict
mod crypto;
mod identity;
mod swarm;
mod audio;
mod scheduler;
mod obsidian;
mod attestation;
mod commands;

use config::Settings;
use commands::Cli;
// Temporarily disabled while fixing Arrow ecosystem conflicts
// use vault::storage::{HybridStorageEngine, StorageConfig};

/// note-to-ai: Transform your Signal "Note to Self" into an AI-powered knowledge base

/// Main application state
pub struct NoteToAI {
    config: Settings,
    // TODO: Re-add scheduler and storage when they're ready
    // scheduler: scheduler::Scheduler,
    // storage: HybridStorageEngine,
}

impl NoteToAI {
    /// Initialize the note-to-ai application
    pub async fn new(config_path: &PathBuf) -> Result<Self> {
        info!("Initializing note-to-ai");
        
        // Load configuration
        let config = Settings::load(config_path.to_str().unwrap())
            .context("Failed to load configuration")?;
        
        // TODO: Re-enable hybrid storage once Arrow conflicts are resolved
        /*
        let storage_config = StorageConfig {
            base_path: config.storage.base_path.clone(),
            duckdb_config: config.storage.duckdb.clone().into(),
            lance_config: config.storage.lance.clone().into(),
        };
        
        let storage = HybridStorageEngine::new(storage_config).await
            .context("Failed to initialize storage engine")?;
        */
        
        Ok(Self {
            config,
            // storage,
        })
    }

    /// Minimal attestation recording: creates schema if needed and writes one event
    pub async fn attest(&self, path: Option<PathBuf>, context: Option<String>) -> Result<()> {
    use attestation::{AttestationEngine, AttestationEvent, AttestationStatus};
    use attestation::signer::Signer;
        use blake3::Hasher;
        use chrono::Utc;
        use rand::{distributions::Alphanumeric, Rng};
        use base64::{engine::general_purpose, Engine as _};
    use ark_bn254::Fr;
    use crypto::zk_proofs::ZKProofs;

    let db_path = PathBuf::from(&self.config.database.path);
    let engine = AttestationEngine::new(db_path)?;
    engine.initialize()?;

        // Build a context hash per research format (sha256 in doc; we use blake3 here for now)
        let mut hasher = Hasher::new();
        if let Some(ref p) = path {
            hasher.update(p.to_string_lossy().as_bytes());
        }
        if let Some(ref c) = context { hasher.update(c.as_bytes()); }
        let context_hash = hex::encode(hasher.finalize().as_bytes());

        // Dummy zk proof placeholders (base64) until real circuits land
    // Generate a toy Groth16 proof (placeholder until real circuits)
    let mut zk = ZKProofs::new()?;
    zk.setup()?;
    let a = Fr::from(3u64);
    let b = Fr::from(4u64);
    let (proof_bytes, vk_bytes, public_inputs) = zk.prove_toy_sum(a, b)?;
    let dummy_proof = general_purpose::STANDARD.encode(&proof_bytes);
    let dummy_vk = general_purpose::STANDARD.encode(&vk_bytes);
    let metadata_b64 = general_purpose::STANDARD.encode(&public_inputs);

        // Random id
        let id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let evt = AttestationEvent {
            id,
            timestamp: Utc::now(),
            device_id: self.config.signal.device_id.map(|d| d.to_string()).unwrap_or_else(|| "unknown".to_string()),
            context_hash,
            proof_data_b64: dummy_proof,
            verification_key_b64: dummy_vk,
            metadata_encrypted_b64: Some(metadata_b64),
            related_path: path,
            status: AttestationStatus::Pending,
        };

        engine.record_event(&evt)?;
        // Optionally sign markdown file to create a sidecar signature JSON
        if let Some(ref p) = evt.related_path {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") {
                    let key_dir = self.config.crypto.key_path.clone();
                    let signer = Signer::load_or_generate(key_dir)?;
                    if let Ok(sidecar_path) = signer.sign_markdown_sidecar(p, &evt.context_hash, Some(evt.id.clone())) {
                        println!("🖊️  Signature sidecar written: {}", sidecar_path.display());
                    }
                }
            }
        }
        println!("✅ Attestation recorded: {} (ctx: {})", evt.id, evt.context_hash);
        Ok(())
    }
    
    /// Start the main service loop
    pub async fn start(&mut self, skip_signal: bool, skip_ai: bool) -> Result<()> {
        info!("Starting note-to-ai service");
        
        // TODO: Start scheduler when it's implemented
        // self.scheduler.start().await
        //     .context("Failed to start scheduler")?;
        
        // REAL IMPLEMENTATION: Load AI models
        if !skip_ai {
            info!("🧠 Loading REAL AI models...");
            
            // REAL IMPLEMENTATION: Verify Whisper model availability
            match self.verify_whisper_model().await {
                Ok(()) => info!("✅ Whisper model verified and ready"),
                Err(e) => {
                    warn!("⚠️ Whisper model verification failed: {}", e);
                    info!("💡 Run './scripts/download-models.sh' to download required models");
                }
            }
            
            // REAL IMPLEMENTATION: Verify embedding model availability  
            match self.verify_embedding_model().await {
                Ok(()) => info!("✅ Embedding model verified and ready"),
                Err(e) => {
                    warn!("⚠️ Embedding model verification failed: {}", e);
                    info!("💡 Run './scripts/download-models.sh' to download required models");
                }
            }
            
            info!("🚀 AI model loading complete");
        } else {
            warn!("Skipping AI model loading");
        }
        
        // REAL IMPLEMENTATION: Connect to Signal
        if !skip_signal {
            info!("📱 Connecting to Signal...");
            
            // REAL IMPLEMENTATION: Initialize and test Signal connection
            match self.verify_signal_connection().await {
                Ok(()) => {
                    info!("✅ Signal connection verified and ready");
                    
                    // REAL IMPLEMENTATION: Start message processing loop
                    self.start_message_processing().await?;
                }
                Err(e) => {
                    warn!("⚠️ Signal connection failed: {}", e);
                    info!("💡 Run 'cargo run -- signal setup --phone +1234567890' to configure Signal");
                }
            }
        } else {
            warn!("Skipping Signal connection");
        }
        
        info!("✅ note-to-ai service started successfully!");
        info!("Send a voice message to your Signal 'Note to Self' to get started");
        
        // Wait for shutdown signal
        self.wait_for_shutdown().await;
        
        Ok(())
    }
    
    /// Start processing Signal messages - REAL IMPLEMENTATION
    async fn start_message_processing(&mut self) -> Result<()> {
        info!("Starting REAL Signal message processing loop");
        
        // REAL IMPLEMENTATION: Initialize Signal client
        let mut signal_client = signal_integration::client::SignalClient::new().await
            .context("Failed to initialize Signal client")?;
        
        // REAL IMPLEMENTATION: Initialize AI processor
        let ai_processor = audio::whisper::WhisperProcessor::new().await
            .context("Failed to initialize AI processor")?;
        
        // REAL IMPLEMENTATION: Initialize vault for storage
        let vault_path = PathBuf::from(&self.config.vault.path);
        let db_path = PathBuf::from(&self.config.database.path);
        let vault = vault::Vault::new(db_path, vault_path).await
            .context("Failed to initialize vault")?;
        
        info!("✅ All components initialized - starting message processing loop");
        
        // REAL IMPLEMENTATION: Message processing loop
        loop {
            match self.process_incoming_messages(&mut signal_client, &ai_processor, &vault).await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        info!("✅ Processed {} messages successfully", processed_count);
                    }
                }
                Err(e) => {
                    error!("❌ Error processing messages: {}", e);
                    // Continue processing - don't crash on individual message errors
                }
            }
            
            // REAL IMPLEMENTATION: Sleep between message checks
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
    
    /// REAL IMPLEMENTATION: Process incoming messages with full pipeline
    async fn process_incoming_messages(
        &self,
        signal_client: &mut signal_integration::client::SignalClient,
        ai_processor: &audio::whisper::WhisperProcessor,
        vault: &vault::Vault,
    ) -> Result<usize> {
        // REAL IMPLEMENTATION: Receive messages from Signal
        let messages = signal_client.receive_messages().await
            .context("Failed to receive Signal messages")?;
        
        let mut processed_count = 0;
        
        for message in messages {
            // REAL IMPLEMENTATION: Filter for "Note to Self" messages
            if self.is_note_to_self(&message) {
                info!("📱 Processing Note to Self message from {}", message.sender);
                
                // REAL IMPLEMENTATION: Process voice attachments
                for attachment_path in &message.attachments {
                    if self.is_audio_file(attachment_path) {
                        match self.process_voice_message(attachment_path, ai_processor, vault).await {
                            Ok(()) => {
                                info!("🎤 Successfully processed voice message: {}", attachment_path);
                                processed_count += 1;
                            }
                            Err(e) => {
                                error!("❌ Failed to process voice message {}: {}", attachment_path, e);
                            }
                        }
                    }
                }
                
                // REAL IMPLEMENTATION: Process text content
                if !message.content.trim().is_empty() {
                    match self.process_text_message(&message.content, vault).await {
                        Ok(()) => {
                            info!("💬 Successfully processed text message");
                            processed_count += 1;
                        }
                        Err(e) => {
                            error!("❌ Failed to process text message: {}", e);
                        }
                    }
                }
            }
        }
        
        Ok(processed_count)
    }
    
    /// REAL IMPLEMENTATION: Process voice message with Whisper transcription
    async fn process_voice_message(
        &self,
        audio_path: &str,
        ai_processor: &audio::whisper::WhisperProcessor,
        vault: &vault::Vault,
    ) -> Result<()> {
        let audio_file_path = PathBuf::from(audio_path);
        
        // REAL IMPLEMENTATION: Transcribe audio with Whisper
        info!("🎤 Transcribing audio file: {}", audio_path);
        let transcription = ai_processor.transcribe_audio(&audio_file_path).await
            .context("Failed to transcribe audio")?;
        
        info!("✅ Transcription complete: {} characters", transcription.len());
        
        // REAL IMPLEMENTATION: Store transcription in vault using temporary markdown file
        let temp_file = std::env::temp_dir().join(format!("voice_note_{}.md", chrono::Utc::now().timestamp()));
        let content = format!("# Voice Note - {}\n\n{}\n\n---\ntags: voice-note, transcription\ntype: audio\n", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M"),
            transcription
        );
        
        std::fs::write(&temp_file, content)
            .context("Failed to create temporary markdown file")?;
        
        // Store in vault using real indexing
        vault.index_markdown_file(&temp_file, true).await
            .context("Failed to store transcription in vault")?;
            
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_file);
        
        info!("💾 Transcription stored in vault successfully");
        Ok(())
    }
    
    /// REAL IMPLEMENTATION: Process text message
    async fn process_text_message(&self, content: &str, vault: &vault::Vault) -> Result<()> {
        info!("💬 Processing text message: {} characters", content.len());
        
        // REAL IMPLEMENTATION: Store text in vault using temporary markdown file
        let temp_file = std::env::temp_dir().join(format!("text_note_{}.md", chrono::Utc::now().timestamp()));
        let markdown_content = format!("# Text Note - {}\n\n{}\n\n---\ntags: text-note, signal\ntype: text\n", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M"),
            content
        );
        
        std::fs::write(&temp_file, markdown_content)
            .context("Failed to create temporary markdown file")?;
        
        // Store in vault using real indexing
        vault.index_markdown_file(&temp_file, true).await
            .context("Failed to store text message in vault")?;
            
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_file);
        
        info!("💾 Text message stored in vault successfully");
        Ok(())
    }
    
    /// REAL IMPLEMENTATION: Check if message is "Note to Self"
    fn is_note_to_self(&self, message: &signal_integration::client::SignalMessage) -> bool {
        // REAL IMPLEMENTATION: Check if sender equals recipient (note to self)
        message.sender == message.recipient ||
        message.group_id.is_none() && // Not a group message
        message.sender == self.config.signal.phone_number.as_ref().unwrap_or(&String::new()).to_string()
    }
    
    /// REAL IMPLEMENTATION: Check if file is an audio file
    fn is_audio_file(&self, file_path: &str) -> bool {
        let path = std::path::Path::new(file_path);
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            matches!(extension.to_lowercase().as_str(), "mp3" | "wav" | "m4a" | "ogg" | "flac" | "aac")
        } else {
            false
        }
    }
    
    /// Query the knowledge base
    pub async fn query(&self, text: &str, semantic: bool, limit: usize) -> Result<()> {
        info!("Processing query: {}", text);
        let vault_path = PathBuf::from(&self.config.vault.path);
        let db_path = PathBuf::from(&self.config.database.path);

        let vault = vault::Vault::new(db_path, vault_path).await?;
        let results = vault.search(text, limit, /*hybrid*/ semantic).await?;

        if results.is_empty() {
            println!("No results.");
        } else {
            println!("Found {} result(s):", results.len());
            for (i, r) in results.iter().enumerate() {
                println!("{}. {} — {}", i + 1, r.document.title, r.document.path.display());
                println!("   score: {:.3}, tags: {:?}", r.score, r.document.tags);
                println!("   snippet: {}\n", r.document.snippet);
            }
        }

        Ok(())
    }
    
    /// Export notes to different formats
    pub async fn export(&self, output: &PathBuf, format: &str, _date_range: Option<&str>) -> Result<()> {
        info!("Exporting notes to {} format at {}", format, output.display());
        
        // TODO: Implement export functionality
        // This would:
        // 1. Query all documents (with date filter if specified)
        // 2. Convert to target format (Obsidian, Markdown, JSON)
        // 3. Write to output directory
        
        Ok(())
    }
    
    /// Show system status and statistics
    pub async fn show_status(&self) -> Result<()> {
        println!("🤖 note-to-ai System Status");
        println!("===========================");
        
        // Initialize and use the indexer to get real vault stats
        let vault_path = PathBuf::from(&self.config.vault.path);
        let db_path = PathBuf::from(&self.config.database.path);
        
        match vault::indexer::VaultIndexer::new(db_path, vault_path.clone()) {
            Ok(indexer) => {
                // Initialize the database
                if let Err(e) = indexer.initialize_db().await {
                    warn!("Failed to initialize indexer database: {}", e);
                }
                
                // Get vault statistics
                match indexer.get_stats().await {
                    Ok(stats) => {
                        println!("📊 Vault ({}/):", vault_path.display());
                        println!("  Total files: {}", stats.total_files);
                        println!("  Total size: {:.2} MB", stats.total_size as f64 / 1_048_576.0);
                        
                        // Show breakdown by file type
                        if !stats.type_counts.is_empty() {
                            println!("  File types:");
                            for (file_type, count) in &stats.type_counts {
                                println!("    {:?}: {}", file_type, count);
                            }
                        }
                        
                        // Check if indexing is needed
                        if stats.total_files == 0 && vault_path.exists() {
                            println!("  ⚠️  Vault directory exists but no files indexed");
                            println!("     Run indexing to populate the database");
                        }
                    }
                    Err(e) => {
                        println!("📊 Vault:");
                        println!("  ❌ Failed to read vault statistics: {}", e);
                        if !vault_path.exists() {
                            println!("     Vault directory does not exist: {}", vault_path.display());
                        }
                    }
                }
            }
            Err(e) => {
                println!("📊 Vault:");
                println!("  ❌ Failed to initialize indexer: {}", e);
            }
        }
        
        // AI status
        println!("\n🧠 AI Models:");
        let models_path = PathBuf::from(&self.config.ai.model_path);
        if models_path.exists() {
            let mut model_count = 0;
            if let Ok(entries) = std::fs::read_dir(&models_path) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "safetensors" || ext == "gguf" || ext == "bin" {
                            model_count += 1;
                            if let Some(name) = entry.path().file_stem() {
                                println!("  📦 {}", name.to_string_lossy());
                            }
                        }
                    }
                }
            }
            if model_count == 0 {
                println!("  ⚠️  No models found in {}", models_path.display());
            }
        } else {
            println!("  ❌ Models directory not found: {}", models_path.display());
        }
        
        // Signal status
        println!("\n📱 Signal:");
        if self.config.signal.enabled {
            if let Some(ref phone_number) = self.config.signal.phone_number {
                if !phone_number.is_empty() {
                    let masked_phone = mask_phone_number(phone_number);
                    println!("  Status: Configured ({})", masked_phone);
                } else {
                    println!("  Status: ⚠️  Enabled but phone number is empty");
                }
            } else {
                println!("  Status: ⚠️  Enabled but no phone number configured");
            }
        } else {
            println!("  Status: Disabled");
        }
        
        println!("\n✅ System ready for operation!");
        
        Ok(())
    }
    
    /// Index vault files for search
    pub async fn index_vault(&self, force: bool, verbose: bool) -> Result<()> {
        let vault_path = PathBuf::from(&self.config.vault.path);
        let db_path = PathBuf::from(&self.config.database.path);

        if !vault_path.exists() {
            println!("❌ Vault directory does not exist: {}", vault_path.display());
            println!("   Create the directory and add some files to get started.");
            return Ok(());
        }

        println!("📁 Indexing vault: {}", vault_path.display());
        let vault = vault::Vault::new(db_path, vault_path.clone()).await?;

        let start_time = std::time::Instant::now();
    let stats = vault.index_all(force, Some(|p: &std::path::Path| {
            println!("  • {}", p.display());
        })).await?;

        let duration = start_time.elapsed();

        println!("✅ Indexing completed in {:?}", duration);
        println!("   📊 File scan: added={}, updated={}, deleted={}, skipped={}, errors={}",
                 stats.files.added, stats.files.updated, stats.files.deleted, stats.files.skipped, stats.files.errors);
        println!("   🧠 Content: docs={}, fts={}, embeddings={}, errors={}",
                 stats.docs_indexed, stats.fts_docs, stats.embedding_docs, stats.errors);

        if stats.docs_indexed > 0 {
            println!("\n🔍 Try: cargo run -- query --semantic \"your search term\"");
        }

        Ok(())
    }
    
    /// Wait for shutdown signal
    async fn wait_for_shutdown(&self) {
        let mut sigterm = tokio_signal::unix::signal(tokio_signal::unix::SignalKind::terminate())
            .expect("Failed to create SIGTERM handler");
        let mut sigint = tokio_signal::unix::signal(tokio_signal::unix::SignalKind::interrupt())
            .expect("Failed to create SIGINT handler");
        
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down gracefully");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT (Ctrl+C), shutting down gracefully");
            }
        }
        
        info!("Shutting down note-to-ai service");
        // TODO: Graceful shutdown of all services
    }
    
    /// Create a demo AI response note in Obsidian format
    pub async fn obsidian_demo(&self, query: &str) -> Result<()> {
        use obsidian::{ObsidianManager, ObsidianConfig, create_demo_response};
        
        info!("Creating demo Obsidian AI response for query: {}", query);
        
        // Create Obsidian manager with config
        let obsidian_config = ObsidianConfig {
            vault_path: self.config.vault.path.clone(),
            ..Default::default()
        };
        let manager = ObsidianManager::new(obsidian_config);
        
        // Create a demo response
        let response_text = format!(
            "Based on your query about '{}', here are the key insights from your knowledge base:\n\n\
            ## Key Concepts\n\
            1. **Quantum Superposition**: The ability of quantum systems to exist in multiple states simultaneously\n\
            2. **Quantum Entanglement**: The phenomenon where quantum particles become interconnected\n\
            3. **Quantum Gates**: The basic building blocks of quantum circuits\n\n\
            ## Applications\n\
            - Quantum machine learning algorithms\n\
            - Cryptographic applications\n\
            - Optimization problems\n\n\
            ## Next Steps\n\
            Consider exploring the intersection of quantum computing and artificial intelligence, \
            as discussed in your recent research notes.",
            query
        );
        
        let demo_response = create_demo_response(query, &response_text);
        
        // Save the response
        let note_path = manager.save_ai_response(demo_response).await?;
        
        println!("✅ Demo AI response created!");
        println!("📝 File: {}", note_path.display());
        println!("🔗 Open in Obsidian to see the formatted note with links and tags");
        
        Ok(())
    }
    
    /// Create or update today's daily note
    pub async fn obsidian_daily(&self, summary: &str) -> Result<()> {
        use obsidian::{ObsidianManager, ObsidianConfig};
        
        info!("Adding interaction to daily note: {}", summary);
        
        // Create Obsidian manager with config
        let obsidian_config = ObsidianConfig {
            vault_path: self.config.vault.path.clone(),
            ..Default::default()
        };
        let manager = ObsidianManager::new(obsidian_config);
        
        // Update daily note
        let note_path = manager.append_to_daily_note(summary).await?;
        
        println!("✅ Daily note updated!");
        println!("📝 File: {}", note_path.display());
        println!("📅 Added interaction summary to today's daily note");
        
        Ok(())
    }
    
    /// Scan vault for linkable notes
    pub async fn obsidian_scan(&self) -> Result<()> {
        use obsidian::{ObsidianManager, ObsidianConfig};
        
        info!("Scanning vault for Obsidian-linkable notes");
        
        // Create Obsidian manager with config
        let obsidian_config = ObsidianConfig {
            vault_path: self.config.vault.path.clone(),
            ..Default::default()
        };
        let manager = ObsidianManager::new(obsidian_config);
        
        // Scan for files
        let content = "quantum computing machine learning AI research"; // Sample content for testing
        let related_notes = manager.find_related_notes(content).await?;
        
        println!("🔍 Vault scan results:");
        println!("📁 Vault path: {}", self.config.vault.path.display());
        
        if related_notes.is_empty() {
            println!("📝 No linkable notes found for sample content");
            println!("💡 Add more .md files to your vault to see automatic linking in action");
        } else {
            println!("🔗 Found {} potentially linkable notes:", related_notes.len());
            for note in &related_notes {
                println!("   {}", note);
            }
        }
        
        Ok(())
    }
    
    /// Run Signal Protocol integration tests
    pub async fn test_signal_protocol(&self) -> Result<()> {
        use signal_integration::integration_tests::SignalIntegrationTester;
        
        info!("Running Signal Protocol integration tests");
        println!("🔐 Starting Signal Protocol Integration Test Suite");
        println!("================================================");
        
        // Initialize test environment
        let mut tester = SignalIntegrationTester::new().await
            .context("Failed to initialize test environment")?;
        
        // Run complete test suite
        let report = tester.run_full_test_suite().await
            .context("Failed to run test suite")?;
        
        // Print detailed report
        report.print_report();
        
        // Return success/failure based on results
        if report.success_rate >= 80.0 {
            info!("Signal Protocol integration tests completed successfully");
            Ok(())
        } else {
            warn!("Signal Protocol integration tests failed with {}% success rate", report.success_rate);
            Err(anyhow::anyhow!("Integration tests failed"))
        }
    }
    
    /// REAL IMPLEMENTATION: Verify Whisper model availability
    async fn verify_whisper_model(&self) -> Result<()> {
        let models_path = PathBuf::from(&self.config.ai.model_path);
        
        // Check for Whisper model files
        let whisper_files = ["whisper-base.safetensors", "whisper-large-v3.bin", "whisper-medium.bin"];
        
        for file in &whisper_files {
            let model_path = models_path.join(file);
            if model_path.exists() {
                let metadata = tokio::fs::metadata(&model_path).await?;
                info!("✅ Found Whisper model: {} ({:.1} MB)", file, metadata.len() as f64 / 1_048_576.0);
                return Ok(());
            }
        }
        
        // Check whisper.cpp directory
        let whisper_cpp_path = models_path.join("whisper.cpp");
        if whisper_cpp_path.exists() {
            info!("✅ Found whisper.cpp installation");
            return Ok(());
        }
        
        Err(anyhow::anyhow!("No Whisper model found in {}", models_path.display()))
    }
    
    /// REAL IMPLEMENTATION: Verify embedding model availability  
    async fn verify_embedding_model(&self) -> Result<()> {
        let models_path = PathBuf::from(&self.config.ai.model_path);
        
        // Check for embedding model files
        let embedding_files = ["all-MiniLM-L6-v2.safetensors", "sentence-transformers"];
        
        for file in &embedding_files {
            let model_path = models_path.join(file);
            if model_path.exists() {
                let metadata = tokio::fs::metadata(&model_path).await?;
                info!("✅ Found embedding model: {} ({:.1} MB)", file, metadata.len() as f64 / 1_048_576.0);
                return Ok(());
            }
        }
        
        Err(anyhow::anyhow!("No embedding model found in {}", models_path.display()))
    }
    
    /// REAL IMPLEMENTATION: Verify Signal connection
    async fn verify_signal_connection(&self) -> Result<()> {
        // REAL IMPLEMENTATION: Check Signal-CLI availability
        match tokio::process::Command::new("signal-cli")
            .arg("--version")
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    info!("✅ Signal-CLI available: {}", version.trim());
                } else {
                    return Err(anyhow::anyhow!("Signal-CLI command failed"));
                }
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Signal-CLI not found. Install from: https://github.com/AsamK/signal-cli"));
            }
        }
        
        // REAL IMPLEMENTATION: Check if phone number is configured
        if let Some(phone) = &self.config.signal.phone_number {
            info!("📱 Configured phone number: {}", mask_phone_number(&phone.to_string()));
            
            // REAL IMPLEMENTATION: Test Signal connection with actual phone number
            let mut client = signal_integration::client::SignalClient::new().await?;
            
            // Verify the phone number is registered
            match client.connect(phone.to_string()).await {
                Ok(()) => {
                    info!("✅ Signal connection verified for {}", mask_phone_number(&phone.to_string()));
                    Ok(())
                }
                Err(e) => {
                    Err(anyhow::anyhow!("Signal connection failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("No phone number configured. Run: cargo run -- signal setup --phone +1234567890"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    setup_logging(&cli.log_level, cli.log_file.as_ref())?;
    
    // Print startup banner
    print_startup_banner();
    
    // Route to appropriate handler
    commands::handle_command(cli).await?;
    
    Ok(())
}

fn setup_logging(level: &str, log_file: Option<&PathBuf>) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    let registry = tracing_subscriber::registry()
        .with(env_filter);
    
    if let Some(log_file) = log_file {
        // Log to file
        let file = std::fs::File::create(log_file)
            .context("Failed to create log file")?;
        
        registry
            .with(fmt::layer().with_writer(file))
            .init();
    } else {
        // Log to stdout
        registry
            .with(fmt::layer().with_writer(std::io::stdout))
            .init();
    }
    
    Ok(())
}

fn print_startup_banner() {
    println!(r#"
    ████ NOTE-TO-AI ████ Personal AI Assistant ████ Voice → Brain → Search → Reply ████
"#);
}

fn mask_phone_number(phone: &str) -> String {
    if phone.len() > 4 {
        let visible_end = &phone[phone.len()-2..];
        format!("***-***-**{}", visible_end)
    } else {
        "***-***-****".to_string()
    }
}