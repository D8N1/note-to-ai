# Feature Flags Guide - Intelligence Compression Engine

This document provides comprehensive guidance on the feature flag system used in the Intelligence Compression Engine.

## Quick Start

```bash
# Minimal build - core compression engine only
cargo run

# AI-enabled build - includes local ML inference
cargo run --features ai-models

# Full development build - all features enabled
cargo run --features all-features

# Production analytics build
cargo run --features "ai-models,analytics"
```

## Available Feature Flags

### Core Features (Production Ready)

#### `compression-engine` (default)
- **Purpose**: 8/32/64 bit cognitive under-clocking system
- **Components**: Core compression algorithms, decision point limiting
- **Binary Impact**: +0MB (baseline)
- **Compile Time**: ~30 seconds
- **Dependencies**: Minimal - only essential crates
- **Use Cases**: All deployments, MVP demos, production core

#### `ai-models`
- **Purpose**: Local AI/ML inference with Candle framework
- **Components**: Transformer models, tokenization, HuggingFace Hub integration
- **Binary Impact**: +130MB 
- **Compile Time**: +2 minutes
- **Dependencies**: candle-core, candle-nn, candle-transformers, tokenizers, hf-hub
- **Use Cases**: Semantic search, content analysis, AI-assisted compression
- **Platform Notes**: Requires substantial RAM (2GB+ recommended)

#### `analytics`  
- **Purpose**: Advanced analytics with DuckDB columnar database
- **Components**: OLAP queries, data warehouse capabilities, Parquet support
- **Binary Impact**: +50MB
- **Compile Time**: +30 seconds
- **Dependencies**: duckdb (with bundled SQLite compatibility)
- **Use Cases**: Data analysis, reporting, business intelligence
- **Note**: Conflicts with rusqlite - use feature flags to separate

### ZK Proving Backends (Experimental)

#### `arkworks-backend`
- **Purpose**: Full-featured ZK-SNARK proving system
- **Components**: BN254 elliptic curve, Groth16 proving system, finite field arithmetic
- **Binary Impact**: +70MB
- **Compile Time**: +3 minutes
- **Dependencies**: Minimal arkworks stack (reduced from full suite)
- **Use Cases**: Complex ZK proofs, cryptographic research, privacy features
- **Security Note**: Uses unmaintained `derivative` crate - monitor for alternatives

#### `barretenberg-backend` (Placeholder)
- **Purpose**: Mobile-optimized ZK proving (future implementation)  
- **Target Impact**: <20MB binary overhead
- **Status**: Waiting for Aztec Barretenberg Rust bindings publication
- **Use Cases**: Mobile deployment, WASM targets, production ZK

#### `hybrid-backend`
- **Purpose**: Enables both arkworks and barretenberg backends
- **Use Cases**: Backend migration, A/B testing, development
- **Dependencies**: Combines both backend feature sets
- **Binary Impact**: Sum of both backends

### Convenience Feature Sets

#### `full-ai`
- **Combination**: `["ai-models", "analytics"]`
- **Purpose**: Complete AI stack for data-heavy applications
- **Total Impact**: +180MB binary, +2.5min compile time

#### `all-features`  
- **Combination**: `["compression-engine", "ai-models", "analytics", "arkworks-backend"]`
- **Purpose**: Everything enabled for development and testing
- **Total Impact**: +250MB binary, +5min compile time
- **Warning**: Large resource requirements

### Development Features

#### `migration-testing`
- **Purpose**: Database migration testing utilities
- **Impact**: Minimal
- **Use Cases**: CI/CD, database schema updates

#### `performance-benchmarks`
- **Purpose**: Performance benchmarking with Criterion
- **Impact**: +10MB binary
- **Use Cases**: Performance regression testing

## Build Matrix & Performance

| Feature Set | Binary Size | Compile Time | RAM Usage | Use Case |
|-------------|-------------|--------------|-----------|----------|
| Default | 20MB | 30s | 50MB | MVP, CLI tools |
| ai-models | 150MB | 2.5min | 2GB | AI inference |
| analytics | 70MB | 1min | 200MB | Data analysis |
| arkworks-backend | 90MB | 3.5min | 100MB | ZK research |
| full-ai | 200MB | 3min | 2.2GB | Full AI stack |
| all-features | 300MB | 5min | 2.5GB | Development |

## Platform Considerations

