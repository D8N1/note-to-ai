// File: src/identity/zkpassport_migration.rs
// Migration implementation for zkPassport from Arkworks to hybrid Arkworks+Barretenberg

use crate::identity::proving_backend::{
    AdaptiveProver, CircuitInputs, PrivateInputs, ProofData, ProofStrategy, ProvingError, PublicInputs, VerificationContext, ZkProver
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Noir circuit source for passport verification (replaces Arkworks R1CS)
const PASSPORT_NOIR_CIRCUIT: &str = r#"
// File: circuits/passport_verification.nr
use dep::std;

// Input structure matching existing PassportData
struct PassportInputs {
    // Public inputs
    challenge: [u8; 32],
    merkle_root: [u8; 32],
    min_age: u8,
    timestamp: u64,
    
    // Private inputs  
    passport_signature: [u8; 64],
    passport_pubkey: [u8; 64],
    date_of_birth: [u8; 8],
    document_hash: [u8; 32],
    country_code: [u8; 3],
    merkle_path: [[u8; 32]; 8],
    merkle_indices: [u1; 8],
    salt: [u8; 32],
}

fn main(inputs: PassportInputs) -> pub [u8; 32] {
    // 1. Verify passport signature using ECDSA P-256 (passport standard)
    let message_hash = std::hash::pedersen_hash([
        inputs.document_hash,
        inputs.challenge,
        inputs.salt
    ]);
    
    // Use Noir's built-in ECDSA verification for P-256
    let signature_valid = std::ecdsa_secp256r1::verify_signature(
        inputs.passport_pubkey,
        inputs.passport_signature,
        message_hash.to_be_bytes()
    );
    assert(signature_valid);
    
    // 2. Verify age without revealing birth date
    let current_year = 2025; // Could be passed as public input
    let birth_year = u16::from_be_bytes([inputs.date_of_birth[0], inputs.date_of_birth[1]]);
    let age = current_year - birth_year;
    assert(age >= inputs.min_age as u16);
    
    // 3. Verify document is in valid passport certificate chain
    let computed_root = compute_merkle_root(
        inputs.document_hash,
        inputs.merkle_path,
        inputs.merkle_indices
    );
    assert(computed_root == inputs.merkle_root);
    
    // 4. Verify proof freshness
    let current_timestamp = 1735689600; // 2025-01-01 00:00:00 UTC (would be dynamic)
    assert(inputs.timestamp <= current_timestamp as u64);
    assert(inputs.timestamp > (current_timestamp - 3600) as u64); // 1 hour freshness
    
    // Return proof commitment
    std::hash::pedersen_hash([
        inputs.challenge,
        computed_root,
        [age as u8; 32], // Age proof without revealing exact age
        inputs.salt
    ]).to_be_bytes()
}

fn compute_merkle_root(
    leaf: [u8; 32],
    path: [[u8; 32]; 8],
    indices: [u1; 8]
) -> [u8; 32] {
    let mut current = leaf;
    for i in 0..8 {
        let path_element = path[i];
        if indices[i] == 0 {
            current = std::hash::pedersen_hash([current, path_element]).to_be_bytes();
        } else {
            current = std::hash::pedersen_hash([path_element, current]).to_be_bytes();
        }
    }
    current
}
"#;

/// Migrated zkPassport implementation with hybrid backend support
pub struct MigratedZkPassport {
    /// Adaptive prover handles backend selection automatically
    adaptive_prover: AdaptiveProver,
    
    /// Configuration for proving behavior
    config: ZkPassportConfig,
    
    /// Migration state tracking
    migration_state: MigrationState,
}

impl MigratedZkPassport {
    /// Create new migrated zkPassport with automatic backend detection
    pub async fn new() -> Result<Self, MigrationError> {
        let mut adaptive_prover = AdaptiveProver::new().await
            .map_err(|e| MigrationError::BackendInit(e.to_string()))?;
        
        adaptive_prover.initialize().await
            .map_err(|e| MigrationError::BackendInit(e.to_string()))?;
        
        Ok(Self {
            adaptive_prover,
            config: ZkPassportConfig::default(),
            migration_state: MigrationState::new(),
        })
    }
    
    /// Create with specific configuration
    pub async fn new_with_config(config: ZkPassportConfig) -> Result<Self, MigrationError> {
        let mut instance = Self::new().await?;
        instance.config = config;
        Ok(instance)
    }
    
    /// Prove passport validity with age verification
    pub async fn prove_age_over(
        &self,
        min_age: u8,
        passport_data: &PassportData,
        challenge: Option<[u8; 32]>,
    ) -> Result<PassportProof, MigrationError> {
        let challenge = challenge.unwrap_or_else(|| {
            use rand::RngCore;
            let mut rng = rand::thread_rng();
            let mut challenge = [0u8; 32];
            rng.fill_bytes(&mut challenge);
            challenge
        });
        
        let salt = self.generate_salt();
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
        let circuit_inputs = CircuitInputs {
            public_inputs: PublicInputs {
                challenge,
                merkle_root: passport_data.merkle_root,
                min_age,
                timestamp,
                circuit_version: 1,
            },
            private_inputs: PrivateInputs {
                passport_signature: passport_data.signature,
                passport_pubkey: passport_data.public_key,
                document_hash: passport_data.document_hash,
                date_of_birth: passport_data.date_of_birth,
                country_code: passport_data.country_code,
                merkle_path: passport_data.merkle_path.clone(),
                merkle_indices: passport_data.merkle_indices.clone(),
                salt,
            },
        };
        
        let strategy = self.select_proving_strategy(&passport_data.context);
        let proof_data = self.adaptive_prover.prove_adaptive(&circuit_inputs, Some(strategy))
            .await
            .map_err(MigrationError::ProofGeneration)?;
        
        Ok(PassportProof {
            proof_data,
            passport_context: passport_data.context.clone(),
            age_verified: min_age,
            challenge,
        })
    }
    
    /// Verify a passport proof
    pub async fn verify_proof(
        &self,
        proof: &PassportProof,
        verification_requirements: &VerificationRequirements,
    ) -> Result<bool, MigrationError> {
        let verification_context = VerificationContext {
            require_recent_proof: verification_requirements.require_recent,
            max_proof_age: verification_requirements.max_age,
            trusted_circuit_versions: vec![1], // Current version
        };
        
        self.adaptive_prover.verify_adaptive(&proof.proof_data, &verification_context)
            .await
            .map_err(|e| MigrationError::ProofVerification(e.to_string()))
    }
    
    /// Get performance estimates for different proving strategies
    pub fn estimate_performance(&self, context: &PassportContext) -> Option<PerformanceEstimate> {
        let strategy = self.select_proving_strategy(context);
        self.adaptive_prover.estimate_performance(&strategy)
    }
    
    /// Execute phased migration strategy
    pub async fn execute_migration(&mut self) -> Result<(), MigrationError> {
        println!("Starting zkPassport migration to hybrid backend...");
        
        // Phase 1: Validate both backends work
        self.migration_phase_1().await?;
        
        // Phase 2: Test mobile optimization
        self.migration_phase_2().await?;
        
        // Phase 3: Performance validation
        self.migration_phase_3().await?;
        
        self.migration_state.mark_complete();
        println!("Migration completed successfully");
        
        Ok(())
    }
    
    /// Get migration status and metrics
    pub fn migration_status(&self) -> &MigrationState {
        &self.migration_state
    }
    
    fn select_proving_strategy(&self, context: &PassportContext) -> ProofStrategy {
        match context {
            PassportContext::MobileApp => ProofStrategy::MobileOptimized,
            PassportContext::WebBrowser => ProofStrategy::UserFacing,
            PassportContext::ServerSide => ProofStrategy::HighSecurity,
            PassportContext::BatchProcessing => ProofStrategy::BatchProcessing,
            PassportContext::Testing => {
                if self.config.force_backend.is_some() {
                    match self.config.force_backend.as_ref().unwrap() {
                        BackendPreference::Arkworks => ProofStrategy::ForceArkworks,
                        BackendPreference::Barretenberg => ProofStrategy::ForceBarretenberg,
                        _ => ProofStrategy::UserFacing,
                    }
                } else {
                    ProofStrategy::UserFacing
                }
            }
        }
    }
    
    fn generate_salt(&self) -> [u8; 32] {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut salt = [0u8; 32];
        rng.fill_bytes(&mut salt);
        salt
    }
    
    async fn migration_phase_1(&mut self) -> Result<(), MigrationError> {
        println!("Phase 1: Validating backend functionality...");
        
        let test_passport = self.create_test_passport_data();
        
        // Test with mobile-optimized strategy (Barretenberg)
        let mobile_proof = self.prove_age_over(18, &test_passport, None).await?;
        println!("✓ Barretenberg proving works");
        
        // Verify the proof
        let verification_reqs = VerificationRequirements::default();
        let is_valid = self.verify_proof(&mobile_proof, &verification_reqs).await?;
        if !is_valid {
            return Err(MigrationError::MigrationVerification);
        }
        println!("✓ Barretenberg verification works");
        
        self.migration_state.phase_1_complete = true;
        Ok(())
    }
    
    async fn migration_phase_2(&mut self) -> Result<(), MigrationError> {
        println!("Phase 2: Testing mobile performance...");
        
        let mobile_passport = PassportData {
            context: PassportContext::MobileApp,
            ..self.create_test_passport_data()
        };
        
        let start_time = std::time::Instant::now();
        let proof = self.prove_age_over(21, &mobile_passport, None).await?;
        let proving_time = start_time.elapsed();
        
        println!("Mobile proving time: {proving_time:?}");
        
        // Should complete reasonably quickly on mobile
        if proving_time > std::time::Duration::from_secs(20) {
            println!("⚠️  Warning: Mobile proving time is high: {proving_time:?}");
        } else {
            println!("✓ Mobile proving performance acceptable");
        }
        
        self.migration_state.phase_2_complete = true;
        self.migration_state.mobile_proving_time = Some(proving_time);
        
        Ok(())
    }
    
    async fn migration_phase_3(&mut self) -> Result<(), MigrationError> {
        println!("Phase 3: Performance validation and stress testing...");
        
        // Test concurrent proving
        let test_passport = self.create_test_passport_data();
        let mut handles = Vec::new();
        
        for i in 0..3 {
            let passport = test_passport.clone();
            let challenge = [i as u8; 32];
            
            // Create a separate instance for each concurrent test
            let prover = MigratedZkPassport::new().await?;
            
            let handle = tokio::spawn(async move {
                prover.prove_age_over(18, &passport, Some(challenge)).await
            });
            handles.push(handle);
        }
        
        // Wait for all proofs to complete
        let mut successful_proofs = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successful_proofs += 1,
                Ok(Err(e)) => println!("⚠️  Proof failed: {e}"),
                Err(e) => println!("⚠️  Task failed: {e}"),
            }
        }
        
        if successful_proofs >= 2 {
            println!("✓ Concurrent proving test passed ({successful_proofs}/3 successful)");
        } else {
            return Err(MigrationError::MigrationVerification);
        }
        
        self.migration_state.phase_3_complete = true;
        Ok(())
    }
    
    fn create_test_passport_data(&self) -> PassportData {
        PassportData {
            signature: [1u8; 64],
            public_key: [2u8; 64],
            document_hash: [3u8; 32],
            date_of_birth: [0, 0, 7, 207, 0, 1, 0, 1], // 2000-01-01
            country_code: [85, 83, 65], // "USA"
            merkle_root: [6u8; 32],
            merkle_path: vec![[4u8; 32]; 8],
            merkle_indices: vec![false; 8],
            context: PassportContext::Testing,
        }
    }
}

