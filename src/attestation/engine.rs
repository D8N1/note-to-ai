use crate::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use super::types::{AttestationEvent, AttestationStatus};

pub struct AttestationEngine {
    db_path: PathBuf,
}

impl AttestationEngine {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        Ok(Self { db_path })
    }

    fn conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        Ok(conn)
    }

    pub fn initialize(&self) -> Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS attestation_events (
                id TEXT PRIMARY KEY,
                ts TEXT NOT NULL,
                device_id TEXT NOT NULL,
                context_hash TEXT NOT NULL,
                proof_data_b64 TEXT NOT NULL,
                verification_key_b64 TEXT NOT NULL,
                metadata_encrypted_b64 TEXT,
                related_path TEXT,
                status TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_att_ts ON attestation_events(ts);
            CREATE INDEX IF NOT EXISTS idx_att_ctx ON attestation_events(context_hash);
            "#,
        )?;
        Ok(())
    }

    pub fn record_event(&self, evt: &AttestationEvent) -> Result<()> {
    let mut conn = self.conn()?;
    let tx = conn.transaction()?;
        tx.execute(
            r#"INSERT OR REPLACE INTO attestation_events (
                id, ts, device_id, context_hash, proof_data_b64, verification_key_b64,
                metadata_encrypted_b64, related_path, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                evt.id,
                evt.timestamp.to_rfc3339(),
                evt.device_id,
                evt.context_hash,
                evt.proof_data_b64,
                evt.verification_key_b64,
                evt.metadata_encrypted_b64,
                evt.related_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                match &evt.status {
                    AttestationStatus::Pending => "Pending".to_string(),
                    AttestationStatus::Verified => "Verified".to_string(),
                    AttestationStatus::Failed(e) => format!("Failed:{e}"),
                }
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<AttestationEvent>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, ts, device_id, context_hash, proof_data_b64, verification_key_b64, metadata_encrypted_b64, related_path, status
             FROM attestation_events ORDER BY ts DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            let ts_str: String = row.get(1)?;
            let status_str: String = row.get(8)?;
            let status = if status_str == "Pending" {
                AttestationStatus::Pending
            } else if status_str == "Verified" {
                AttestationStatus::Verified
            } else if let Some(rest) = status_str.strip_prefix("Failed:") {
                AttestationStatus::Failed(rest.to_string())
            } else {
                AttestationStatus::Failed("Unknown".to_string())
            };
            Ok(AttestationEvent {
                id: row.get(0)?,
                timestamp: ts_str.parse().unwrap_or(DateTime::<Utc>::from(std::time::SystemTime::now())),
                device_id: row.get(2)?,
                context_hash: row.get(3)?,
                proof_data_b64: row.get(4)?,
                verification_key_b64: row.get(5)?,
                metadata_encrypted_b64: row.get(6)?,
                related_path: row.get::<_, Option<String>>(7)?.map(PathBuf::from),
                status,
            })
        })?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }
}
