// File: src/identity/proving_backend.rs
// Hybrid proving backend for zkPassport with Arkworks + Barretenberg UltraHonk

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Device capability detection for optimal backend selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub available_memory_gb: f32,
    pub is_mobile: bool,
    pub supports_wasm: bool,
    pub supports_multithreading: bool,
    pub cpu_cores: usize,
}

impl DeviceCapabilities {
    /// Detect current device capabilities for optimal backend selection
    pub fn detect_current_device() -> Self {
        Self {
            available_memory_gb: Self::detect_available_memory(),
            is_mobile: cfg!(any(target_os = "ios", target_os = "android")) 
                      || Self::is_mobile_browser(),
            supports_wasm: cfg!(target_arch = "wasm32"),
            supports_multithreading: !cfg!(target_os = "ios") && num_cpus::get() > 1,
            cpu_cores: num_cpus::get(),
        }
    }
    
    fn detect_available_memory() -> f32 {
        if cfg!(target_os = "ios") {
            2.0 // Conservative estimate for iOS devices
        } else if cfg!(target_os = "android") {
            3.0 // Conservative estimate for Android devices
        } else if cfg!(target_arch = "wasm32") {
            1.0 // Browser environment is very limited
        } else {
            8.0 // Desktop default - assume sufficient memory
        }
    }
    
    fn is_mobile_browser() -> bool {
        // In WASM context, check for mobile user agent patterns
        cfg!(target_arch = "wasm32")
    }
    
    /// Determine if this device is suitable for Arkworks proving
    pub fn can_handle_arkworks(&self) -> bool {
        self.available_memory_gb >= 4.0 && !self.is_mobile
    }
    
    /// Determine if this device should use Barretenberg for better UX
    pub fn prefers_barretenberg(&self) -> bool {
        self.is_mobile || self.available_memory_gb < 4.0 || self.supports_wasm
    }
}

/// Proving strategy based on context and device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStrategy {
    /// High security server-side operations - use Arkworks
    HighSecurity,
    /// Mobile or resource-constrained - use Barretenberg
    MobileOptimized,
    /// User-facing interactive proofs - prefer Barretenberg
    UserFacing,
    /// Batch processing - use most efficient backend
    BatchProcessing,
    /// Force specific backend for testing
    ForceArkworks,
    ForceBarretenberg,
}

impl ProofStrategy {
    /// Select optimal proving backend based on strategy and device capabilities
    pub fn select_backend(&self, device: &DeviceCapabilities) -> ProvingBackend {
        match self {
            ProofStrategy::HighSecurity => {
                if device.can_handle_arkworks() {
                    ProvingBackend::Arkworks
                } else {
                    ProvingBackend::Barretenberg // Fallback for constrained devices
                }
            }
            ProofStrategy::MobileOptimized | ProofStrategy::UserFacing => {
                ProvingBackend::Barretenberg
            }
            ProofStrategy::BatchProcessing => {
                if device.cpu_cores >= 8 && device.available_memory_gb >= 8.0 {
                    ProvingBackend::Arkworks // Better for parallel batch processing
                } else {
                    ProvingBackend::Barretenberg
                }
            }
            ProofStrategy::ForceArkworks => ProvingBackend::Arkworks,
            ProofStrategy::ForceBarretenberg => ProvingBackend::Barretenberg,
        }
    }
}

/// Unified proving backend enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvingBackend {
    /// Arkworks with Groth16 - High security, mature, server-optimized
    Arkworks,
    /// Barretenberg with UltraHonk - Mobile-optimized, faster proving
    Barretenberg,
}

