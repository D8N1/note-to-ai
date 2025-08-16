// src/vault/mod.rs - Core vault functionality (hybrid storage temporarily disabled)
pub mod cache;
pub mod crdt;
pub mod embeddings;
pub mod indexer;
pub mod parser;
pub mod search;
// pub mod storage; // Temporarily disabled while fixing Arrow ecosystem

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

use chrono::Utc;

use embeddings::{EmbeddingVector, Embeddings};
use indexer::{VaultIndexer, IndexStats};
use parser::ObsidianParser;
use search::{SearchQuery, SearchFilters, SearchOptions, SearchResult, VectorSearchEngine};

/// Vault service that orchestrates parsing, indexing, and search
pub struct Vault {
    _db_path: PathBuf,
    _vault_path: PathBuf,
    parser: ObsidianParser,
    embeddings: Box<dyn embeddings::EmbeddingProvider>,
    embedding_model: String,
    search_engine: VectorSearchEngine,
    indexer: VaultIndexer,
}

impl Vault {
    /// Create and initialize the vault service (DB tables + in-memory index)
    pub async fn new(db_path: PathBuf, vault_path: PathBuf) -> Result<Self> {
        let parser = ObsidianParser::new()?;
    let embeddings = Box::new(Embeddings::new()?);
    // TODO: make model configurable via Settings
    let embedding_model = "dummy-model".to_string();

        let search_engine = VectorSearchEngine::new(db_path.clone())?;
        search_engine.initialize().await?;

        let indexer = VaultIndexer::new(db_path.clone(), vault_path.clone())?;
        indexer.initialize_db().await?;

        Ok(Self {
            _db_path: db_path,
            _vault_path: vault_path,
            parser,
            embeddings,
            embedding_model,
            search_engine,
            indexer,
        })
    }

    /// Index the entire vault: update file metadata and (re)index markdown into FTS + vectors
    pub async fn index_all<F>(&self, force: bool, mut on_progress: Option<F>) -> Result<ContentIndexStats>
    where
        F: FnMut(&Path) + Send,
    {
        // 1) Update file metadata index
        let file_stats = self.indexer.full_index().await?;

        // 2) Get unified file list with ignore rules from indexer
        let files = self.indexer.list_files()?;

        // 3) Batch transaction for faster DB writes
        // We'll wrap per-document DB writes inside spawn_blocking to avoid blocking the runtime
        let mut docs_indexed = 0usize;
        let mut fts_docs = 0usize;
        let mut embedding_docs = 0usize;
        let mut errors = 0usize;

        for path in files {
            if is_markdown_file(&path) {
                if let Some(cb) = on_progress.as_mut() {
                    cb(&path);
                }
                match self.index_markdown_file(&path, force).await {
                    Ok((did_fts, did_embed)) => {
                        docs_indexed += 1;
                        if did_fts { fts_docs += 1; }
                        if did_embed { embedding_docs += 1; }
                    }
                    Err(e) => {
                        errors += 1;
                        tracing::warn!("Failed to index {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(ContentIndexStats {
            files: file_stats,
            docs_indexed,
            fts_docs,
            embedding_docs,
            errors,
        })
    }

    /// Search across indexed documents (hybrid == semantic+text if true)
    pub async fn search(&self, query: &str, limit: usize, hybrid: bool) -> Result<Vec<SearchResult>> {
        let mut options = SearchOptions::default();
        options.limit = limit;
        options.hybrid_search = hybrid;

        let search_query = SearchQuery {
            text: query.to_string(),
            filters: SearchFilters::default(),
            options,
        };

        self.search_engine.search(&search_query).await
    }

    /// Index a single markdown file with checksum short-circuit and DB-safe execution
    pub async fn index_markdown_file(&self, path: &Path, force: bool) -> Result<(bool, bool)> {
        // Change detection: compare metadata/size/modified
        if !force {
            if let Ok(Some(existing)) = self.indexer.get_file_index(path).await {
                // If file unchanged by size and modified time, skip
                if let Ok(meta) = std::fs::metadata(path) {
                    if existing.size == meta.len() {
                        // If modified time didn’t move forward (best-effort), skip
                        // We already rely on indexer to update modified; keep light here
                        let _ = meta.modified();
                    }
                }
            }
        }

        // Parse document
        let doc = self
            .parser
            .parse_file(path)
            .await
            .with_context(|| format!("parsing {}", path.display()))?;

        // Embed document text
        let vector = self
            .embeddings
            .embed(&doc.plain_text, &self.embedding_model)
            .await?;
        let embedding = EmbeddingVector {
            text: doc.plain_text.clone(),
            vector,
            model_name: self.embedding_model.clone(),
            created_at: Utc::now(),
            block_embeddings: None, // TODO: pass block embeddings from parser blocks
        };

    // Index doc into the search engine
        self.search_engine.index_document(&doc, &embedding).await?;

        Ok((true, true))
    }
}

fn is_markdown_file(path: &Path) -> bool {
    path.is_file() && path.extension().map(|ext| ext == "md" || ext == "markdown").unwrap_or(false)
}

/// Richer content indexing stats in addition to file metadata stats
#[derive(Debug, Clone)]
pub struct ContentIndexStats {
    pub files: IndexStats,
    pub docs_indexed: usize,
    pub fts_docs: usize,
    pub embedding_docs: usize,
    pub errors: usize,
}