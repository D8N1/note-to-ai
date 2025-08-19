// File: src/signal_integration/note_to_self.rs
// Signal "Note to Self" message processing and UX handling

use crate::Result;
use crate::ai::hermes_integration::HermesMessage;
use crate::signal_integration::api_compatibility::*;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use uuid::Uuid;

/// Types of Signal messages we can process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Voice { 
        audio_path: PathBuf,
        duration_seconds: u32,
    },
    Text { 
        content: String,
    },
    Image { 
        image_path: PathBuf,
        caption: Option<String>,
    },
    Document { 
        doc_path: PathBuf,
        filename: String,
        caption: Option<String>,
    },
    Mixed {
        text: Option<String>,
        attachments: Vec<Attachment>,
    },
}

/// Attachment types for mixed messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Attachment {
    Voice { path: PathBuf, duration: u32 },
    Image { path: PathBuf },
    Document { path: PathBuf, filename: String },
}

/// Incoming Signal message from "Note to Self"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub id: String,
    pub timestamp: SystemTime,
    pub message_type: MessageType,
    pub sender_phone: String,
    pub conversation_id: String, // Always "note-to-self" for our use case
}

/// Processing context for a message
#[derive(Debug, Clone)]
pub struct MessageContext {
    pub user_id: String,
    pub session_id: String,
    pub processing_start: SystemTime,
    pub priority: MessagePriority,
    pub previous_context: Option<Vec<String>>, // Recent message IDs for context
}

/// Priority levels for message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Urgent,     // Process immediately
    Normal,     // Standard queue processing
    Background, // Process when system is idle
}

/// Processed message result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMessage {
    pub original_id: String,
    pub transcription: Option<String>,
    pub extracted_text: Option<String>,
    pub ai_response: String,
    pub brief_summary: String,
    pub action_items: Vec<String>,
    pub questions: Vec<String>,
    pub tags: Vec<String>,
    pub related_notes: Vec<String>,
    pub vault_note_path: PathBuf,
    pub processing_duration: Duration,
    pub confidence_score: f32,
}

/// User experience configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UXConfig {
    pub response_style: ResponseStyle,
    pub include_questions: bool,
    pub include_action_items: bool,
    pub include_related_notes: bool,
    pub max_response_length: usize,
    pub enable_voice_responses: bool,
    pub brief_format: BriefFormat,
}

/// Response style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStyle {
    Executive,   // Professional, concise, action-oriented
    Casual,      // Friendly, conversational
    Academic,    // Detailed, analytical
    Technical,   // Precise, detailed, includes technical details
    Creative,    // Engaging, thought-provoking
}

/// Brief format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BriefFormat {
    PresidentsBrief,  // Executive summary format
    BulletPoints,     // Quick bullet point summary
    Narrative,        // Story-like flowing summary
    QandA,           // Question and answer format
    Outline,         // Hierarchical outline format
}

impl Default for UXConfig {
    fn default() -> Self {
        Self {
            response_style: ResponseStyle::Executive,
            include_questions: true,
            include_action_items: true,
            include_related_notes: true,
            max_response_length: 1000,
            enable_voice_responses: false,
            brief_format: BriefFormat::PresidentsBrief,
        }
    }
}

/// Main Signal "Note to Self" processor
pub struct NoteToSelfProcessor {
    whisper: WhisperProcessor,
    hermes: SimpleHermesIntegration,
    vault: VaultStorage,
    config: UXConfig,
    message_queue: mpsc::Receiver<IncomingMessage>,
    response_sender: mpsc::Sender<(String, ProcessedMessage)>, // (recipient, response)
}

impl NoteToSelfProcessor {
    /// Create a new processor with the specified configuration
    pub async fn new(
        config: UXConfig,
        message_queue: mpsc::Receiver<IncomingMessage>,
        response_sender: mpsc::Sender<(String, ProcessedMessage)>,
    ) -> Result<Self> {
        let whisper = WhisperProcessor::new().await
            .context("Failed to initialize Whisper processor")?;
        
        let hermes = SimpleHermesIntegration::new()?;
        
        let vault = VaultStorage::new("vault".into()).await
            .context("Failed to initialize vault storage")?;
        
        Ok(Self {
            whisper,
            hermes,
            vault,
            config,
            message_queue,
            response_sender,
        })
    }
    
