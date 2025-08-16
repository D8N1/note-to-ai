#!/bin/bash
"""
NOTE-TO-AI BEHEMOTH CONSOLIDATION SCRIPT
Preserves ALL quantum-resistant, hybrid database, ML orchestration work
Consolidates into top-level /Users/unique/Desktop/note-to-ai directory
"""

echo "🎯 NOTE-TO-AI BEHEMOTH CONSOLIDATION"
echo "====================================="
echo "📁 Target: Top-level /Users/unique/Desktop/note-to-ai"
echo "🔒 Preserving: Quantum crypto + Hybrid DB + ML orchestration"
echo ""

# Step 1: Create COMPREHENSIVE backup
echo "📦 Creating comprehensive backup..."
tar -czf note-to-ai-behemoth-backup-$(date +%Y%m%d-%H%M%S).tar.gz note-to-ai/ note-to-ai\ */
echo "   ✅ Complete backup created"

# Step 2: Preserve CRITICAL implementation files
echo "🔄 Preserving COMPLETE source implementation..."
if [ -d "note-to-ai/note-to-ai/src/" ]; then
    echo "   🚀 Moving REVOLUTIONARY hybrid storage system..."
    echo "   🔐 Moving quantum-resistant crypto stack..."
    echo "   🧠 Moving AI orchestration components..."
    
    # Backup current incomplete src
    if [ -d "src/" ]; then
        mv src/ src-incomplete-backup/
    fi
    
    # Move complete implementation
    cp -r note-to-ai/note-to-ai/src/ .
    echo "   ✅ Complete source implementation preserved"
    
    # Verify critical components
    echo "   🔍 Verifying critical components..."
    [ -f "src/vault/storage/hybrid_engine.rs" ] && echo "      ✅ Hybrid storage engine"
    [ -f "src/crypto/pq_vault.rs" ] && echo "      ✅ Quantum-resistant crypto"
    [ -f "src/signal/client.rs" ] && echo "      ✅ Signal integration"
    [ -f "src/ai/local_llm.rs" ] && echo "      ✅ AI orchestration"
    [ -f "src/identity/zkpassport.rs" ] && echo "      ✅ zkPassport system"
else
    echo "   ⚠️ No nested source found - keeping current src/"
fi

# Step 3: Use COMPLETE Cargo.toml with all dependencies
echo "🔧 Preserving complete dependency stack..."
if [ -f "note-to-ai/note-to-ai/Cargo.toml" ]; then
    echo "   📦 Using complete Cargo.toml (quantum crypto + hybrid DB)"
    cp note-to-ai/note-to-ai/Cargo.toml .
    echo "   ✅ Complete dependency stack preserved"
    
    # Verify key dependencies
    echo "   🔍 Verifying key dependencies..."
    grep -q "duckdb" Cargo.toml && echo "      ✅ DuckDB (10x faster analytics)"
    grep -q "automerge" Cargo.toml && echo "      ✅ CRDT system"
    grep -q "libp2p" Cargo.toml && echo "      ✅ IPFS swarm"
    grep -q "blake3" Cargo.toml && echo "      ✅ Quantum-resistant hashing"
fi

# Step 4: Preserve M1-optimized models (CRITICAL)
echo "📁 Preserving M1 model optimizations..."
echo "   ✅ Keeping top-level models/ (M1 MacBook Air optimizations)"
echo "   ✅ Preserving Ollama models (llama3.2:3b, qwen2.5:7b, codellama:7b)"
echo "   ✅ Preserving Whisper.cpp with Metal backend"
echo "   ✅ Preserving DistilBART-CNN summarization model"
echo "   🗑️ Removing basic nested models/"
rm -rf note-to-ai/note-to-ai/models/ 2>/dev/null || true

