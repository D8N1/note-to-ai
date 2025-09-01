// MVP Core: Intelligence Compression Engine
pub mod compression;

// Essential modules for MVP
pub mod ai;
pub mod audio;
pub mod config;
pub mod logger;
pub mod obsidian;
pub mod vault;

// Post-MVP modules (deferred for cross-industry focus)
// pub mod crypto;
// pub mod attestation;
// pub mod identity;
// pub mod scheduler;
// pub mod signal_integration;
// pub mod swarm;

pub use config::Settings;

// MVP exports - Intelligence Compression Engine
pub use compression::{
    CompressionEngine, CompressionContext, CognitiveOutput, 
    InformationPacket, DecisionPoint, IntelligenceSummary
};
pub use compression::legal::{LegalCompressionEngine, LegalIntelligence};
pub use compression::medical::{MedicalCompressionEngine, MedicalIntelligence};

// Core AI exports for MVP
pub use vault::embeddings::EmbeddingProvider;
pub use vault::storage::HybridStorageEngine;

// Obsidian exports
pub use obsidian::{AIResponse, ObsidianManager, ObsidianConfig};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>; 