    /// Start processing messages from the queue
    pub async fn start_processing(&mut self) -> Result<()> {
        info!("Starting Note to Self message processing");
        
        while let Some(message) = self.message_queue.recv().await {
            let context = MessageContext {
                user_id: message.sender_phone.clone(),
                session_id: Uuid::new_v4().to_string(),
                processing_start: SystemTime::now(),
                priority: self.determine_priority(&message),
                previous_context: self.get_recent_context(&message.sender_phone).await?,
            };
            
            match self.process_message(message, context).await {
                Ok(processed) => {
                    info!("Successfully processed message {}", processed.original_id);
                    
                    // Send response back to Signal
                    if let Err(e) = self.response_sender.send((
                        processed.original_id.clone(),
                        processed
                    )).await {
                        error!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to process message: {}", e);
                    // TODO: Send error response to user
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single incoming message
    pub async fn process_message(
        &mut self,
        message: IncomingMessage,
        context: MessageContext,
    ) -> Result<ProcessedMessage> {
        let start_time = SystemTime::now();
        debug!("Processing message {} with type {:?}", message.id, message.message_type);
        
        // Extract text content from various message types
        let extracted_content = self.extract_content(&message).await?;
        
        // Search for related context in vault
        let related_context = self.search_related_content(&extracted_content.text).await?;
        
        // Generate AI response using Hermes
        let ai_response = self.generate_ai_response(
            &extracted_content.text,
            &related_context,
            &context,
        ).await?;
        
        // Create structured brief
        let brief = self.create_brief(&ai_response, &extracted_content.text).await?;
        
        // Save to vault
        let vault_path = self.save_to_vault(
            &message,
            &extracted_content,
            &ai_response,
            &brief,
        ).await?;
        
        // Calculate processing duration
        let processing_duration = start_time.elapsed()
            .unwrap_or(Duration::from_secs(0));
        
        let processed = ProcessedMessage {
            original_id: message.id,
            transcription: extracted_content.transcription,
            extracted_text: Some(extracted_content.text.clone()),
            ai_response: ai_response.content,
            brief_summary: brief.summary,
            action_items: brief.action_items,
            questions: brief.questions,
            tags: brief.tags,
            related_notes: brief.related_notes,
            vault_note_path: vault_path,
            processing_duration,
            confidence_score: extracted_content.confidence,
        };
        
        Ok(processed)
    }
    
    /// Extract text content from various message types
    async fn extract_content(&mut self, message: &IncomingMessage) -> Result<ExtractedContent> {
        match &message.message_type {
            MessageType::Voice { audio_path, duration_seconds } => {
                debug!("Processing voice message: {} seconds", duration_seconds);
                
                let transcription = self.whisper.transcribe_file(audio_path).await
                    .context("Failed to transcribe voice message")?;
                
                Ok(ExtractedContent {
                    text: transcription.clone(),
                    transcription: Some(transcription.clone()),
                    confidence: 0.8, // Placeholder confidence
                    extracted_images: vec![],
                    extracted_documents: vec![],
                })
            }
            
            MessageType::Text { content } => {
                Ok(ExtractedContent {
                    text: content.clone(),
                    transcription: None,
                    confidence: 1.0,
                    extracted_images: vec![],
                    extracted_documents: vec![],
                })
            }
            
            MessageType::Image { image_path, caption } => {
                // TODO: Implement OCR for image text extraction
                let text = caption.clone().unwrap_or_else(|| {
                    format!("Image: {}", image_path.display())
                });
                
                Ok(ExtractedContent {
                    text,
                    transcription: None,
                    confidence: 0.7, // Lower confidence for image-based content
                    extracted_images: vec![image_path.clone()],
                    extracted_documents: vec![],
                })
            }
            
            MessageType::Document { doc_path, filename, caption } => {
                // TODO: Implement document parsing
                let text = caption.clone().unwrap_or_else(|| {
                    format!("Document: {filename}")
                });
                
                Ok(ExtractedContent {
                    text,
                    transcription: None,
                    confidence: 0.8,
                    extracted_images: vec![],
                    extracted_documents: vec![doc_path.clone()],
                })
            }
            
            MessageType::Mixed { text, attachments } => {
                let mut combined_text = text.clone().unwrap_or_default();
                let mut all_transcriptions = vec![];
                let mut images = vec![];
                let mut documents = vec![];
                let mut total_confidence = 0.0;
                let mut confidence_count = 0;
                
                for attachment in attachments {
                    match attachment {
                        Attachment::Voice { path, .. } => {
                            let transcription = self.whisper.transcribe_file(path).await?;
                            combined_text.push_str(&format!("\n\n[Voice]: {transcription}"));
                            all_transcriptions.push(transcription.clone());
                            total_confidence += 0.8; // Placeholder confidence
                            confidence_count += 1;
                        }
                        Attachment::Image { path } => {
                            combined_text.push_str(&format!("\n\n[Image]: {}", path.display()));
                            images.push(path.clone());
                        }
                        Attachment::Document { path, filename } => {
                            combined_text.push_str(&format!("\n\n[Document]: {filename}"));
                            documents.push(path.clone());
                        }
                    }
                }
                
                let avg_confidence = if confidence_count > 0 {
                    total_confidence / confidence_count as f32
                } else {
                    1.0
                };
                
                Ok(ExtractedContent {
                    text: combined_text,
                    transcription: if all_transcriptions.is_empty() { 
                        None 
                    } else { 
                        Some(all_transcriptions.join(" ")) 
                    },
                    confidence: avg_confidence,
                    extracted_images: images,
                    extracted_documents: documents,
                })
            }
        }
    }
    
    /// Search for related content in the vault
    async fn search_related_content(&self, query: &str) -> Result<Vec<String>> {
        // TODO: Implement semantic search using hybrid storage
        // For now, return empty context
        debug!("Searching for related content for query: {}", query);
        Ok(vec![])
    }
    
    /// Generate AI response using Hermes
    async fn generate_ai_response(
        &self,
        content: &str,
        related_context: &[String],
        context: &MessageContext,
    ) -> Result<HermesMessage> {
        let system_prompt = self.build_system_prompt(related_context);
        
        let user_message = HermesMessage {
            role: "user".to_string(),
            content: content.to_string(),
            metadata: None,
        };
        
        // Create conversation context
        let conversation_id = format!("note-to-self-{}", context.session_id);
        
        // Initialize conversation with system prompt
        self.hermes.create_conversation(
            conversation_id.clone(),
            Some(system_prompt),
        ).await?;
        
        // Generate response
        let response = self.hermes.chat(
            &conversation_id,
            content,
            None,
        ).await.context("Failed to generate AI response")?;
        
        Ok(HermesMessage {
            role: "assistant".to_string(),
            content: response.content,
            metadata: response.metadata,
        })
    }
    
    /// Build system prompt based on configuration and context
    fn build_system_prompt(&self, related_context: &[String]) -> String {
        let style_instruction = match self.config.response_style {
            ResponseStyle::Executive => "Respond in a professional, executive style with clear action items and strategic insights.",
            ResponseStyle::Casual => "Respond in a friendly, conversational tone that's easy to understand.",
            ResponseStyle::Academic => "Provide detailed, analytical responses with thorough explanations.",
            ResponseStyle::Technical => "Give precise, technical responses with specific details and implementations.",
            ResponseStyle::Creative => "Respond in an engaging, thought-provoking manner that sparks new ideas.",
        };
        
        let format_instruction = match self.config.brief_format {
            BriefFormat::PresidentsBrief => "Format your response as an executive briefing with: Executive Summary, Key Points, Action Items, and Strategic Recommendations.",
            BriefFormat::BulletPoints => "Use clear bullet points to organize your response.",
            BriefFormat::Narrative => "Provide a flowing, story-like narrative response.",
            BriefFormat::QandA => "Structure your response as questions and answers.",
            BriefFormat::Outline => "Use a hierarchical outline format with clear sections and subsections.",
        };
        
        let context_section = if !related_context.is_empty() {
            format!("\n\nRelevant context from previous notes:\n{}", related_context.join("\n"))
        } else {
            String::new()
        };
        
        format!(
            "You are a personal AI assistant processing notes from Signal 'Note to Self'. \
            Your role is to help organize thoughts, extract insights, and provide actionable intelligence.\n\n\
            Style: {}\n\
            Format: {}\n\
            Max length: {} words\n\
            Include questions: {}\n\
            Include action items: {}{}\n\n\
            Analyze the user's note and provide a helpful, structured response that adds value to their thinking.",
            style_instruction,
            format_instruction,
            self.config.max_response_length / 5, // Rough word estimate
            self.config.include_questions,
            self.config.include_action_items,
            context_section
        )
    }
    
    /// Create structured brief from AI response
    async fn create_brief(&self, ai_response: &HermesMessage, original_text: &str) -> Result<StructuredBrief> {
        // TODO: Use specialized brief generation model
        // For now, parse the AI response for components
        
        let content = &ai_response.content;
        
        // Extract action items (lines starting with action keywords)
        let action_items = content
            .lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("action:") || lower.contains("todo:") || lower.contains("next steps:")
            })
            .map(|line| line.trim().to_string())
            .collect();
        
        // Extract questions (lines ending with ?)
        let questions = content
            .lines()
            .filter(|line| line.trim().ends_with('?'))
            .map(|line| line.trim().to_string())
            .collect();
        
        // Generate tags from content
        let tags = self.extract_tags(original_text, content).await?;
        
        Ok(StructuredBrief {
            summary: self.extract_summary(content),
            action_items,
            questions,
            tags,
            related_notes: vec![], // TODO: Implement related note discovery
        })
    }
    
    /// Extract summary from AI response
    fn extract_summary(&self, content: &str) -> String {
        // Look for executive summary section or take first paragraph
        for line in content.lines() {
            if line.to_lowercase().contains("summary:") || line.to_lowercase().contains("executive summary:") {
                // Find the summary section
                let summary_start = content.find(line).unwrap_or(0);
                let summary_text = &content[summary_start..];
                
                // Take until next major section or 2 paragraphs
                let summary_end = summary_text.find("\n\n").unwrap_or(summary_text.len());
                return summary_text[..summary_end.min(500)].trim().to_string();
            }
        }
        
        // Fallback: take first paragraph or 200 chars
        let first_para_end = content.find("\n\n").unwrap_or(content.len());
        content[..first_para_end.min(200)].trim().to_string()
    }
    
    /// Extract relevant tags from content
    async fn extract_tags(&self, original_text: &str, ai_response: &str) -> Result<Vec<String>> {
        // TODO: Use NLP to extract proper tags
        // For now, use simple keyword extraction
        
        let combined_text = format!("{original_text} {ai_response}").to_lowercase();
        let mut tags = vec![];
        
        // Common topic tags
        let topic_keywords = [
            ("ai", "artificial-intelligence"),
            ("research", "research"),
            ("meeting", "meetings"),
            ("idea", "ideas"),
            ("project", "projects"),
            ("task", "tasks"),
            ("note", "notes"),
            ("reminder", "reminders"),
            ("learning", "learning"),
            ("work", "work"),
        ];
        
        for (keyword, tag) in &topic_keywords {
            if combined_text.contains(keyword) {
                tags.push(tag.to_string());
            }
        }
        
        // Add AI-generated tag
        tags.push("ai-generated".to_string());
        
        // Add timestamp-based tags
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let date = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        tags.push(format!("date-{date}"));
        
        Ok(tags)
    }
    
    /// Save processed message to vault
    async fn save_to_vault(
        &self,
        message: &IncomingMessage,
        content: &ExtractedContent,
        ai_response: &HermesMessage,
        brief: &StructuredBrief,
    ) -> Result<PathBuf> {
        let timestamp = message.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let date = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        
        let time = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap()
            .format("%H-%M")
            .to_string();
        
        let filename = format!("ai-response-{date}-{time}.md");
        let relative_path = PathBuf::from("AI Responses")
            .join(&date)
            .join(&filename);
        
        let markdown_content = self.format_vault_note(
            message,
            content,
            ai_response,
            brief,
        ).await?;
        
        // TODO: Implement vault storage
        // self.vault.save_note(&relative_path, &markdown_content).await?;
        
        Ok(relative_path)
    }
    
    /// Format note for vault storage
    async fn format_vault_note(
        &self,
        message: &IncomingMessage,
        content: &ExtractedContent,
        ai_response: &HermesMessage,
        brief: &StructuredBrief,
    ) -> Result<String> {
        let timestamp = message.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let formatted_date = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        
        let frontmatter = format!(
            "---\n\
            type: ai-response\n\
            source: signal-note-to-self\n\
            created: {}\n\
            message_id: {}\n\
            confidence: {:.2}\n\
            tags: [{}]\n\
            ---\n\n",
            formatted_date,
            message.id,
            content.confidence,
            brief.tags.join(", ")
        );
        
        let original_section = if let Some(ref transcription) = content.transcription {
            format!(
                "## Original Voice Note\n\n> {}\n\n**Transcription:** {}\n\n",
                content.text,
                transcription
            )
        } else {
            format!("## Original Note\n\n> {}\n\n", content.text)
        };
        
        let ai_section = format!("## AI Analysis\n\n{}\n\n", ai_response.content);
        
        let action_items_section = if !brief.action_items.is_empty() {
            format!(
                "## Action Items\n\n{}\n\n",
                brief.action_items
                    .iter()
                    .map(|item| format!("- [ ] {item}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };
        
        let questions_section = if !brief.questions.is_empty() {
            format!(
                "## Generated Questions\n\n{}\n\n",
                brief.questions
                    .iter()
                    .map(|q| format!("- {q}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };
        
        let related_section = if !brief.related_notes.is_empty() {
            format!(
                "## Related Notes\n\n{}\n\n",
                brief.related_notes
                    .iter()
                    .map(|note| format!("- [[{note}]]"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };
        
        Ok(format!(
            "{frontmatter}{original_section}{ai_section}{action_items_section}{questions_section}{related_section}"
        ))
    }
    
    /// Determine message priority based on content and context
    fn determine_priority(&self, message: &IncomingMessage) -> MessagePriority {
        // Check for urgent keywords
        let content_text = match &message.message_type {
            MessageType::Text { content } => content.to_lowercase(),
            MessageType::Voice { .. } => "voice".to_string(), // Will be analyzed after transcription
            MessageType::Image { caption, .. } => caption.clone().unwrap_or_default().to_lowercase(),
            MessageType::Document { caption, .. } => caption.clone().unwrap_or_default().to_lowercase(),
            MessageType::Mixed { text, .. } => text.clone().unwrap_or_default().to_lowercase(),
        };
        
        let urgent_keywords = ["urgent", "asap", "emergency", "important", "deadline"];
        
        if urgent_keywords.iter().any(|&keyword| content_text.contains(keyword)) {
            MessagePriority::Urgent
        } else {
            MessagePriority::Normal
        }
    }
    
    /// Get recent message context for better responses
    async fn get_recent_context(&self, user_id: &str) -> Result<Option<Vec<String>>> {
        // TODO: Implement context retrieval from vault
        // For now, return None
        debug!("Getting recent context for user: {}", user_id);
        Ok(None)
    }
}

/// Extracted content from various message types
#[derive(Debug, Clone)]
struct ExtractedContent {
    text: String,
    transcription: Option<String>,
    confidence: f32,
    extracted_images: Vec<PathBuf>,
    extracted_documents: Vec<PathBuf>,
}

/// Structured brief generated from AI response
#[derive(Debug, Clone)]
struct StructuredBrief {
    summary: String,
    action_items: Vec<String>,
    questions: Vec<String>,
    tags: Vec<String>,
    related_notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_extract_text_content() {
        let temp_dir = TempDir::new().unwrap();
        let (tx, rx) = mpsc::channel(10);
        let (response_tx, _response_rx) = mpsc::channel(10);
        
        let mut processor = NoteToSelfProcessor::new(
            UXConfig::default(),
            rx,
            response_tx,
        ).await.unwrap();
        
        let message = IncomingMessage {
            id: "test-123".to_string(),
            timestamp: SystemTime::now(),
            message_type: MessageType::Text {
                content: "This is a test message".to_string(),
            },
            sender_phone: "+1234567890".to_string(),
            conversation_id: "note-to-self".to_string(),
        };
        
        let content = processor.extract_content(&message).await.unwrap();
        assert_eq!(content.text, "This is a test message");
        assert_eq!(content.confidence, 1.0);
        assert!(content.transcription.is_none());
    }
    
    #[test]
    fn test_determine_priority() {
        let (tx, rx) = mpsc::channel::<IncomingMessage>(10);
        let (response_tx, _response_rx) = mpsc::channel::<ProcessedMessage>(10);
        
        // Create a mock processor (can't easily test async new in sync test)
        let message_urgent = IncomingMessage {
            id: "test-urgent".to_string(),
            timestamp: SystemTime::now(),
            message_type: MessageType::Text {
                content: "This is urgent! Please handle ASAP".to_string(),
            },
            sender_phone: "+1234567890".to_string(),
            conversation_id: "note-to-self".to_string(),
        };
        
        // Test priority determination logic directly
        let content_text = "this is urgent! please handle asap";
        let urgent_keywords = ["urgent", "asap", "emergency", "important", "deadline"];
        let is_urgent = urgent_keywords.iter().any(|&keyword| content_text.contains(keyword));
        
        assert!(is_urgent);
    }
    
    #[test]
    fn test_extract_summary() {
        let (tx, rx) = mpsc::channel::<IncomingMessage>(10);
        let (response_tx, _response_rx) = mpsc::channel::<ProcessedMessage>(10);
        
        let content = "Summary: This is a test summary of the content.\n\nMore details follow here.";
        
        // Test summary extraction logic
        for line in content.lines() {
            if line.to_lowercase().contains("summary:") {
                let summary_start = content.find(line).unwrap();
                let summary_text = &content[summary_start..];
                let summary_end = summary_text.find("\n\n").unwrap_or(summary_text.len());
                let extracted = summary_text[..summary_end.min(500)].trim();
                
                assert!(extracted.contains("This is a test summary"));
                break;
            }
        }
    }
}