### macOS (M1/M2)
- All features supported
- Excellent performance for AI models (Metal acceleration)
- Native compilation for all backends

### Linux x86_64
- All features supported  
- CPU-only AI inference (slower)
- Consider GPU-enabled builds for production AI

### Windows  
- Core features supported
- AI models require additional MSVC setup
- ZK backends fully supported

### Mobile/WASM (Future)
- `barretenberg-backend` designed for mobile
- Core compression engine WASM-compatible
- AI models too large for mobile currently

## Security Considerations

### Current Vulnerabilities (Post-Update)
- ✅ SQLx vulnerability fixed (0.7.4 → 0.8.6)
- ✅ Slab vulnerability fixed (0.4.10 → 0.4.11)
- ✅ Tracing-subscriber fixed (0.3.19 → 0.3.20)
- ⚠️ Ring 0.16.20 in libp2p (no fix available yet)
- ⚠️ RSA timing attack in SQLx-mysql (no fix available)
- ⚠️ Tokio broadcast channel unsoundness

### Unmaintained Dependencies
- `derivative` 2.2.0 (arkworks only) - seeking replacement
- `fxhash` 0.2.1 (automerge) - monitor for updates
- `paste` 1.0.15 (multiple) - widespread but unmaintained

### Recommendations
- Use `cargo audit` regularly
- Enable only needed features in production
- Monitor security advisories for core dependencies

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Feature Matrix Tests
jobs:
  test-core:
    run: cargo test --features compression-engine
  test-ai:
    run: cargo test --features ai-models
  test-full:
    run: cargo test --features all-features
```

### Docker Multi-Stage
```dockerfile
# Core stage - minimal dependencies
FROM rust:alpine AS core
COPY . .
RUN cargo build --release --features compression-engine

# AI stage - full ML stack  
FROM rust:latest AS ai
COPY . .
RUN cargo build --release --features full-ai
```

## Migration Guide

### From 0.1.0 to Current
- Replace `dotenv` with `dotenvy` in your code
- Replace `yaml-rust` with `serde_yaml`
- Update SQLx usage for 0.8.x compatibility
- Review arkworks usage - some crates removed for efficiency

### Dependency Conflicts
- **SQLite**: rusqlite temporarily disabled due to libsqlite3-sys version conflicts with SQLx 0.8
- **Resolution**: Use feature flags to separate rusqlite and SQLx usage
- **Timeline**: Waiting for compatible versions or implementing feature separation

## Performance Tuning

### Compile Time Optimization
```bash
# Use release mode for benchmarking
cargo build --release --features compression-engine

# Parallel compilation
export CARGO_BUILD_JOBS=$(nproc)

# Link-time optimization
export RUSTFLAGS="-C lto=fat"
```

### Runtime Optimization
- Enable only needed features
- Use `compression-engine` only for minimal deployments
- Consider `analytics` feature for data-heavy workloads
- Profile memory usage with AI models enabled

## Troubleshooting

### Common Issues

**Compilation Fails with SQLite Errors**
```bash
# Temporarily remove rusqlite from Cargo.toml
# Or use feature flags to separate storage backends
```

**Large Binary Size**
```bash
# Check what features are enabled
cargo tree --format "{p} {f}"

# Build with minimal features
cargo build --no-default-features --features compression-engine
```

**Slow Compilation**
```bash
# Use cargo cache
export CARGO_TARGET_DIR=/tmp/cargo-target

# Enable incremental compilation  
export RUSTC_WRAPPER=sccache
```

### Feature Flag Debugging
```bash
# Check enabled features
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "note-to-ai") | .features'

# Verify feature dependencies
cargo tree --features all-features --format "{p} {f}"
```

## Contributing

When adding new features:

1. **Update this guide** with feature impact metrics
2. **Add CI tests** for the new feature combination
3. **Document security implications** of new dependencies
4. **Provide migration guide** for breaking changes
5. **Test cross-platform compatibility**

### Feature Flag Best Practices

- Use descriptive feature names (`ai-models` not `ml`)
- Group related functionality (`full-ai` convenience flag)
- Document performance impact quantitatively
- Prefer optional dependencies over conditional compilation
- Test feature combinations in CI

## Support

For questions about feature flags:
- Check this documentation first
- Review `Cargo.toml` feature definitions
- Run `cargo metadata` to inspect current configuration
- Open issues for feature requests or problems

Last updated: September 17, 2025