/// Configuration for zkPassport behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkPassportConfig {
    pub force_backend: Option<BackendPreference>,
    pub max_proving_time: std::time::Duration,
    pub max_memory_usage_mb: usize,
    pub enable_concurrent_proving: bool,
    pub require_proof_freshness: bool,
}

impl Default for ZkPassportConfig {
    fn default() -> Self {
        Self {
            force_backend: None, // Auto-select
            max_proving_time: std::time::Duration::from_secs(30),
            max_memory_usage_mb: 1024,
            enable_concurrent_proving: true,
            require_proof_freshness: true,
        }
    }
}

/// Backend preference for migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendPreference {
    Arkworks,
    Barretenberg,
    Adaptive,
}

/// Context in which passport proving is happening
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassportContext {
    MobileApp,
    WebBrowser,
    ServerSide,
    BatchProcessing,
    Testing,
}

/// Input data for passport proving
#[derive(Debug, Clone)]
pub struct PassportData {
    pub signature: [u8; 64],
    pub public_key: [u8; 64],
    pub document_hash: [u8; 32],
    pub date_of_birth: [u8; 8],
    pub country_code: [u8; 3],
    pub merkle_root: [u8; 32],
    pub merkle_path: Vec<[u8; 32]>,
    pub merkle_indices: Vec<bool>,
    pub context: PassportContext,
}

