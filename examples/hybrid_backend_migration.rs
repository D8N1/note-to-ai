// File: examples/hybrid_backend_migration.rs
// Example demonstrating the migration from Arkworks to hybrid Arkworks+Barretenberg backend

use note_to_ai::{
    AdaptiveProver, DeviceCapabilities, MigratedZkPassport, PassportContext, 
    PassportData, ProofStrategy, ZkPassportConfig
};
use std::time::Instant;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🚀 Starting Hybrid Backend Migration Example");
    println!("============================================");
    
    // Phase 1: Device Capability Detection
    println!("\n📱 Phase 1: Device Capability Detection");
    let device_caps = DeviceCapabilities::detect_current_device();
    print_device_capabilities(&device_caps);
    
    // Phase 2: Adaptive Prover Setup
    println!("\n⚙️  Phase 2: Adaptive Prover Setup");
    let mut adaptive_prover = match AdaptiveProver::new().await {
        Ok(prover) => {
            println!("✅ Adaptive prover created successfully");
            prover
        }
        Err(e) => {
            println!("❌ Failed to create adaptive prover: {}", e);
            return Err(e.into());
        }
    };
    
    match adaptive_prover.initialize().await {
        Ok(_) => println!("✅ Adaptive prover initialized"),
        Err(e) => {
            println!("⚠️  Adaptive prover initialization failed: {}", e);
            println!("   This is expected without barretenberg-backend feature enabled");
        }
    }
    
    // Phase 3: Migrated zkPassport Setup
    println!("\n🔐 Phase 3: Migrated zkPassport Setup");
    let config = ZkPassportConfig {
        max_proving_time: std::time::Duration::from_secs(30),
        max_memory_usage_mb: if device_caps.is_mobile { 512 } else { 2048 },
        enable_concurrent_proving: !device_caps.is_mobile,
        require_proof_freshness: true,
        force_backend: None, // Let adaptive selection work
    };
    
    let migrated_passport = match MigratedZkPassport::new_with_config(config).await {
        Ok(passport) => {
            println!("✅ Migrated zkPassport created successfully");
            passport
        }
        Err(e) => {
            println!("⚠️  Migrated zkPassport creation failed: {}", e);
            println!("   This is expected without backend implementations");
            return demonstrate_strategy_selection();
        }
    };
    
    // Phase 4: Performance Estimation
    println!("\n📊 Phase 4: Performance Estimation");
    demonstrate_performance_estimation(&migrated_passport);
    
    // Phase 5: Test Different Contexts
    println!("\n🧪 Phase 5: Testing Different Contexts");
    test_different_contexts(&migrated_passport).await?;
    
    // Phase 6: Migration Process
    println!("\n🔄 Phase 6: Migration Process");
    demonstrate_migration_process(migrated_passport).await?;
    
    println!("\n🎉 Hybrid Backend Migration Example Completed Successfully!");
    
    Ok(())
}

fn print_device_capabilities(caps: &DeviceCapabilities) {
    println!("   Available Memory: {:.1} GB", caps.available_memory_gb);
    println!("   Is Mobile: {}", caps.is_mobile);
    println!("   Supports WASM: {}", caps.supports_wasm);
    println!("   Supports Multithreading: {}", caps.supports_multithreading);
    println!("   CPU Cores: {}", caps.cpu_cores);
    
    if caps.can_handle_arkworks() {
        println!("   ✅ Device can handle Arkworks (high-security operations)");
    } else {
        println!("   ⚠️  Device may struggle with Arkworks (recommend Barretenberg)");
    }
    
    if caps.prefers_barretenberg() {
        println!("   ✅ Device prefers Barretenberg (mobile-optimized)");
    } else {
        println!("   ℹ️  Device can use either backend effectively");
    }
}

fn demonstrate_performance_estimation(passport: &MigratedZkPassport) {
    let contexts = vec![
        PassportContext::MobileApp,
        PassportContext::WebBrowser,
        PassportContext::ServerSide,
        PassportContext::BatchProcessing,
    ];
    
    for context in contexts {
        if let Some(estimate) = passport.estimate_performance(&context) {
            println!("   Context: {:?}", context);
            println!("     Proving Time: {:?}", estimate.estimated_proving_time);
            println!("     Memory Usage: {:.1} MB", 
                estimate.estimated_memory_usage as f64 / 1024.0 / 1024.0);
            println!("     Verification Time: {:?}", estimate.estimated_verification_time);
            println!("     Proof Size: {} bytes", estimate.proof_size_bytes);
            println!("     Parallel Proving: {}", estimate.supports_parallel_proving);
            println!();
        }
    }
}

