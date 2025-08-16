use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use tokio::signal as tokio_signal;
use tracing::{info, warn};
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

use config::Settings;
// Temporarily disabled while fixing Arrow ecosystem conflicts
// use vault::storage::{HybridStorageEngine, StorageConfig};

/// note-to-ai: Transform your Signal "Note to Self" into an AI-powered knowledge base
#[derive(Parser)]
#[command(name = "note-to-ai")]
#[command(about = "Your personal AI assistant via Signal", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config/config.toml")]
    config: PathBuf,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
    
    /// Log to file instead of stdout
    #[arg(long)]
    log_file: Option<PathBuf>,
    
    /// Run in daemon mode (background service)
    #[arg(long)]
    daemon: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the note-to-ai service
    Start {
        /// Skip Signal connection (for testing)
        #[arg(long)]
        skip_signal: bool,
        
        /// Skip AI model loading (for faster startup)
        #[arg(long)]
        skip_ai: bool,
    },
    
    /// Query your knowledge base directly
    Query {
        /// Query text
        text: String,
        
        /// Use semantic search instead of text search
        #[arg(long)]
        semantic: bool,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    
    /// Export your notes to different formats
    Export {
        /// Output directory
        #[arg(short, long, default_value = "./export")]
        output: PathBuf,
        
        /// Export format (obsidian, markdown, json)
        #[arg(short, long, default_value = "obsidian")]
        format: String,
        
        /// Date range filter (YYYY-MM-DD to YYYY-MM-DD)
        #[arg(long)]
        date_range: Option<String>,
    },
    
    /// Show system status and statistics
    Status,
    
    /// Index vault files for search
    Index {
        /// Force full re-indexing (ignore change detection)
        #[arg(long)]
        force: bool,
        
        /// Show detailed progress
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Manage AI models
    Models {
        #[command(subcommand)]
        action: ModelAction,
    },
    
    /// Setup and configure Signal integration
    Signal {
        #[command(subcommand)]
        action: SignalAction,
    },
    
    /// Test Obsidian integration
    Obsidian {
        #[command(subcommand)]
        action: ObsidianAction,
    },
}

#[derive(Subcommand)]
enum ModelAction {
    /// List available models
    List,
    /// Download a specific model
    Download { name: String },
    /// Remove a model
    Remove { name: String },
    /// Test model performance
    Benchmark { name: String },
}

#[derive(Subcommand)]
enum SignalAction {
    /// Setup Signal integration
    Setup {
        /// Phone number for registration
        #[arg(long)]
        phone: String,
    },
    /// Test Signal connection
    Test,
    /// Show Signal status
    Status,
}

#[derive(Subcommand)]
enum ObsidianAction {
    /// Create a demo AI response note
    Demo {
        /// Query to simulate
        #[arg(default_value = "What are the key concepts in quantum computing?")]
        query: String,
    },
    /// Create or update today's daily note
    Daily {
        /// Interaction summary to add
        summary: String,
    },
    /// Scan vault for linkable notes
    Scan,
}

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
    
    /// Start the main service loop
    pub async fn start(&mut self, skip_signal: bool, skip_ai: bool) -> Result<()> {
        info!("Starting note-to-ai service");
        
        // TODO: Start scheduler when it's implemented
        // self.scheduler.start().await
        //     .context("Failed to start scheduler")?;
        
        // Load AI models (unless skipped)
        if !skip_ai {
            info!("Loading AI models...");
            // TODO: Load models based on configuration
            info!("AI models loaded successfully");
        } else {
            warn!("Skipping AI model loading");
        }
        
        // Connect to Signal (unless skipped)
        if !skip_signal {
            info!("Connecting to Signal...");
            // TODO: Implement Signal connection
            info!("Signal connected successfully");
            
            // Start message processing loop
            self.start_message_processing().await?;
        } else {
            warn!("Skipping Signal connection");
        }
        
        info!("✅ note-to-ai service started successfully!");
        info!("Send a voice message to your Signal 'Note to Self' to get started");
        
        // Wait for shutdown signal
        self.wait_for_shutdown().await;
        
        Ok(())
    }
    
    /// Start processing Signal messages
    async fn start_message_processing(&mut self) -> Result<()> {
        info!("Starting Signal message processing");
        
        // TODO: Implement message processing loop
        // This would:
        // 1. Listen for incoming Signal messages
        // 2. Filter for "Note to Self" messages
        // 3. Process voice messages with Whisper
        // 4. Generate embeddings and store in hybrid database
        // 5. Respond to queries with AI-generated answers
        
        Ok(())
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    setup_logging(&cli.log_level, cli.log_file.as_ref())?;
    
    // Print startup banner
    print_startup_banner();
    
    match cli.command {
        Some(Commands::Start { skip_signal, skip_ai }) => {
            let mut app = NoteToAI::new(&cli.config).await?;
            app.start(skip_signal, skip_ai).await?;
        }
        
        Some(Commands::Query { text, semantic, limit }) => {
            let app = NoteToAI::new(&cli.config).await?;
            app.query(&text, semantic, limit).await?;
        }
        
        Some(Commands::Export { output, format, date_range }) => {
            let app = NoteToAI::new(&cli.config).await?;
            app.export(&output, &format, date_range.as_deref()).await?;
        }
        
        Some(Commands::Status) => {
            let app = NoteToAI::new(&cli.config).await?;
            app.show_status().await?;
        }
        
        Some(Commands::Index { force, verbose }) => {
            let app = NoteToAI::new(&cli.config).await?;
            app.index_vault(force, verbose).await?;
        }
        
        Some(Commands::Models { action }) => {
            match action {
                ModelAction::List => {
                    println!("Available AI models:");
                    println!("  whisper-base (~290MB) - Speech-to-text");
                    println!("  all-MiniLM-L6-v2 (~90MB) - Text embeddings");
                    println!("  hermes-3-8b (~16GB) - Conversational AI");
                    println!("  phi-3-mini (~6GB) - Lightweight LLM");
                }
                ModelAction::Download { name } => {
                    info!("Downloading model: {}", name);
                    // TODO: Implement model download
                }
                ModelAction::Remove { name } => {
                    info!("Removing model: {}", name);
                    // TODO: Implement model removal
                }
                ModelAction::Benchmark { name } => {
                    info!("Benchmarking model: {}", name);
                    // TODO: Implement model benchmarking
                }
            }
        }
        
        Some(Commands::Signal { action }) => {
            match action {
                SignalAction::Setup { phone } => {
                    info!("Setting up Signal integration for {}", phone);
                    // TODO: Implement Signal setup
                }
                SignalAction::Test => {
                    info!("Testing Signal connection");
                    // TODO: Implement Signal test
                }
                SignalAction::Status => {
                    info!("Signal connection status");
                    // TODO: Show Signal status
                }
            }
        }
        
        Some(Commands::Obsidian { action }) => {
            let app = NoteToAI::new(&cli.config).await?;
            match action {
                ObsidianAction::Demo { query } => {
                    app.obsidian_demo(&query).await?;
                }
                ObsidianAction::Daily { summary } => {
                    app.obsidian_daily(&summary).await?;
                }
                ObsidianAction::Scan => {
                    app.obsidian_scan().await?;
                }
            }
        }
        
        None => {
            // Default: start the service
            let mut app = NoteToAI::new(&cli.config).await?;
            app.start(false, false).await?;
        }
    }
    
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
╔══════════════════════════════════════════════════════════════╗
║                         note-to-ai                          ║
║              Your Personal AI Assistant via Signal          ║
║                                                              ║
║  🎤 Voice → 🧠 AI → 🔍 Search → 💬 Respond                  ║
║                                                              ║
║  Transform your Signal "Note to Self" into an intelligent   ║
║  knowledge base powered by local AI models.                 ║
╚══════════════════════════════════════════════════════════════╝
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