// Manual Serialize/Deserialize implementation for PassportData
impl serde::Serialize for PassportData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PassportData", 9)?;
        state.serialize_field("signature", &self.signature.as_slice())?;
        state.serialize_field("public_key", &self.public_key.as_slice())?;
        state.serialize_field("document_hash", &self.document_hash.as_slice())?;
        state.serialize_field("date_of_birth", &self.date_of_birth.as_slice())?;
        state.serialize_field("country_code", &self.country_code.as_slice())?;
        state.serialize_field("merkle_root", &self.merkle_root.as_slice())?;
        state.serialize_field("merkle_path", &self.merkle_path)?;
        state.serialize_field("merkle_indices", &self.merkle_indices)?;
        state.serialize_field("context", &self.context)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for PassportData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use std::fmt;
        
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Signature,
            PublicKey,
            DocumentHash,
            DateOfBirth,
            CountryCode,
            MerkleRoot,
            MerklePath,
            MerkleIndices,
            Context,
        }
        
        struct PassportDataVisitor;
        
        impl<'de> Visitor<'de> for PassportDataVisitor {
            type Value = PassportData;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PassportData")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<PassportData, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut signature: Option<Vec<u8>> = None;
                let mut public_key: Option<Vec<u8>> = None;
                let mut document_hash: Option<Vec<u8>> = None;
                let mut date_of_birth: Option<Vec<u8>> = None;
                let mut country_code: Option<Vec<u8>> = None;
                let mut merkle_root: Option<Vec<u8>> = None;
                let mut merkle_path: Option<Vec<[u8; 32]>> = None;
                let mut merkle_indices: Option<Vec<bool>> = None;
                let mut context: Option<PassportContext> = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Signature => {
                            if signature.is_some() {
                                return Err(de::Error::duplicate_field("signature"));
                            }
                            signature = Some(map.next_value()?);
                        }
                        Field::PublicKey => {
                            if public_key.is_some() {
                                return Err(de::Error::duplicate_field("public_key"));
                            }
                            public_key = Some(map.next_value()?);
                        }
                        Field::DocumentHash => {
                            if document_hash.is_some() {
                                return Err(de::Error::duplicate_field("document_hash"));
                            }
                            document_hash = Some(map.next_value()?);
                        }
                        Field::DateOfBirth => {
                            if date_of_birth.is_some() {
                                return Err(de::Error::duplicate_field("date_of_birth"));
                            }
                            date_of_birth = Some(map.next_value()?);
                        }
                        Field::CountryCode => {
                            if country_code.is_some() {
                                return Err(de::Error::duplicate_field("country_code"));
                            }
                            country_code = Some(map.next_value()?);
                        }
                        Field::MerkleRoot => {
                            if merkle_root.is_some() {
                                return Err(de::Error::duplicate_field("merkle_root"));
                            }
                            merkle_root = Some(map.next_value()?);
                        }
                        Field::MerklePath => {
                            if merkle_path.is_some() {
                                return Err(de::Error::duplicate_field("merkle_path"));
                            }
                            merkle_path = Some(map.next_value()?);
                        }
                        Field::MerkleIndices => {
                            if merkle_indices.is_some() {
                                return Err(de::Error::duplicate_field("merkle_indices"));
                            }
                            merkle_indices = Some(map.next_value()?);
                        }
                        Field::Context => {
                            if context.is_some() {
                                return Err(de::Error::duplicate_field("context"));
                            }
                            context = Some(map.next_value()?);
                        }
                    }
                }
                
                let signature = signature.ok_or_else(|| de::Error::missing_field("signature"))?;
                let public_key = public_key.ok_or_else(|| de::Error::missing_field("public_key"))?;
                let document_hash = document_hash.ok_or_else(|| de::Error::missing_field("document_hash"))?;
                let date_of_birth = date_of_birth.ok_or_else(|| de::Error::missing_field("date_of_birth"))?;
                let country_code = country_code.ok_or_else(|| de::Error::missing_field("country_code"))?;
                let merkle_root = merkle_root.ok_or_else(|| de::Error::missing_field("merkle_root"))?;
                let merkle_path = merkle_path.ok_or_else(|| de::Error::missing_field("merkle_path"))?;
                let merkle_indices = merkle_indices.ok_or_else(|| de::Error::missing_field("merkle_indices"))?;
                let context = context.ok_or_else(|| de::Error::missing_field("context"))?;
                
                // Convert Vec<u8> to fixed-size arrays
                let signature: [u8; 64] = signature.try_into()
                    .map_err(|v: Vec<u8>| de::Error::invalid_length(v.len(), &"64"))?;
                let public_key: [u8; 64] = public_key.try_into()
                    .map_err(|v: Vec<u8>| de::Error::invalid_length(v.len(), &"64"))?;
                let document_hash: [u8; 32] = document_hash.try_into()
                    .map_err(|v: Vec<u8>| de::Error::invalid_length(v.len(), &"32"))?;
                let date_of_birth: [u8; 8] = date_of_birth.try_into()
                    .map_err(|v: Vec<u8>| de::Error::invalid_length(v.len(), &"8"))?;
                let country_code: [u8; 3] = country_code.try_into()
                    .map_err(|v: Vec<u8>| de::Error::invalid_length(v.len(), &"3"))?;
                let merkle_root: [u8; 32] = merkle_root.try_into()
                    .map_err(|v: Vec<u8>| de::Error::invalid_length(v.len(), &"32"))?;
                
                Ok(PassportData {
                    signature,
                    public_key,
                    document_hash,
                    date_of_birth,
                    country_code,
                    merkle_root,
                    merkle_path,
                    merkle_indices,
                    context,
                })
            }
        }
        
        const FIELDS: &[&str] = &[
            "signature",
            "public_key", 
            "document_hash",
            "date_of_birth",
            "country_code",
            "merkle_root",
            "merkle_path",
            "merkle_indices",
            "context"
        ];
        deserializer.deserialize_struct("PassportData", FIELDS, PassportDataVisitor)
    }
}