async fn test_different_contexts(passport: &MigratedZkPassport) -> Result<(), Box<dyn std::error::Error>> {
    let test_passport_data = create_test_passport_data();
    
    let contexts = vec![
        (PassportContext::MobileApp, "Mobile App"),
        (PassportContext::WebBrowser, "Web Browser"),
        (PassportContext::ServerSide, "Server Side"),
        (PassportContext::Testing, "Testing"),
    ];
    
    for (context, name) in contexts {
        println!("   Testing context: {}", name);
        
        let mut test_data = test_passport_data.clone();
        test_data.context = context;
        
        let start_time = Instant::now();
        
        match passport.prove_age_over(21, &test_data, None).await {
            Ok(proof) => {
                let proving_time = start_time.elapsed();
                println!("     ✅ Proof generated in {:?}", proving_time);
                println!("     Backend used: {}", proof.proof_data.backend_used);
                println!("     Proof size: {} bytes", proof.proof_data.proof_bytes.len());
                
                // Test verification
                let verification_reqs = note_to_ai::identity::zkpassport_migration::VerificationRequirements::default();
                match passport.verify_proof(&proof, &verification_reqs).await {
                    Ok(true) => println!("     ✅ Proof verified successfully"),
                    Ok(false) => println!("     ❌ Proof verification failed"),
                    Err(e) => println!("     ⚠️  Verification error: {}", e),
                }
            }
            Err(e) => {
                println!("     ⚠️  Proof generation failed: {}", e);
                println!("        This is expected without backend implementations");
            }
        }
        
        println!();
    }
    
    Ok(())
}

async fn demonstrate_migration_process(mut passport: MigratedZkPassport) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Starting migration execution...");
    
    match passport.execute_migration().await {
        Ok(_) => {
            let status = passport.migration_status();
            println!("   ✅ Migration completed successfully!");
            println!("   Migration completion: {:.1}%", status.completion_percentage());
            
            if let Some(proving_time) = status.mobile_proving_time {
                println!("   Mobile proving time: {:?}", proving_time);
            }
            
            // Save migration state
            if let Err(e) = passport.save_migration_state().await {
                println!("   ⚠️  Failed to save migration state: {}", e);
            } else {
                println!("   ✅ Migration state saved");
            }
        }
        Err(e) => {
            println!("   ⚠️  Migration failed: {}", e);
            println!("      This is expected without backend implementations");
            
            let status = passport.migration_status();
            println!("   Migration progress: {:.1}%", status.completion_percentage());
        }
    }
    
    Ok(())
}

fn demonstrate_strategy_selection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Demonstrating Strategy Selection (Backend-Independent)");
    
    let device_caps = DeviceCapabilities::detect_current_device();
    
    let strategies = vec![
        ProofStrategy::HighSecurity,
        ProofStrategy::MobileOptimized,
        ProofStrategy::UserFacing,
        ProofStrategy::BatchProcessing,
    ];
    
    for strategy in strategies {
        let backend = strategy.select_backend(&device_caps);
        println!("   Strategy: {:?} → Backend: {}", strategy, backend.name());
        
        let est_time = backend.estimated_proving_time(&device_caps);
        let est_memory = backend.estimated_memory_usage();
        
        println!("     Estimated proving time: {:?}", est_time);
        println!("     Estimated memory usage: {:.1} MB", 
            est_memory as f64 / 1024.0 / 1024.0);
        println!();
    }
    
    Ok(())
}

fn create_test_passport_data() -> PassportData {
    PassportData {
        signature: [1u8; 64],
        public_key: [2u8; 64],
        document_hash: [3u8; 32],
        date_of_birth: [0, 0, 7, 207, 0, 1, 0, 1], // 2000-01-01 (25 years old)
        country_code: [85, 83, 65], // "USA"
        merkle_root: [6u8; 32],
        merkle_path: vec![[4u8; 32]; 8], // 8-level merkle tree
        merkle_indices: vec![false; 8], // All left children for simplicity
        context: PassportContext::Testing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_device_capability_detection() {
        let caps = DeviceCapabilities::detect_current_device();
        assert!(caps.cpu_cores > 0);
        assert!(caps.available_memory_gb > 0.0);
    }
    
    #[test]
    fn test_strategy_selection() {
        let mobile_device = DeviceCapabilities {
            available_memory_gb: 3.0,
            is_mobile: true,
            supports_wasm: false,
            supports_multithreading: false,
            cpu_cores: 4,
        };
        
        let strategy = ProofStrategy::MobileOptimized;
        let backend = strategy.select_backend(&mobile_device);
        
        // Mobile optimized should select Barretenberg
        assert_eq!(backend.name(), "barretenberg-ultrahonk");
    }
    
    #[test]
    fn test_passport_data_creation() {
        let passport_data = create_test_passport_data();
        assert_eq!(passport_data.country_code, [85, 83, 65]); // "USA"
        assert_eq!(passport_data.merkle_path.len(), 8);
        assert!(matches!(passport_data.context, PassportContext::Testing));
    }
}
