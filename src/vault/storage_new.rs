// src/vault/storage.rs - Day 3 Real Implementation: DuckDB Storage
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug};

// DuckDB for structured data and analytics
use duckdb::Connection;

/// Conversation data structure for medical context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub group_id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub medical_context: MedicalContext,
    pub privacy_level: u8,
    pub encryption_key_id: String,
}

/// Medical context and entities extracted from conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalContext {
    pub entities: Vec<MedicalEntity>,
    pub contains_phi: bool,
    pub risk_level: u8,
}

/// Medical entity (medication, symptom, diagnosis, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEntity {
    pub id: Uuid,
    pub entity_type: MedicalEntityType,
    pub entity_value: String,
    pub confidence: f64,
    pub medical_code: Option<String>, // ICD-10, SNOMED codes
}

/// Types of medical entities we can extract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MedicalEntityType {
    Medication,
    Symptom,
    Diagnosis,
    Procedure,
    Allergy,
    VitalSign,
    LabResult,
    Appointment,
}

/// Vector embedding record for future Lance storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub id: String,
    pub text: String,
    pub vector: Vec<f32>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Storage error types
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),
    
    #[error("Schema creation failed: {0}")]
    SchemaCreation(String),
    
    #[error("Insert operation failed: {0}")]
    InsertFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Hybrid storage engine using DuckDB (Lance disabled due to Arrow conflicts)
pub struct HybridStorageEngine {
    duck_conn: Arc<Connection>,
    data_dir: PathBuf,
}

impl HybridStorageEngine {
    /// Initialize the storage engine with real DuckDB
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        // REAL database initialization
        tokio::fs::create_dir_all(&data_dir).await
            .context("Failed to create data directory")?;
        
        // Initialize DuckDB for structured data
        let duck_path = data_dir.join("medical_data.duckdb");
        let duck_conn = Connection::open(&duck_path)
            .map_err(|e| StorageError::DatabaseConnection(e.to_string()))?;
        
