use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use note_to_ai::vault::indexer::VaultIndexer;
use note_to_ai::obsidian::ObsidianVault;
use note_to_ai::config::Settings;
use std::path::PathBuf;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

// Benchmark document indexing performance
fn benchmark_vault_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("vault_indexing");
    group.measurement_time(Duration::from_secs(10));
    
    // Test with different file counts
    for file_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("index_files", file_count),
            file_count,
            |b, &file_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_setup(
                        || setup_test_vault(file_count),
                        |temp_dir| async move {
                            let vault_path = temp_dir.path().to_path_buf();
                            let db_path = vault_path.join("test.db");
                            let indexer = VaultIndexer::new(db_path, vault_path.clone()).unwrap();
                            black_box(indexer.index_vault().await.unwrap())
                        },
                    );
            },
        );
    }
    group.finish();
}

// Benchmark BLAKE3 hashing performance
fn benchmark_blake3_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_hashing");
    
    // Test with different content sizes
    let content_sizes = [
        ("small", 1024),      // 1KB
        ("medium", 10240),    // 10KB  
        ("large", 102400),    // 100KB
        ("xlarge", 1048576),  // 1MB
    ];
    
    for (name, size) in content_sizes.iter() {
        let content = "a".repeat(*size);
        group.bench_function(BenchmarkId::new("hash_content", name), |b| {
            b.iter(|| {
                let mut hasher = blake3::Hasher::new();
                hasher.update(content.as_bytes());
                black_box(hasher.finalize().to_hex().to_string())
            });
        });
    }
    group.finish();
}

// Benchmark Obsidian integration performance
fn benchmark_obsidian_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("obsidian_integration");
    group.measurement_time(Duration::from_secs(5));
    
    group.bench_function("create_ai_response", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter_setup(
                || setup_obsidian_vault(),
                |temp_dir| async move {
                    let vault_path = temp_dir.path().to_path_buf();
                    let obsidian = ObsidianVault::new(vault_path).await.unwrap();
                    
                    let query = "What are the key concepts in quantum computing?";
                    let response = "Test AI response content for benchmarking performance...";
                    
                    black_box(
                        obsidian
                            .create_ai_response(query, response, "hermes-3-8b", 0.85)
                            .await
                            .unwrap()
                    )
                },
            );
    });
    
    group.bench_function("update_daily_note", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter_setup(
                || setup_obsidian_vault(),
                |temp_dir| async move {
                    let vault_path = temp_dir.path().to_path_buf();
                    let obsidian = ObsidianVault::new(vault_path).await.unwrap();
                    
                    let summary = "Benchmark test interaction summary";
                    
                    black_box(
                        obsidian
                            .add_daily_interaction(summary)
                            .await
                            .unwrap()
                    )
                },
            );
    });
    
    group.finish();
}

// Benchmark configuration loading
fn benchmark_config_loading(c: &mut Criterion) {
    c.bench_function("load_config", |b| {
        b.iter(|| {
            black_box(Settings::load_from_file("config/config.toml").unwrap())
        });
    });
}

// Benchmark database operations
fn benchmark_database_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_operations");
    
    group.bench_function("database_init", |b| {
        b.iter_setup(
            || TempDir::new().unwrap(),
            |temp_dir| {
                let vault_path = temp_dir.path().to_path_buf();
                let db_path = vault_path.join("test.db");
                black_box(VaultIndexer::new(db_path, vault_path).unwrap())
            },
        );
    });
    
    group.finish();
}

// Helper function to create test vault with specified number of files
fn setup_test_vault(file_count: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();
    
    // Create sample markdown files
    for i in 0..file_count {
        let content = format!(
            r#"# Test Document {i}

This is test document number {i} for performance benchmarking.

## Content
- Point 1 for document {i}
- Point 2 for document {i}
- Point 3 for document {i}

## Tags
#test #benchmark #document-{i}

## Links
[[related-doc-{next}]]

Some longer content to make the file more realistic for testing purposes.
This includes multiple paragraphs and various markdown elements.

The content should be substantial enough to provide meaningful benchmark data
while still being generated quickly for testing purposes.
"#,
            i = i,
            next = (i + 1) % file_count
        );
        
        let file_path = vault_path.join(format!("test-doc-{:03}.md", i));
        fs::write(file_path, content).unwrap();
    }
    
    temp_dir
}

// Helper function to create Obsidian test vault
fn setup_obsidian_vault() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    temp_dir
}

criterion_group!(
    benches,
    benchmark_vault_indexing,
    benchmark_blake3_hashing,
    benchmark_obsidian_integration,
    benchmark_config_loading,
    benchmark_database_operations
);
criterion_main!(benches);
