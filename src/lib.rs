pub mod ai;
pub mod audio;
pub mod config;
pub mod crypto;
pub mod attestation;
pub mod identity;
pub mod logger;
pub mod obsidian;
pub mod scheduler;
pub mod signal_integration;  // Updated to match renamed module
pub mod swarm;
pub mod vault;

pub use config::Settings;

// Re-export key types for easier access
pub use identity::zkpassport::ZKPassport;

// New hybrid backend exports for Barretenberg UltraHonk integration
pub use identity::proving_backend::{
    AdaptiveProver, DeviceCapabilities, ProofStrategy, ProvingBackend, ZkProver
};
pub use identity::zkpassport_migration::{
    MigratedZkPassport, PassportContext, PassportData, ZkPassportConfig
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>; 