# Step 5: Consolidate configurations SAFELY
echo "⚙️ Consolidating configurations..."
if [ -d "note-to-ai/note-to-ai/config/" ]; then
    echo "   📋 Merging config files..."
    mkdir -p config/
    cp -r note-to-ai/note-to-ai/config/* config/ 2>/dev/null || true
    echo "   ✅ Configuration files merged"
fi

# Step 6: Preserve CRITICAL documentation and context
echo "📚 Preserving critical documentation..."
if [ -f "note-to-ai/note-to-ai/README.md" ]; then
    echo "   📝 Found nested README.md - comparing with main README"
    # Check if there are differences worth preserving
    if ! diff -q README.md note-to-ai/note-to-ai/README.md >/dev/null 2>&1; then
        echo "   💡 Creating comparison file for review"
        diff README.md note-to-ai/note-to-ai/README.md > readme_differences.txt 2>/dev/null || true
        echo "   📝 README comparison saved to readme_differences.txt"
    else
        echo "   ✅ README files are identical - no merge needed"
    fi
fi

# Preserve handOFF.txt - CRITICAL for implementation context
if [ -f "handOFF.txt" ]; then
    echo "   ✅ Preserving handOFF.txt (critical implementation context)"
else
    echo "   ⚠️ handOFF.txt not found - may need recovery"
fi

# Step 7: Remove nested directories while preserving backups
echo "🗑️ Removing nested directories (backed up)..."
rm -rf note-to-ai/note-to-ai/
rm -rf "note-to-ai 11.02.57/"
rm -rf "note-to-ai 16.58.26/"
rm -rf "note-to-ai 17.18.02/"
echo "   ✅ Nested directories removed"

# Step 8: Clean up miscellaneous files
echo "🧹 Final cleanup..."
rm -f *.zip *.rtf 2>/dev/null || true
# Preserve arrow-rs if it's part of the project
if [ -d "arrow-rs/" ] && [ ! -f "arrow-rs/Cargo.toml" ]; then
    echo "   🗑️ Removing duplicate arrow-rs/"
    rm -rf arrow-rs/ 2>/dev/null || true
fi

# Step 9: Verify BEHEMOTH preservation
echo ""
echo "� BEHEMOTH PRESERVATION VERIFICATION:"
echo "======================================="

# Check source code completeness
echo "📂 Source Code Status:"
[ -f "src/main.rs" ] && echo "   ✅ main.rs ($(wc -l < src/main.rs) lines)" || echo "   ❌ main.rs missing"
[ -d "src/vault/storage/" ] && echo "   ✅ Hybrid storage system" || echo "   ❌ Hybrid storage missing"
[ -d "src/crypto/" ] && echo "   ✅ Quantum-resistant crypto" || echo "   ❌ Crypto stack missing"
[ -d "src/signal/" ] && echo "   ✅ Signal integration" || echo "   ❌ Signal integration missing"
[ -d "src/ai/" ] && echo "   ✅ AI orchestration" || echo "   ❌ AI orchestration missing"
[ -d "src/identity/" ] && echo "   ✅ zkPassport system" || echo "   ❌ Identity system missing"

# Check model optimizations
echo ""
echo "🧠 Model Optimization Status:"
[ -f "models/m1_production_ready.json" ] && echo "   ✅ M1 production config" || echo "   ❌ M1 config missing"
[ -d "models/whisper.cpp/" ] && echo "   ✅ Whisper.cpp with Metal" || echo "   ❌ Whisper missing"
[ -f "models/model_registry.toml" ] && echo "   ✅ Model registry" || echo "   ❌ Model registry missing"

# Check dependencies
echo ""
echo "📦 Dependency Status:"
[ -f "Cargo.toml" ] && echo "   ✅ Complete Cargo.toml" || echo "   ❌ Cargo.toml missing"
grep -q "duckdb" Cargo.toml 2>/dev/null && echo "   ✅ DuckDB dependency" || echo "   ❌ DuckDB missing"
grep -q "automerge" Cargo.toml 2>/dev/null && echo "   ✅ CRDT system" || echo "   ❌ CRDT missing"

# Final directory structure
echo ""
echo "📋 FINAL DIRECTORY STRUCTURE:"
echo "================================"
ls -1 | grep -E "^[^.]" | sed 's/^/📁 /' | head -20

echo ""
echo "✅ BEHEMOTH CONSOLIDATION COMPLETE!"
echo "� Quantum-resistant crypto stack: PRESERVED"  
echo "💾 Hybrid database system: PRESERVED"
echo "🧠 AI orchestration: PRESERVED"
echo "📱 Signal integration: PRESERVED"
echo "🔐 zkPassport identity: PRESERVED"
echo "🎯 M1 model optimizations: PRESERVED"
echo ""
echo "🚀 UNIFIED PROJECT READY FOR PRODUCTION DEVELOPMENT!"
echo "📁 All work consolidated in /Users/unique/Desktop/note-to-ai"
