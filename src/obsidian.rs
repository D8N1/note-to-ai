// src/obsidian.rs - Obsidian vault integration for AI-generated content
use crate::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::info;

/// Configuration for Obsidian integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianConfig {
    /// Path to the Obsidian vault (same as our vault directory)
    pub vault_path: PathBuf,
    /// Template for AI response notes
    pub ai_response_template: String,
    /// Template for daily notes
    pub daily_note_template: String,
    /// Whether to auto-link related notes
    pub auto_link: bool,
    /// Tags to add to AI-generated content
    pub default_tags: Vec<String>,
}

impl Default for ObsidianConfig {
    fn default() -> Self {
        Self {
            vault_path: PathBuf::from("vault"),
            ai_response_template: "AI Responses/{{date}}/{{timestamp}} - {{query_summary}}".to_string(),
            daily_note_template: "Daily Notes/{{date}}".to_string(),
            auto_link: true,
            default_tags: vec![
                "#ai-generated".to_string(),
                "#note-to-ai".to_string(),
            ],
        }
    }
}

/// Represents an AI response to be saved in Obsidian format
#[derive(Debug, Clone)]
pub struct AIResponse {
    pub query: String,
    pub response: String,
    pub sources: Vec<String>,
    pub timestamp: DateTime<Local>,
    pub confidence: Option<f32>,
    pub model_used: String,
}

/// Obsidian vault manager for AI-generated content
pub struct ObsidianManager {
    config: ObsidianConfig,
}

impl ObsidianManager {
    pub fn new(config: ObsidianConfig) -> Self {
        Self { config }
    }

    /// Create a new AI response note in Obsidian format
    pub async fn save_ai_response(&self, response: AIResponse) -> Result<PathBuf> {
        let note_path = self.generate_response_note_path(&response)?;
        let content = self.format_ai_response(&response)?;
        
        // Ensure directory exists
        if let Some(parent) = note_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }
        
        // Write the note
        async_fs::write(&note_path, content).await?;
        