impl ProvingBackend {
    pub fn name(&self) -> &'static str {
        match self {
            ProvingBackend::Arkworks => "arkworks-groth16",
            ProvingBackend::Barretenberg => "barretenberg-ultrahonk",
        }
    }
    
    /// Estimate proving time for this backend on given device
    pub fn estimated_proving_time(&self, device: &DeviceCapabilities) -> Duration {
        match self {
            ProvingBackend::Arkworks => {
                if device.is_mobile {
                    Duration::from_secs(60) // Very slow on mobile
                } else if device.cpu_cores >= 8 {
                    Duration::from_secs(15) // Fast on server hardware
                } else {
                    Duration::from_secs(30) // Moderate on desktop
                }
            }
            ProvingBackend::Barretenberg => {
                if device.is_mobile {
                    if device.available_memory_gb < 3.0 {
                        Duration::from_secs(15) // Low-end mobile
                    } else {
                        Duration::from_secs(8)  // Modern mobile
                    }
                } else {
                    Duration::from_secs(3) // Very fast on desktop
                }
            }
        }
    }
    
    /// Estimate peak memory usage for this backend
    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            ProvingBackend::Arkworks => 1024 * 1024 * 1024 * 2, // ~2GB
            ProvingBackend::Barretenberg => 1024 * 1024 * 512,   // ~512MB
        }
    }
}

/// Unified interface for ZK proving across different backends
#[async_trait::async_trait]
pub trait ZkProver: Send + Sync {
    /// Generate a proof for the given circuit inputs
    async fn prove(
        &self,
        circuit_inputs: &CircuitInputs,
        proving_context: &ProvingContext,
    ) -> Result<ProofData, ProvingError>;
    
    /// Verify a proof with public inputs
    async fn verify(
        &self,
        proof: &ProofData,
        public_inputs: &PublicInputs,
        verification_context: &VerificationContext,
    ) -> Result<bool, ProvingError>;
    
    /// Get backend name for logging and metrics
    fn backend_name(&self) -> &str;
    
    /// Get estimated performance characteristics
    fn performance_characteristics(&self, device: &DeviceCapabilities) -> PerformanceEstimate;
    
    /// Check if this prover is ready (circuit compiled, keys loaded, etc.)
    async fn is_ready(&self) -> bool;
    
    /// Warm up the prover (compile circuits, load keys, etc.)
    async fn initialize(&mut self) -> Result<(), ProvingError>;
}

/// Performance characteristics for a prover on a specific device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEstimate {
    pub estimated_proving_time: Duration,
    pub estimated_memory_usage: usize,
    pub estimated_verification_time: Duration,
    pub proof_size_bytes: usize,
    pub supports_parallel_proving: bool,
}

/// Circuit inputs for proving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInputs {
    pub public_inputs: PublicInputs,
    pub private_inputs: PrivateInputs,
}

/// Public inputs visible in the proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    pub challenge: [u8; 32],
    pub merkle_root: [u8; 32],
    pub min_age: u8,
    pub timestamp: u64,
    pub circuit_version: u32,
}

/// Private inputs hidden by the proof
#[derive(Debug, Clone)]
pub struct PrivateInputs {
    pub passport_signature: [u8; 64],
    pub passport_pubkey: [u8; 64],
    pub document_hash: [u8; 32],
    pub date_of_birth: [u8; 8],
    pub country_code: [u8; 3],
    pub merkle_path: Vec<[u8; 32]>,
    pub merkle_indices: Vec<bool>,
    pub salt: [u8; 32],
}

