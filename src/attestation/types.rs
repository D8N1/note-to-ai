use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttestationStatus {
    Pending,
    Verified,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub context_hash: String,
    pub proof_data_b64: String,
    pub verification_key_b64: String,
    pub metadata_encrypted_b64: Option<String>,
    pub related_path: Option<PathBuf>,
    pub status: AttestationStatus,
}