        info!("Saved AI response to: {}", note_path.display());
        Ok(note_path)
    }

    /// Update or create today's daily note with new AI interaction
    pub async fn append_to_daily_note(&self, interaction_summary: &str) -> Result<PathBuf> {
        let daily_note_path = self.generate_daily_note_path()?;
        
        let interaction_entry = format!(
            "\n## {} - AI Interaction\n{}\n",
            Local::now().format("%H:%M"),
            interaction_summary
        );
        
        // If daily note exists, append; otherwise create new
        if daily_note_path.exists() {
            let mut content = async_fs::read_to_string(&daily_note_path).await?;
            
            // Find a good place to insert (before tags section if it exists)
            if let Some(tags_pos) = content.rfind("\n---\n*Tags:") {
                content.insert_str(tags_pos, &interaction_entry);
            } else {
                content.push_str(&interaction_entry);
            }
            
            async_fs::write(&daily_note_path, content).await?;
        } else {
            let new_content = self.create_daily_note_with_interaction(&interaction_entry)?;
            
            // Ensure directory exists
            if let Some(parent) = daily_note_path.parent() {
                async_fs::create_dir_all(parent).await?;
            }
            
            async_fs::write(&daily_note_path, new_content).await?;
        }
        
        info!("Updated daily note: {}", daily_note_path.display());
        Ok(daily_note_path)
    }

    /// Generate Obsidian-style links to related notes
    pub async fn find_related_notes(&self, content: &str) -> Result<Vec<String>> {
        let mut related = Vec::new();
        
        // Simple keyword-based linking (can be enhanced with semantic search later)
        let vault_files = self.scan_vault_files().await?;
        
        for file_path in vault_files {
            if let Some(filename) = file_path.file_stem() {
                let filename_str = filename.to_string_lossy();
                
                // Check if any words from the filename appear in the content
                if content.to_lowercase().contains(&filename_str.to_lowercase()) {
                    let obsidian_link = format!("[[{filename_str}]]");
                    if !related.contains(&obsidian_link) {
                        related.push(obsidian_link);
                    }
                }
            }
        }
        
        Ok(related)
    }

    /// Format AI response in Obsidian markdown style
    fn format_ai_response(&self, response: &AIResponse) -> Result<String> {
        let mut content = String::new();
        
        // Title
        let query_summary = self.summarize_query(&response.query);
        content.push_str(&format!("# AI Response: {query_summary}\n\n"));
        
        // Metadata
        content.push_str("## Query Details\n");
        content.push_str(&format!("**Original Query:** {}\n", response.query));
        content.push_str(&format!("**Timestamp:** {}\n", response.timestamp.format("%Y-%m-%d %H:%M:%S")));
        content.push_str(&format!("**Model Used:** {}\n", response.model_used));
        
        if let Some(confidence) = response.confidence {
            content.push_str(&format!("**Confidence:** {:.1}%\n", confidence * 100.0));
        }
        content.push('\n');
        
        // Main response
        content.push_str("## AI Response\n");
        content.push_str(&response.response);
        content.push_str("\n\n");
        
        // Sources
        if !response.sources.is_empty() {
            content.push_str("## Sources\n");
            for source in &response.sources {
                content.push_str(&format!("- [[{source}]]\n"));
            }
            content.push('\n');
        }
        
        // Related notes (if auto-linking is enabled)
        if self.config.auto_link {
            if let Ok(related) = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(self.find_related_notes(&response.response))
            }) {
                if !related.is_empty() {
                    content.push_str("## Related Notes\n");
                    for link in related {
                        content.push_str(&format!("- {link}\n"));
                    }
                    content.push('\n');
                }
            }
        }
        
        // Tags
        content.push_str("---\n");
        let mut all_tags = self.config.default_tags.clone();
        all_tags.push(format!("#query-{}", Local::now().format("%Y-%m-%d")));
        content.push_str(&format!("*Tags: {}*\n", all_tags.join(" ")));
        content.push_str(&format!("*Generated: {}*\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        Ok(content)
    }

    /// Generate file path for AI response note
    fn generate_response_note_path(&self, response: &AIResponse) -> Result<PathBuf> {
        let date = response.timestamp.format("%Y-%m-%d").to_string();
        let timestamp = response.timestamp.format("%H%M%S").to_string();
        let query_summary = self.summarize_query(&response.query);
        
        let filename = format!("{timestamp} - {query_summary}.md");
        let relative_path = format!("AI Responses/{date}/{filename}");
        
        Ok(self.config.vault_path.join(relative_path))
    }

    /// Generate file path for daily note
    fn generate_daily_note_path(&self) -> Result<PathBuf> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let filename = format!("daily-notes-{date}.md");
        Ok(self.config.vault_path.join(filename))
    }

    /// Create a new daily note with an interaction
    fn create_daily_note_with_interaction(&self, interaction: &str) -> Result<String> {
        let date = Local::now().format("%B %d, %Y").to_string();
        let content = format!(
            "# Daily Notes - {}\n\n{}\n\n---\n*Tags: #daily-notes #note-to-ai*\n*Generated: {}*\n",
            date,
            interaction,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        Ok(content)
    }

    /// Summarize a query for filename/title purposes
    fn summarize_query(&self, query: &str) -> String {
        // Simple summarization: take first few words and clean for filename
        let words: Vec<&str> = query.split_whitespace().take(5).collect();
        let summary = words.join(" ");
        
        // Clean for filename
        summary
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
            .to_lowercase()
    }

    /// Scan vault for existing markdown files
    async fn scan_vault_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if !self.config.vault_path.exists() {
            return Ok(files);
        }
        
        // Use a simple iterative approach instead of recursion
        self.scan_directory_iterative(&self.config.vault_path, &mut files).await?;
        Ok(files)
    }

    /// Iteratively scan directories for markdown files
    async fn scan_directory_iterative(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = async_fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            } else if path.is_dir() {
                // Simple one-level recursion only to avoid complex async recursion
                let mut sub_entries = async_fs::read_dir(&path).await?;
                while let Some(sub_entry) = sub_entries.next_entry().await? {
                    let sub_path = sub_entry.path();
                    if sub_path.is_file() && sub_path.extension().is_some_and(|ext| ext == "md") {
                        files.push(sub_path);
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Helper function to create a demo AI response for testing
pub fn create_demo_response(query: &str, response: &str) -> AIResponse {
    AIResponse {
        query: query.to_string(),
        response: response.to_string(),
        sources: vec![
            "quantum-computing-notes.md".to_string(),
            "daily-notes-2025-08-08.md".to_string(),
        ],
        timestamp: Local::now(),
        confidence: Some(0.85),
        model_used: "Hermes 3 8B".to_string(),
    }
}