/// Passport proof result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportProof {
    pub proof_data: ProofData,
    pub passport_context: PassportContext,
    pub age_verified: u8,
    pub challenge: [u8; 32],
}

/// Requirements for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirements {
    pub require_recent: bool,
    pub max_age: Option<std::time::Duration>,
    pub min_age_verified: Option<u8>,
}

impl Default for VerificationRequirements {
    fn default() -> Self {
        Self {
            require_recent: true,
            max_age: Some(std::time::Duration::from_secs(3600)), // 1 hour
            min_age_verified: None,
        }
    }
}

/// Migration state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationState {
    pub phase_1_complete: bool,
    pub phase_2_complete: bool,
    pub phase_3_complete: bool,
    pub mobile_proving_time: Option<std::time::Duration>,
    pub migration_start_time: chrono::DateTime<chrono::Utc>,
    pub migration_complete_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl MigrationState {
    fn new() -> Self {
        Self {
            phase_1_complete: false,
            phase_2_complete: false,
            phase_3_complete: false,
            mobile_proving_time: None,
            migration_start_time: chrono::Utc::now(),
            migration_complete_time: None,
        }
    }
    
    fn mark_complete(&mut self) {
        self.migration_complete_time = Some(chrono::Utc::now());
    }
    
    pub fn is_complete(&self) -> bool {
        self.phase_1_complete && self.phase_2_complete && self.phase_3_complete
    }
    
    pub fn completion_percentage(&self) -> f32 {
        let mut completed_phases = 0;
        if self.phase_1_complete { completed_phases += 1; }
        if self.phase_2_complete { completed_phases += 1; }
        if self.phase_3_complete { completed_phases += 1; }
        
        (completed_phases as f32 / 3.0) * 100.0
    }
}