// Manual Serialize/Deserialize implementation for PrivateInputs to handle large arrays
impl serde::Serialize for PrivateInputs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PrivateInputs", 8)?;
        state.serialize_field("passport_signature", &self.passport_signature.as_slice())?;
        state.serialize_field("passport_pubkey", &self.passport_pubkey.as_slice())?;
        state.serialize_field("document_hash", &self.document_hash.as_slice())?;
        state.serialize_field("date_of_birth", &self.date_of_birth.as_slice())?;
        state.serialize_field("country_code", &self.country_code.as_slice())?;
        state.serialize_field("merkle_path", &self.merkle_path)?;
        state.serialize_field("merkle_indices", &self.merkle_indices)?;
        state.serialize_field("salt", &self.salt.as_slice())?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for PrivateInputs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use std::fmt;
        
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            PassportSignature,
            PassportPubkey,
            DocumentHash,
            DateOfBirth,
            CountryCode,
            MerklePath,
            MerkleIndices,
            Salt,
        }
        
        struct PrivateInputsVisitor;
        
        impl<'de> Visitor<'de> for PrivateInputsVisitor {
            type Value = PrivateInputs;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PrivateInputs")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<PrivateInputs, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut passport_signature: Option<Vec<u8>> = None;
                let mut passport_pubkey: Option<Vec<u8>> = None;
                let mut document_hash: Option<Vec<u8>> = None;
                let mut date_of_birth: Option<Vec<u8>> = None;
                let mut country_code: Option<Vec<u8>> = None;
                let mut merkle_path: Option<Vec<[u8; 32]>> = None;
                let mut merkle_indices: Option<Vec<bool>> = None;
                let mut salt: Option<Vec<u8>> = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::PassportSignature => {
                            if passport_signature.is_some() {
                                return Err(de::Error::duplicate_field("passport_signature"));
                            }
                            passport_signature = Some(map.next_value()?);
                        }
                        Field::PassportPubkey => {
                            if passport_pubkey.is_some() {
                                return Err(de::Error::duplicate_field("passport_pubkey"));
                            }
                            passport_pubkey = Some(map.next_value()?);
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
                        Field::Salt => {
                            if salt.is_some() {
                                return Err(de::Error::duplicate_field("salt"));
                            }
                            salt = Some(map.next_value()?);
                        }
                    }
                }
                
                let passport_signature = passport_signature.ok_or_else(|| de::Error::missing_field("passport_signature"))?;
                let passport_pubkey = passport_pubkey.ok_or_else(|| de::Error::missing_field("passport_pubkey"))?;
                let document_hash = document_hash.ok_or_else(|| de::Error::missing_field("document_hash"))?;
                let date_of_birth = date_of_birth.ok_or_else(|| de::Error::missing_field("date_of_birth"))?;
                let country_code = country_code.ok_or_else(|| de::Error::missing_field("country_code"))?;
                let merkle_path = merkle_path.ok_or_else(|| de::Error::missing_field("merkle_path"))?;
                let merkle_indices = merkle_indices.ok_or_else(|| de::Error::missing_field("merkle_indices"))?;
                let salt = salt.ok_or_else(|| de::Error::missing_field("salt"))?;
                
                // Convert Vec<u8> to fixed-size arrays
                let passport_signature: [u8; 64] = passport_signature.clone().try_into()
                    .map_err(|_| de::Error::invalid_length(passport_signature.len(), &"64"))?;
                let passport_pubkey: [u8; 64] = passport_pubkey.clone().try_into()
                    .map_err(|_| de::Error::invalid_length(passport_pubkey.len(), &"64"))?;
                let document_hash: [u8; 32] = document_hash.clone().try_into()
                    .map_err(|_| de::Error::invalid_length(document_hash.len(), &"32"))?;
                let date_of_birth: [u8; 8] = date_of_birth.clone().try_into()
                    .map_err(|_| de::Error::invalid_length(date_of_birth.len(), &"8"))?;
                let country_code: [u8; 3] = country_code.clone().try_into()
                    .map_err(|_| de::Error::invalid_length(country_code.len(), &"3"))?;
                let salt: [u8; 32] = salt.clone().try_into()
                    .map_err(|_| de::Error::invalid_length(salt.len(), &"32"))?;
                
                Ok(PrivateInputs {
                    passport_signature,
                    passport_pubkey,
                    document_hash,
                    date_of_birth,
                    country_code,
                    merkle_path,
                    merkle_indices,
                    salt,
                })
            }
        }
        
        const FIELDS: &[&str] = &[
            "passport_signature",
            "passport_pubkey", 
            "document_hash",
            "date_of_birth",
            "country_code",
            "merkle_path",
            "merkle_indices",
            "salt"
        ];
        deserializer.deserialize_struct("PrivateInputs", FIELDS, PrivateInputsVisitor)
    }
}

/// Context for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvingContext {
    pub strategy: ProofStrategy,
    pub device_capabilities: DeviceCapabilities,
    pub max_proving_time: Option<Duration>,
    pub max_memory_usage: Option<usize>,
    pub parallel_proving: bool,
}

