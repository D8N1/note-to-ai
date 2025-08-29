// File: src/commands/mod.rs
// Command-line interface commands for note-to-ai

use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod signal_link;
pub use signal_link::SignalLinkCommand;

/// note-to-ai: Transform your Signal "Note to Self" into an AI-powered knowledge base
#[derive(Parser)]
#[command(name = "note-to-ai")]
#[command(about = "Your personal AI assistant via Signal", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config/config.toml")]
    pub config: PathBuf,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,
    
    /// Log to file instead of stdout
    #[arg(long)]
    pub log_file: Option<PathBuf>,
    
    /// Run in daemon mode (background service)
    #[arg(long)]
    pub daemon: bool,
}

#[derive(Subcommand)]
pub enum Commands {
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
    
    /// Test Signal Protocol implementation
    TestSignal,
    
    /// Test Obsidian integration
    Obsidian {
        #[command(subcommand)]
        action: ObsidianAction,
    },

    /// Record an attestation event (prototype)
    Attest {
        /// Path to related file/content (optional)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Freeform context string to hash
        #[arg(short, long)]
        context: Option<String>,
    },
    
    /// Link Signal device with QR code
    #[command(name = "signal-link")]
    SignalLink {
        #[command(subcommand)]
        action: Option<SignalLinkAction>,
    },
}

#[derive(Subcommand)]
pub enum SignalLinkAction {
    /// Quick device linking with QR code
    Quick,
    /// Step-by-step device linking
    Setup,
    /// Test QR code display
    Test,
    /// Check linking status
    Status,
}

#[derive(Subcommand)]
pub enum ModelAction {
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
pub enum SignalAction {
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
pub enum ObsidianAction {
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

/// Main command handler
pub async fn handle_command(cli: Cli) -> Result<()> {
    use crate::NoteToAI;
    use tracing::info;
    
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
                    // TODO: Implement Signal setup with phone number registration
                }
                SignalAction::Test => {
                    info!("Testing Signal connection");
                    // TODO: Implement Signal test
                }
                SignalAction::Status => {
                    info!("Checking Signal connection status");
                    // TODO: Show Signal status
                }
            }
        }
        
        Some(Commands::TestSignal) => {
            let app = NoteToAI::new(&cli.config).await?;
            app.test_signal_protocol().await?;
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

        Some(Commands::Attest { path, context }) => {
            let app = NoteToAI::new(&cli.config).await?;
            app.attest(path, context).await?;
        }
        
        Some(Commands::SignalLink { action }) => {
            let signal_link_cmd = SignalLinkCommand::new();
            
            match action {
                Some(SignalLinkAction::Quick) | None => {
                    // Default to quick linking
                    signal_link_cmd.quick_device_link().await?;
                }
                Some(SignalLinkAction::Setup) => {
                    signal_link_cmd.step_by_step_linking().await?;
                }
                Some(SignalLinkAction::Test) => {
                    signal_link_cmd.test_qr_display().await?;
                }
                Some(SignalLinkAction::Status) => {
                    signal_link_cmd.check_linking_status().await?;
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

fn mask_phone_number(phone: &str) -> String {
    if phone.len() > 4 {
        let visible_end = &phone[phone.len()-2..];
        format!("***-***-**{}", visible_end)
    } else {
        "***-***-****".to_string()
    }
}