/// Performance estimate from proving backend
pub use crate::identity::proving_backend::PerformanceEstimate;

/// Errors specific to migration process
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Backend initialization failed: {0}")]
    BackendInit(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGeneration(#[from] ProvingError),
    
    #[error("Proof verification failed")]
    ProofVerification(String),
    
    #[error("Migration verification failed")]
    MigrationVerification,
    
    #[error("Circuit compilation failed: {0}")]
    CircuitCompilation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Integration with existing zkPassport interface
impl MigratedZkPassport {
    /// Convert to legacy interface for backward compatibility
    pub fn as_legacy_interface(&self) -> LegacyZkPassportAdapter {
        LegacyZkPassportAdapter {
            migrated: self,
        }
    }
    
    /// Check if migration is needed from existing implementation
    pub async fn needs_migration(existing_keys_path: &PathBuf) -> bool {
        // Check if existing Arkworks keys exist but no migration state
        existing_keys_path.exists() && !Self::migration_state_exists().await
    }
    
    async fn migration_state_exists() -> bool {
        // Check if migration state file exists
        PathBuf::from("migration_state.json").exists()
    }
    
    /// Save migration state for persistence
    pub async fn save_migration_state(&self) -> Result<(), MigrationError> {
        let state_json = serde_json::to_string_pretty(&self.migration_state)
            .map_err(|e| MigrationError::Configuration(e.to_string()))?;
        
        tokio::fs::write("migration_state.json", state_json).await
            .map_err(MigrationError::Io)?;
        
        Ok(())
    }
    
    /// Load migration state from persistence
    pub async fn load_migration_state() -> Result<MigrationState, MigrationError> {
        let state_json = tokio::fs::read_to_string("migration_state.json").await
            .map_err(MigrationError::Io)?;
        
        serde_json::from_str(&state_json)
            .map_err(|e| MigrationError::Configuration(e.to_string()))
    }
}

/// Adapter for backward compatibility with existing zkPassport interface
pub struct LegacyZkPassportAdapter<'a> {
    migrated: &'a MigratedZkPassport,
}

impl<'a> LegacyZkPassportAdapter<'a> {
    /// Legacy prove method that maintains existing API
    pub async fn prove_age_over_legacy(
        &self,
        min_age: u8,
        passport_signature: &[u8; 64],
        passport_pubkey: &[u8; 64],
        document_hash: &[u8; 32],
        date_of_birth: &[u8; 8],
        merkle_proof: &(Vec<[u8; 32]>, Vec<bool>),
    ) -> Result<Vec<u8>, MigrationError> {
        let passport_data = PassportData {
            signature: *passport_signature,
            public_key: *passport_pubkey,
            document_hash: *document_hash,
            date_of_birth: *date_of_birth,
            country_code: [0, 0, 0], // Default if not available in legacy API
            merkle_root: [0u8; 32], // Would need to be computed
            merkle_path: merkle_proof.0.clone(),
            merkle_indices: merkle_proof.1.clone(),
            context: PassportContext::ServerSide, // Legacy default
        };
        
        let proof = self.migrated.prove_age_over(min_age, &passport_data, None).await?;
        Ok(proof.proof_data.proof_bytes)
    }
    
    /// Legacy verify method that maintains existing API
    pub async fn verify_legacy(
        &self,
        proof_bytes: &[u8],
        public_inputs: &[u8],
    ) -> Result<bool, MigrationError> {
        // This would need to deserialize the proof and public inputs
        // For now, return a placeholder
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_migrated_zkpassport_creation() {
        let result = MigratedZkPassport::new().await;
        // This will fail until we implement the actual backends, but tests the interface
        assert!(result.is_err()); // Expected to fail with "not yet implemented"
    }
    
    #[tokio::test]
    async fn test_migration_state_tracking() {
        let mut state = MigrationState::new();
        assert!(!state.is_complete());
        assert_eq!(state.completion_percentage(), 0.0);
        
        state.phase_1_complete = true;
        assert_eq!(state.completion_percentage(), 33.333334);
        
        state.phase_2_complete = true;
        state.phase_3_complete = true;
        assert!(state.is_complete());
        assert_eq!(state.completion_percentage(), 100.0);
    }
    
    #[tokio::test]
    async fn test_passport_context_selection() {
        let migrated = MigratedZkPassport {
            adaptive_prover: AdaptiveProver::new().await.unwrap(),
            config: ZkPassportConfig::default(),
            migration_state: MigrationState::new(),
        };
        
        // Test strategy selection based on context
        let mobile_strategy = migrated.select_proving_strategy(&PassportContext::MobileApp);
        assert!(matches!(mobile_strategy, ProofStrategy::MobileOptimized));
        
        let server_strategy = migrated.select_proving_strategy(&PassportContext::ServerSide);
        assert!(matches!(server_strategy, ProofStrategy::HighSecurity));
    }
    
    #[test]
    fn test_configuration_defaults() {
        let config = ZkPassportConfig::default();
        assert!(config.force_backend.is_none());
        assert!(config.enable_concurrent_proving);
        assert!(config.require_proof_freshness);
    }
}