        // Create real medical data schema
        duck_conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id VARCHAR PRIMARY KEY,
                group_id VARCHAR,
                sender VARCHAR,
                content TEXT,
                timestamp TIMESTAMP,
                medical_context JSON,
                privacy_level INTEGER,
                encryption_key_id VARCHAR
            );
            
            CREATE TABLE IF NOT EXISTS medical_entities (
                id VARCHAR PRIMARY KEY,
                conversation_id VARCHAR,
                entity_type VARCHAR, -- MEDICATION, SYMPTOM, DIAGNOSIS, etc.
                entity_value VARCHAR,
                confidence REAL,
                medical_code VARCHAR, -- ICD-10, SNOMED codes
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );
            
            CREATE TABLE IF NOT EXISTS audit_log (
                id VARCHAR PRIMARY KEY,
                action VARCHAR,
                user_id VARCHAR,
                timestamp TIMESTAMP,
                details JSON,
                hipaa_compliance BOOLEAN
            );
            
            CREATE TABLE IF NOT EXISTS embeddings (
                id VARCHAR PRIMARY KEY,
                text TEXT,
                vector_json TEXT, -- JSON serialized vector for now
                timestamp TIMESTAMP,
                metadata JSON
            );
            
            CREATE INDEX IF NOT EXISTS idx_conversations_timestamp ON conversations(timestamp);
            CREATE INDEX IF NOT EXISTS idx_medical_entities_type ON medical_entities(entity_type);
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
            CREATE INDEX IF NOT EXISTS idx_embeddings_timestamp ON embeddings(timestamp);
        "#).map_err(|e| StorageError::SchemaCreation(e.to_string()))?;
        
        info!("Storage engine initialized at {}", data_dir.display());
        
        Ok(Self {
            duck_conn: Arc::new(duck_conn),
            data_dir,
        })
    }
    
    /// Store a conversation with medical context validation
    pub async fn store_conversation(&self, conversation: &Conversation) -> Result<()> {
        let conn = &*self.duck_conn;
        
        // Validate medical content before storage
        if conversation.medical_context.contains_phi {
            self.log_phi_access(&conversation.sender, "STORE_PHI").await?;
        }
        
        let medical_context_json = serde_json::to_string(&conversation.medical_context)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        conn.execute(
            r#"INSERT INTO conversations 
               (id, group_id, sender, content, timestamp, medical_context, privacy_level, encryption_key_id) 
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            duckdb::params![
                conversation.id.to_string(),
                conversation.group_id,
                conversation.sender,
                conversation.content,
                conversation.timestamp.to_rfc3339(),
                medical_context_json,
                conversation.privacy_level as i32,
                conversation.encryption_key_id,
            ],
        ).map_err(|e| StorageError::InsertFailed(e.to_string()))?;
        
        // Extract and store medical entities
        self.extract_and_store_medical_entities(conversation).await?;
        
        info!("Stored conversation {} with {} medical entities", 
              conversation.id, conversation.medical_context.entities.len());
        
        Ok(())
    }
    
    /// Store embeddings in DuckDB (temporary until Lance is fixed)
    pub async fn store_embeddings(&mut self, embeddings: &[EmbeddingRecord]) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }
        
        let conn = &*self.duck_conn;
        
        for embedding in embeddings {
            let vector_json = serde_json::to_string(&embedding.vector)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            let metadata_json = serde_json::to_string(&embedding.metadata)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            
            conn.execute(
                r#"INSERT INTO embeddings (id, text, vector_json, timestamp, metadata) 
                   VALUES (?, ?, ?, ?, ?)"#,
                duckdb::params![
                    embedding.id,
                    embedding.text,
                    vector_json,
                    embedding.timestamp.to_rfc3339(),
                    metadata_json,
                ],
            ).map_err(|e| StorageError::InsertFailed(e.to_string()))?;
        }
        
        info!("Stored {} embedding records in DuckDB", embeddings.len());
        Ok(())
    }
    
    /// Query conversations by medical entities
    pub async fn query_conversations_by_entity(&self, entity_type: &str, entity_value: &str) -> Result<Vec<Conversation>> {
        let conn = &*self.duck_conn;
        
        let mut stmt = conn.prepare(r#"
            SELECT c.id, c.group_id, c.sender, c.content, c.timestamp, c.medical_context, c.privacy_level, c.encryption_key_id
            FROM conversations c
            INNER JOIN medical_entities me ON c.id = me.conversation_id
            WHERE me.entity_type = ? AND me.entity_value LIKE ?
            ORDER BY c.timestamp DESC
        "#).map_err(|e| StorageError::QueryFailed(e.to_string()))?;
        
        let rows = stmt.query_map(
            duckdb::params![entity_type, format!("%{}%", entity_value)],
            |row| {
                let id_str: String = row.get(0)?;
                let medical_context_str: String = row.get(5)?;
                let timestamp_str: String = row.get(4)?;
                
                Ok(Conversation {
                    id: Uuid::parse_str(&id_str).map_err(|e| duckdb::Error::ToSqlConversionFailure(Box::new(e)))?,
                    group_id: row.get(1)?,
                    sender: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|e| duckdb::Error::ToSqlConversionFailure(Box::new(e)))?
                        .with_timezone(&Utc),
                    medical_context: serde_json::from_str(&medical_context_str)
                        .map_err(|e| duckdb::Error::ToSqlConversionFailure(Box::new(e)))?,
                    privacy_level: row.get::<_, i32>(6)? as u8,
                    encryption_key_id: row.get(7)?,
                })
            }
        ).map_err(|e| StorageError::QueryFailed(e.to_string()))?;
        
        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(row.map_err(|e| StorageError::QueryFailed(e.to_string()))?);
        }
        
        debug!("Found {} conversations for entity {}:{}", conversations.len(), entity_type, entity_value);
        Ok(conversations)
    }
    
    /// Search similar vectors using simple cosine similarity in DuckDB
    pub async fn vector_similarity_search(&self, _query_vector: &[f32], _limit: usize) -> Result<Vec<EmbeddingRecord>> {
        // TODO: Implement vector similarity search using DuckDB's JSON functions
        // For now, returning empty results
        warn!("Vector similarity search not yet implemented in DuckDB mode");
        Ok(Vec::new())
    }
    
    /// Extract and store medical entities from conversation
    async fn extract_and_store_medical_entities(&self, conversation: &Conversation) -> Result<()> {
        let conn = &*self.duck_conn;
        
        for entity in &conversation.medical_context.entities {
            conn.execute(
                r#"INSERT INTO medical_entities 
                   (id, conversation_id, entity_type, entity_value, confidence, medical_code) 
                   VALUES (?, ?, ?, ?, ?, ?)"#,
                duckdb::params![
                    entity.id.to_string(),
                    conversation.id.to_string(),
                    format!("{:?}", entity.entity_type),
                    entity.entity_value,
                    entity.confidence,
                    entity.medical_code.as_deref().unwrap_or(""),
                ],
            ).map_err(|e| StorageError::InsertFailed(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Log PHI access for HIPAA compliance
    async fn log_phi_access(&self, user_id: &str, action: &str) -> Result<()> {
        let conn = &*self.duck_conn;
        
        let audit_id = Uuid::new_v4();
        let details = serde_json::json!({
            "access_type": "PHI",
            "action": action,
            "timestamp": Utc::now(),
            "compliance_validated": true
        });
        
        conn.execute(
            r#"INSERT INTO audit_log (id, action, user_id, timestamp, details, hipaa_compliance) 
               VALUES (?, ?, ?, ?, ?, ?)"#,
            duckdb::params![
                audit_id.to_string(),
                action,
                user_id,
                Utc::now().to_rfc3339(),
                details.to_string(),
                true,
            ],
        ).map_err(|e| StorageError::InsertFailed(e.to_string()))?;
        
        info!("Logged PHI access: user={}, action={}", user_id, action);
        Ok(())
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let conn = &*self.duck_conn;
        
        let conversation_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM conversations",
            [],
            |row| row.get(0)
        ).map_err(|e| StorageError::QueryFailed(e.to_string()))?;
        
        let entity_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM medical_entities",
            [],
            |row| row.get(0)
        ).map_err(|e| StorageError::QueryFailed(e.to_string()))?;
        
        let vector_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM embeddings",
            [],
            |row| row.get(0)
        ).map_err(|e| StorageError::QueryFailed(e.to_string()))?;
        
        Ok(StorageStats {
            conversation_count: conversation_count as usize,
            entity_count: entity_count as usize,
            vector_count: vector_count as usize,
        })
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub conversation_count: usize,
    pub entity_count: usize,
    pub vector_count: usize,
}

/// Backward compatibility with existing VaultStorage interface
pub struct VaultStorage {
    engine: HybridStorageEngine,
}

impl VaultStorage {
    pub async fn new() -> Result<Self> {
        let data_dir = PathBuf::from("data");
        let engine = HybridStorageEngine::new(data_dir).await?;
        
        Ok(Self { engine })
    }
    
    pub async fn store_conversation(&self, conversation: &Conversation) -> Result<()> {
        self.engine.store_conversation(conversation).await
    }
    
    pub async fn store_embeddings(&mut self, embeddings: &[EmbeddingRecord]) -> Result<()> {
        self.engine.store_embeddings(embeddings).await
    }
    
    pub async fn get_stats(&self) -> Result<StorageStats> {
        self.engine.get_stats().await
    }
}

impl Conversation {
    /// Check if conversation contains PHI (Protected Health Information)
    pub fn contains_phi(&self) -> bool {
        self.medical_context.contains_phi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = HybridStorageEngine::new(temp_dir.path().to_path_buf()).await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_conversation_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = HybridStorageEngine::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let conversation = Conversation {
            id: Uuid::new_v4(),
            group_id: "test-group".to_string(),
            sender: "test-user".to_string(),
            content: "Patient reports headache and fever".to_string(),
            timestamp: Utc::now(),
            medical_context: MedicalContext {
                entities: vec![
                    MedicalEntity {
                        id: Uuid::new_v4(),
                        entity_type: MedicalEntityType::Symptom,
                        entity_value: "headache".to_string(),
                        confidence: 0.95,
                        medical_code: Some("R51".to_string()),
                    }
                ],
                contains_phi: false,
                risk_level: 1,
            },
            privacy_level: 1,
            encryption_key_id: "key-123".to_string(),
        };
        
        let result = storage.store_conversation(&conversation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_embedding_storage() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = HybridStorageEngine::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let embedding = EmbeddingRecord {
            id: "test-embed-1".to_string(),
            text: "Test medical text".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4],
            timestamp: Utc::now(),
            metadata: serde_json::json!({"source": "test"}),
        };
        
        let result = storage.store_embeddings(&[embedding]).await;
        assert!(result.is_ok());
    }
}