impl ProvingContext {
    pub fn new(strategy: ProofStrategy) -> Self {
        let device_capabilities = DeviceCapabilities::detect_current_device();
        
        Self {
            strategy,
            device_capabilities,
            max_proving_time: None,
            max_memory_usage: None,
            parallel_proving: false,
        }
    }
    
    pub fn mobile_optimized() -> Self {
        let mut context = Self::new(ProofStrategy::MobileOptimized);
        context.max_proving_time = Some(Duration::from_secs(15));
        context.max_memory_usage = Some(512 * 1024 * 1024); // 512MB
        context.parallel_proving = false; // Avoid threads on mobile
        context
    }
    
    pub fn high_security() -> Self {
        let mut context = Self::new(ProofStrategy::HighSecurity);
        context.max_proving_time = Some(Duration::from_secs(60));
        context.max_memory_usage = Some(4 * 1024 * 1024 * 1024); // 4GB
        context.parallel_proving = true;
        context
    }
}

/// Context for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationContext {
    pub require_recent_proof: bool,
    pub max_proof_age: Option<Duration>,
    pub trusted_circuit_versions: Vec<u32>,
}

impl Default for VerificationContext {
    fn default() -> Self {
        Self {
            require_recent_proof: true,
            max_proof_age: Some(Duration::from_secs(3600)), // 1 hour
            trusted_circuit_versions: vec![1], // Current version
        }
    }
}

/// Proof data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    pub proof_bytes: Vec<u8>,
    pub backend_used: String,
    pub proof_timestamp: chrono::DateTime<chrono::Utc>,
    pub public_inputs: PublicInputs,
    pub performance_metrics: Option<ProofMetrics>,
    pub circuit_version: u32,
}

/// Performance metrics for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetrics {
    pub proving_time: Duration,
    pub memory_usage_peak: usize,
    pub cpu_usage_percent: f32,
    pub device_capabilities: DeviceCapabilities,
}

/// Errors that can occur during proving or verification
#[derive(Debug, thiserror::Error)]
pub enum ProvingError {
    #[error("Circuit compilation failed: {message}")]
    CircuitCompilation { message: String },
    
    #[error("Proof generation failed: {message}")]
    ProofGeneration { message: String },
    
    #[error("Proof verification failed: {message}")]
    ProofVerification { message: String },
    
    #[error("Backend initialization failed: {message}")]
    BackendInitialization { message: String },
    
    #[error("Invalid circuit inputs: {message}")]
    InvalidInputs { message: String },
    
    #[error("Resource constraints exceeded: {constraint}")]
    ResourceConstraints { constraint: String },
    
    #[error("Backend not available: {backend}")]
    BackendUnavailable { backend: String },
    
    #[error("Proof expired: age={age:?}, max_age={max_age:?}")]
    ProofExpired { age: Duration, max_age: Duration },
    
    #[error("Unsupported circuit version: {version}")]
    UnsupportedCircuitVersion { version: u32 },
    
    #[error("I/O error: {source}")]
    Io { 
        #[from]
        source: std::io::Error 
    },
    
    #[error("Serialization error: {source}")]
    Serialization { 
        #[from]
        source: bincode::Error 
    },
}

/// Adaptive prover that selects optimal backend based on context
pub struct AdaptiveProver {
    arkworks_prover: Option<Arc<RwLock<Box<dyn ZkProver + Send + Sync>>>>,
    barretenberg_prover: Option<Arc<RwLock<Box<dyn ZkProver + Send + Sync>>>>,
    device_capabilities: DeviceCapabilities,
    default_strategy: ProofStrategy,
}

impl AdaptiveProver {
    /// Create a new adaptive prover with automatic backend selection
    pub async fn new() -> Result<Self, ProvingError> {
        let device_capabilities = DeviceCapabilities::detect_current_device();
        
        // Determine default strategy based on device
        let default_strategy = if device_capabilities.prefers_barretenberg() {
            ProofStrategy::MobileOptimized
        } else if device_capabilities.can_handle_arkworks() {
            ProofStrategy::HighSecurity
        } else {
            ProofStrategy::UserFacing
        };
        
        Ok(Self {
            arkworks_prover: None,
            barretenberg_prover: None,
            device_capabilities,
            default_strategy,
        })
    }
    
