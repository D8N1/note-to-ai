// MVP Core: Intelligence Compression Engine
pub mod compression;

// Essential modules for MVP
#[cfg(feature = "ai-models")]
pub mod ai;
pub mod audio;
pub mod config;
pub mod logger;
pub mod obsidian;
#[cfg(feature = "analytics")]
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

// Core AI exports for MVP (conditional based on features)
#[cfg(feature = "ai-models")]
pub use vault::embeddings::EmbeddingProvider;
#[cfg(feature = "analytics")]
pub use vault::storage::HybridStorageEngine;

// Obsidian exports
pub use obsidian::{AIResponse, ObsidianManager, ObsidianConfig};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>; 