    /// Initialize provers based on device capabilities and strategy
    pub async fn initialize(&mut self) -> Result<(), ProvingError> {
        // Always try to initialize Barretenberg for better UX
        if let Ok(bb_prover) = self.create_barretenberg_prover().await {
            self.barretenberg_prover = Some(Arc::new(RwLock::new(bb_prover)));
        }
        
        // Initialize Arkworks if device can handle it
        if self.device_capabilities.can_handle_arkworks() {
            if let Ok(ark_prover) = self.create_arkworks_prover().await {
                self.arkworks_prover = Some(Arc::new(RwLock::new(ark_prover)));
            }
        }
        
        // Ensure at least one backend is available
        if self.arkworks_prover.is_none() && self.barretenberg_prover.is_none() {
            return Err(ProvingError::BackendInitialization {
                message: "No proving backends available".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Select optimal prover based on strategy and device capabilities
    pub fn select_prover(&self, strategy: &ProofStrategy) -> Result<Arc<RwLock<Box<dyn ZkProver + Send + Sync>>>, ProvingError> {
        let backend = strategy.select_backend(&self.device_capabilities);
        
        match backend {
            ProvingBackend::Arkworks => {
                self.arkworks_prover.clone().ok_or_else(|| ProvingError::BackendUnavailable {
                    backend: "arkworks".to_string(),
                })
            }
            ProvingBackend::Barretenberg => {
                self.barretenberg_prover.clone().ok_or_else(|| ProvingError::BackendUnavailable {
                    backend: "barretenberg".to_string(),
                })
            }
        }
    }
    
    /// Prove with automatic backend selection
    pub async fn prove_adaptive(
        &self,
        circuit_inputs: &CircuitInputs,
        strategy: Option<ProofStrategy>,
    ) -> Result<ProofData, ProvingError> {
        let strategy = strategy.unwrap_or_else(|| self.default_strategy.clone());
        let proving_context = ProvingContext::new(strategy.clone());
        
        let prover = self.select_prover(&strategy)?;
        let prover_lock = prover.read().await;
        
        prover_lock.prove(circuit_inputs, &proving_context).await
    }
    
    /// Verify proof with automatic backend detection
    pub async fn verify_adaptive(
        &self,
        proof: &ProofData,
        verification_context: &VerificationContext,
    ) -> Result<bool, ProvingError> {
        // Select prover based on backend used to generate proof
        let prover = if proof.backend_used == "arkworks-groth16" {
            self.select_prover(&ProofStrategy::ForceArkworks)?
        } else {
            self.select_prover(&ProofStrategy::ForceBarretenberg)?
        };
        
        let prover_lock = prover.read().await;
        prover_lock.verify(proof, &proof.public_inputs, verification_context).await
    }
    
    /// Get performance estimates for different strategies
    pub fn estimate_performance(&self, strategy: &ProofStrategy) -> Option<PerformanceEstimate> {
        let backend = strategy.select_backend(&self.device_capabilities);
        
        match backend {
            ProvingBackend::Arkworks => {
                self.arkworks_prover.as_ref().map(|p| {
                    // Would get this from the actual prover, for now estimate
                    PerformanceEstimate {
                        estimated_proving_time: backend.estimated_proving_time(&self.device_capabilities),
                        estimated_memory_usage: backend.estimated_memory_usage(),
                        estimated_verification_time: Duration::from_millis(100),
                        proof_size_bytes: 192, // Groth16 proof size
                        supports_parallel_proving: !self.device_capabilities.is_mobile,
                    }
                })
            }
            ProvingBackend::Barretenberg => {
                self.barretenberg_prover.as_ref().map(|p| {
                    PerformanceEstimate {
                        estimated_proving_time: backend.estimated_proving_time(&self.device_capabilities),
                        estimated_memory_usage: backend.estimated_memory_usage(),
                        estimated_verification_time: Duration::from_millis(50),
                        proof_size_bytes: 512, // UltraHonk proof size (variable)
                        supports_parallel_proving: true,
                    }
                })
            }
        }
    }
    
    async fn create_arkworks_prover(&self) -> Result<Box<dyn ZkProver + Send + Sync>, ProvingError> {
        use crate::identity::arkworks_prover::ArkworksProver;
        
        let mut prover = ArkworksProver::new().await?;
        prover.initialize().await?;
        
        Ok(Box::new(prover))
    }
    
    async fn create_barretenberg_prover(&self) -> Result<Box<dyn ZkProver + Send + Sync>, ProvingError> {
        use crate::identity::barretenberg_prover::BarretenbergProver;
        
        // Load the passport verification circuit
        let circuit_source = include_str!("../../circuits/passport_verification.nr");
        let mut prover = BarretenbergProver::new(circuit_source).await?;
        prover.initialize().await?;
        
        Ok(Box::new(prover))
    }
}

/// Configuration for proving backend selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvingConfig {
    pub preferred_backend: Option<ProvingBackend>,
    pub fallback_enabled: bool,
    pub max_proving_time: Duration,
    pub max_memory_usage: usize,
    pub enable_parallel_proving: bool,
    pub mobile_optimization: bool,
}

impl Default for ProvingConfig {
    fn default() -> Self {
        let device = DeviceCapabilities::detect_current_device();
        
        Self {
            preferred_backend: None, // Auto-select based on device
            fallback_enabled: true,
            max_proving_time: if device.is_mobile {
                Duration::from_secs(15)
            } else {
                Duration::from_secs(60)
            },
            max_memory_usage: if device.is_mobile {
                512 * 1024 * 1024 // 512MB
            } else {
                2 * 1024 * 1024 * 1024 // 2GB
            },
            enable_parallel_proving: !device.is_mobile,
            mobile_optimization: device.is_mobile,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_capability_detection() {
        let caps = DeviceCapabilities::detect_current_device();
        assert!(caps.cpu_cores > 0);
        assert!(caps.available_memory_gb > 0.0);
    }
    
    #[test]
    fn test_backend_selection() {
        let mobile_device = DeviceCapabilities {
            available_memory_gb: 3.0,
            is_mobile: true,
            supports_wasm: false,
            supports_multithreading: false,
            cpu_cores: 4,
        };
        
        let desktop_device = DeviceCapabilities {
            available_memory_gb: 16.0,
            is_mobile: false,
            supports_wasm: false,
            supports_multithreading: true,
            cpu_cores: 8,
        };
        
        // Mobile should prefer Barretenberg
        let mobile_strategy = ProofStrategy::MobileOptimized;
        assert!(matches!(
            mobile_strategy.select_backend(&mobile_device),
            ProvingBackend::Barretenberg
        ));
        
        // Desktop high security should use Arkworks if available
        let security_strategy = ProofStrategy::HighSecurity;
        assert!(matches!(
            security_strategy.select_backend(&desktop_device),
            ProvingBackend::Arkworks
        ));
    }
    
    #[test]
    fn test_performance_estimates() {
        let mobile_device = DeviceCapabilities {
            available_memory_gb: 3.0,
            is_mobile: true,
            supports_wasm: false,
            supports_multithreading: false,
            cpu_cores: 4,
        };
        
        let bb_time = ProvingBackend::Barretenberg.estimated_proving_time(&mobile_device);
        let ark_time = ProvingBackend::Arkworks.estimated_proving_time(&mobile_device);
        
        // Barretenberg should be faster on mobile
        assert!(bb_time < ark_time);
        
        let bb_memory = ProvingBackend::Barretenberg.estimated_memory_usage();
        let ark_memory = ProvingBackend::Arkworks.estimated_memory_usage();
        
        // Barretenberg should use less memory
        assert!(bb_memory < ark_memory);
    }
}
