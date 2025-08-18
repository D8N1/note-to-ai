// File: src/signal_integration/conversational_assistant.rs
// Conversational AI assistant for Signal "Note to Self" integration

use crate::Result;
use crate::signal_integration::api_compatibility::*;
use crate::signal_integration::note_to_self::{
    IncomingMessage, MessageType, ProcessedMessage, MessageContext, UXConfig, ResponseStyle, BriefFormat
};
use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

// Simple message type for compatibility
#[derive(Debug, Clone)]
pub struct HermesMessage {
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

// Simple AI response type
#[derive(Debug, Clone)]
pub struct AIResponse {
    pub content: String,
}

// Simple AI integration for Signal
pub struct SimpleHermesIntegration;

impl SimpleHermesIntegration {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub async fn create_conversation(&self, _conversation_id: String, _system_prompt: Option<String>) -> Result<()> {
        Ok(())
    }
    
    pub async fn chat(&self, _conversation_id: &str, message: &HermesMessage, _task_context: Option<()>) -> Result<AIResponse> {
        // Enhanced response with basic NLP patterns
        let content = message.content.to_lowercase();
        
        let response = if content.contains("strategic") || content.contains("business") {
            self.generate_strategic_response(&content).await?
        } else if content.contains("research") || content.contains("analyze") {
            self.generate_research_response(&content).await?
        } else if content.contains("meeting") || content.contains("prep") {
            self.generate_meeting_response(&content).await?
        } else if content.contains("decision") || content.contains("choose") {
            self.generate_decision_response(&content).await?
        } else if content.contains("task") || content.contains("todo") {
            self.generate_task_response(&content).await?
        } else {
            self.generate_general_response(&content).await?
        };
        
        Ok(AIResponse {
            content: response,
        })
    }
    
    async fn generate_strategic_response(&self, content: &str) -> Result<String> {
        // Analyze strategic content and provide structured response
        let key_themes = self.extract_business_themes(content);
        let mut response = String::from("📊 Strategic Analysis:\n\n");
        
        if key_themes.contains(&"competition".to_string()) {
            response.push_str("• Competitive landscape considerations\n");
        }
        if key_themes.contains(&"market".to_string()) {
            response.push_str("• Market opportunity assessment\n");
        }
        if key_themes.contains(&"growth".to_string()) {
            response.push_str("• Growth strategy implications\n");
        }
        
        response.push_str("\n💡 Recommendations:\n");
        response.push_str("• Monitor key metrics closely\n");
        response.push_str("• Consider stakeholder perspectives\n");
        response.push_str("• Plan for multiple scenarios\n");
        
        Ok(response)
    }
    
    async fn generate_research_response(&self, content: &str) -> Result<String> {
        let topics = self.extract_research_topics(content);
        let mut response = String::from("🔍 Research Plan:\n\n");
        
        for topic in &topics {
            response.push_str(&format!("• {}\n", topic));
        }
        
        response.push_str("\n📋 Next Steps:\n");
        response.push_str("• Gather primary sources\n");
        response.push_str("• Analyze market data\n");
        response.push_str("• Validate findings\n");
        
        Ok(response)
    }
    
    async fn generate_meeting_response(&self, content: &str) -> Result<String> {
        let mut response = String::from("📅 Meeting Preparation:\n\n");
        
        if content.contains("board") || content.contains("executive") {
            response.push_str("🎯 Executive Focus:\n");
            response.push_str("• Key metrics and KPIs\n");
            response.push_str("• Strategic initiatives\n");
            response.push_str("• Risk assessment\n\n");
        }
        
        response.push_str("📝 Action Items:\n");
        response.push_str("• Prepare agenda\n");
        response.push_str("• Review background materials\n");
        response.push_str("• Identify key decisions needed\n");
        
        Ok(response)
    }
    
    async fn generate_decision_response(&self, content: &str) -> Result<String> {
        let mut response = String::from("🤔 Decision Framework:\n\n");
        
        response.push_str("📊 Consider:\n");
        response.push_str("• Pros and cons analysis\n");
        response.push_str("• Risk vs. reward assessment\n");
        response.push_str("• Stakeholder impact\n");
        response.push_str("• Timeline constraints\n\n");
        
        response.push_str("✅ Decision Process:\n");
        response.push_str("• Gather all relevant data\n");
        response.push_str("• Consult key stakeholders\n");
        response.push_str("• Document rationale\n");
        
        Ok(response)
    }
    
    async fn generate_task_response(&self, content: &str) -> Result<String> {
        let tasks = self.extract_tasks(content);
        let mut response = String::from("✅ Task Management:\n\n");
        
        for task in &tasks {
            response.push_str(&format!("• {}\n", task));
        }
        
        response.push_str("\n🎯 Priority Assessment:\n");
        response.push_str("• Urgent & Important (Do First)\n");
        response.push_str("• Important (Schedule)\n");
        response.push_str("• Urgent (Delegate)\n");
        
        Ok(response)
    }
    
    async fn generate_general_response(&self, _content: &str) -> Result<String> {
        Ok("I'm here to help with strategic analysis, research, meeting prep, and decision support. How can I assist you today?".to_string())
    }
    
    fn extract_business_themes(&self, content: &str) -> Vec<String> {
        let mut themes = Vec::new();
        let keywords = [
            ("competition", "competitive"),
            ("market", "marketplace"),
            ("growth", "expansion"),
            ("revenue", "profit"),
            ("customer", "client"),
            ("product", "solution"),
            ("strategy", "strategic"),
        ];
        
        for (theme, keyword) in keywords.iter() {
            if content.contains(keyword) {
                themes.push(theme.to_string());
            }
        }
        
        themes
    }
    
    fn extract_research_topics(&self, content: &str) -> Vec<String> {
        let mut topics = Vec::new();
        
        // Simple pattern matching for research topics
        if content.contains("market") {
            topics.push("Market analysis".to_string());
        }
        if content.contains("competitor") {
            topics.push("Competitive intelligence".to_string());
        }
        if content.contains("customer") {
            topics.push("Customer research".to_string());
        }
        if content.contains("technology") {
            topics.push("Technology assessment".to_string());
        }
        
        if topics.is_empty() {
            topics.push("General research".to_string());
        }
        
        topics
    }
    
    fn extract_tasks(&self, content: &str) -> Vec<String> {
        let mut tasks = Vec::new();
        
        // Simple task extraction patterns
        if content.contains("call") || content.contains("phone") {
            tasks.push("Schedule phone call".to_string());
        }
        if content.contains("email") || content.contains("message") {
            tasks.push("Send follow-up email".to_string());
        }
        if content.contains("review") {
            tasks.push("Review documents".to_string());
        }
        if content.contains("prepare") || content.contains("prep") {
            tasks.push("Prepare materials".to_string());
        }
        
        if tasks.is_empty() {
            tasks.push("Follow up on discussion".to_string());
        }
        
        tasks
    }
}

/// Types of user intents detected from messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentType {
    StrategicUpdate,      // Business strategy, competitive intel, market updates
    ConfidentialRequest,  // Request for sensitive/private information
    Research,             // Request for research or analysis
    QuickQuestion,        // Simple question requiring brief response
    TaskManagement,       // Adding tasks, reminders, action items
    PersonalReflection,   // Personal thoughts, journaling, brainstorming
    MeetingPrep,          // Preparation for upcoming meetings
    DecisionSupport,      // Help with decision making
    StatusUpdate,         // Update on projects, initiatives
    Casual,               // General conversation, not requiring special handling
}

/// Conversation context and memory
#[derive(Debug, Clone)]
pub struct ConversationMemory {
    recent_messages: Vec<ProcessedMessage>,
    active_topics: HashMap<String, TopicContext>,
    user_preferences: UserProfile,
    session_start: SystemTime,
}

/// Context for topics being tracked across conversations
#[derive(Debug, Clone)]
pub struct TopicContext {
    topic: String,
    mentions: u32,
    last_mentioned: SystemTime,
    urgency_level: UrgencyLevel,
    related_action_items: Vec<String>,
    growing_urgency: bool,
}

/// User profile for personalized responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub preferred_response_style: ResponseStyle,
    pub timezone: String,
    pub work_hours: (u8, u8), // (start_hour, end_hour)
    pub executive_level: ExecutiveLevel,
    pub industry_context: Vec<String>,
    pub key_stakeholders: Vec<String>,
    pub recent_priorities: Vec<String>,
}

/// Executive level affects response sophistication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutiveLevel {
    IC,           // Individual contributor
    Manager,      // Team manager
    Director,     // Department director
    VP,           // Vice president
    CEO,          // Chief executive
    Board,        // Board member
}

/// Urgency levels for proactive insights
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Strategic insight generated proactively
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveInsight {
    pub insight: String,
    pub urgency: UrgencyLevel,
    pub topic: String,
    pub suggested_action: String,
    pub confidence: f32,
    pub generated_at: SystemTime,
}

/// Conversational response from the assistant
#[derive(Debug, Clone)]
pub struct ConversationalResponse {
    pub content: String,
    pub response_type: ResponseType,
    pub action_items: Vec<ActionItem>,
    pub follow_up_questions: Vec<String>,
    pub background_research: Option<ResearchTask>,
    pub requires_verification: bool,
    pub urgency: UrgencyLevel,
}

/// Types of responses the assistant can generate
#[derive(Debug, Clone)]
pub enum ResponseType {
    StrategicBrief,
    QuickAnswer,
    ActionableAdvice,
    ResearchSummary,
    VerificationPrompt,
    ProactiveInsight,
    StatusUpdate,
}

/// Action items extracted or generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub priority: Priority,
    pub due_date: Option<SystemTime>,
    pub assigned_to: Option<String>,
    pub context: String,
}

/// Priority levels for action items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Background research task
#[derive(Debug, Clone)]
pub struct ResearchTask {
    pub topic: String,
    pub research_type: ResearchType,
    pub estimated_duration: Duration,
    pub priority: Priority,
}

/// Types of research the assistant can perform
#[derive(Debug, Clone)]
pub enum ResearchType {
    CompetitiveAnalysis,
    MarketResearch,
    TechnologyTrends,
    IndustryNews,
    Regulatory,
    Financial,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            preferred_response_style: ResponseStyle::Executive,
            timezone: "UTC".to_string(),
            work_hours: (9, 17), // 9 AM to 5 PM
            executive_level: ExecutiveLevel::Manager,
            industry_context: vec!["Technology".to_string()],
            key_stakeholders: vec![],
            recent_priorities: vec![],
        }
    }
}

/// Main conversational assistant
pub struct ConversationalAssistant {
    pub hermes: SimpleHermesIntegration,
    pub memory: RwLock<ConversationMemory>,
    pub config: UXConfig,
    pub timing: ConversationTiming,
    pub interruption_manager: InterruptionManager,
}

/// Natural conversation timing simulation
pub struct ConversationTiming {
    pub typing_speed_wpm: u32,
    pub thinking_delay_ms: u64,
    pub voice_processing_delay_ms: u64,
}

/// Smart interruption management
pub struct InterruptionManager {
    pub last_user_message: Option<SystemTime>,
    pub daily_proactive_count: u32,
    pub quiet_hours: (u8, u8),
    pub max_daily_proactive: u32,
}

impl ConversationTiming {
    pub fn executive_assistant() -> Self {
        Self {
            typing_speed_wpm: 80,           // Fast, professional typing
            thinking_delay_ms: 800,         // Brief pause for analysis
            voice_processing_delay_ms: 1200, // Time to "process" voice note
        }
    }
    
    pub fn calculate_response_delay(&self, response_length: usize) -> Duration {
        // Calculate natural typing time
        let words = response_length / 5; // Average word length
        let typing_time_ms = (words as u64 * 60 * 1000) / self.typing_speed_wpm as u64;
        
        // Add thinking delay for complex responses
        let total_delay = if response_length > 500 {
            typing_time_ms + self.thinking_delay_ms
        } else {
            typing_time_ms.min(2000) // Cap at 2 seconds for short responses
        };
        
        Duration::from_millis(total_delay.max(500).min(5000)) // 0.5-5 second range
    }
    
    pub async fn simulate_voice_processing(&self, audio_duration: Duration) {
        // Simulate time to "listen" to voice note
        let processing_time = audio_duration.as_millis() as u64 / 4; // 4x real-time processing
        let delay = processing_time.max(self.voice_processing_delay_ms).min(3000);
        
        tokio::time::sleep(Duration::from_millis(delay)).await;
    }
}

impl InterruptionManager {
    pub fn new() -> Self {
        Self {
            last_user_message: None,
            daily_proactive_count: 0,
            quiet_hours: (22, 7), // 10 PM to 7 AM
            max_daily_proactive: 3,
        }
    }
    
    pub fn should_send_proactive_message(&self, insight: &ProactiveInsight) -> InterruptDecision {
        // Check if user is actively messaging
        if let Some(last_msg) = self.last_user_message {
            let time_since = SystemTime::now()
                .duration_since(last_msg)
                .unwrap_or(Duration::from_secs(0));
            
            if time_since < Duration::from_secs(5 * 60) {
                return InterruptDecision::WaitForConversationPause;
            }
        }
        
        // Check daily limit
        if self.daily_proactive_count >= self.max_daily_proactive {
            return match insight.urgency {
                UrgencyLevel::Critical | UrgencyLevel::High => InterruptDecision::SendImmediately,
                _ => InterruptDecision::QueueForTomorrow,
            };
        }
        
        // Check urgency
        match insight.urgency {
            UrgencyLevel::Critical => InterruptDecision::SendImmediately,
            UrgencyLevel::High => InterruptDecision::SendImmediately,
            UrgencyLevel::Medium => InterruptDecision::SendImmediately,
            UrgencyLevel::Low => InterruptDecision::BatchWithOthers,
        }
    }
    
    pub fn user_activity_detected(&mut self) {
        self.last_user_message = Some(SystemTime::now());
    }
}

#[derive(Debug, PartialEq)]
pub enum InterruptDecision {
    SendImmediately,
    WaitForConversationPause,
    QueueForMorning,
    QueueForTomorrow,
    BatchWithOthers,
    Skip,
}

impl ConversationalAssistant {
    /// Create new conversational assistant
    pub async fn new() -> anyhow::Result<Self> {
        let hermes = SimpleHermesIntegration::new().unwrap();
        
        let memory = RwLock::new(ConversationMemory {
            recent_messages: vec![],
            active_topics: HashMap::new(),
            user_preferences: UserProfile::default(),
            session_start: SystemTime::now(),
        });
        
        Ok(Self {
            hermes,
            memory,
            config: UXConfig::default(),
            timing: ConversationTiming::executive_assistant(),
            interruption_manager: InterruptionManager::new(),
        })
    }
    
    /// Process incoming Signal message with natural conversation flow
    pub async fn process_signal_message(
        &mut self,
        message: IncomingMessage,
    ) -> Result<ConversationalResponse> {
        debug!("Processing Signal message: {:?}", message.message_type);
        
        // Update user activity
        self.interruption_manager.user_activity_detected();
        
        // Extract content from message
        let content = self.extract_message_content(&message).await?;
        
        // Detect intent and context
        let intent = self.analyze_intent(&content).await?;
        debug!("Detected intent: {:?}", intent);
        
        // Check if verification is needed
        let requires_verification = self.requires_verification(&intent, &content).await?;
        
        if requires_verification {
            return self.handle_verification_flow(&content, &intent).await;
        }
        
        // Search for related context
        let related_context = self.search_conversation_history(&content).await?;
        
        // Generate appropriate response based on intent
        let response = match intent {
            IntentType::StrategicUpdate => self.generate_strategic_brief(&content, &related_context).await?,
            IntentType::Research => self.generate_research_response(&content).await?,
            IntentType::QuickQuestion => self.generate_quick_answer(&content, &related_context).await?,
            IntentType::TaskManagement => self.generate_task_response(&content).await?,
            IntentType::MeetingPrep => self.generate_meeting_prep(&content).await?,
            IntentType::DecisionSupport => self.generate_decision_support(&content, &related_context).await?,
            _ => self.generate_conversational_response(&content, &related_context).await?,
        };
        
        // Update conversation memory
        self.update_conversation_memory(&message, &response).await?;
        
        // Update topic tracking for proactive insights
        self.update_topic_tracking(&content, &response).await?;
        
        Ok(response)
    }
    
    /// Extract text content from various message types
    async fn extract_message_content(&self, message: &IncomingMessage) -> Result<String> {
        match &message.message_type {
            MessageType::Text { content } => Ok(content.clone()),
            MessageType::Voice { audio_path, duration_seconds } => {
                // Simulate voice processing delay
                self.timing.simulate_voice_processing(Duration::from_secs(*duration_seconds as u64)).await;
                
                // TODO: Integrate with Whisper for actual transcription
                // For now, simulate transcribed content
                Ok(format!("Transcribed voice note: [{}]", audio_path.display()))
            }
            MessageType::Image { image_path, caption } => {
                let text = caption.clone().unwrap_or_else(|| {
                    format!("Image shared: {}", image_path.display())
                });
                Ok(text)
            }
            MessageType::Document { doc_path, filename, caption } => {
                let text = caption.clone().unwrap_or_else(|| {
                    format!("Document shared: {}", filename)
                });
                Ok(text)
            }
            MessageType::Mixed { text, attachments } => {
                let mut combined = text.clone().unwrap_or_default();
                
                for attachment in attachments {
                    match attachment {
                        crate::signal_integration::note_to_self::Attachment::Voice { path, duration } => {
                            combined.push_str(&format!("\n[Voice note: {} seconds]", duration));
                        }
                        crate::signal_integration::note_to_self::Attachment::Image { path } => {
                            combined.push_str(&format!("\n[Image: {}]", path.display()));
                        }
                        crate::signal_integration::note_to_self::Attachment::Document { path, filename } => {
                            combined.push_str(&format!("\n[Document: {}]", filename));
                        }
                    }
                }
                
                Ok(combined)
            }
        }
    }
    
    /// Analyze user intent from message content
    async fn analyze_intent(&self, content: &str) -> Result<IntentType> {
        let content_lower = content.to_lowercase();
        
        // Strategic keywords
        if content_lower.contains("strategic") || content_lower.contains("competitive") 
           || content_lower.contains("market") || content_lower.contains("revenue") {
            return Ok(IntentType::StrategicUpdate);
        }
        
        // Confidential request keywords
        if content_lower.contains("confidential") || content_lower.contains("private")
           || content_lower.contains("board") || content_lower.contains("sensitive") {
            return Ok(IntentType::ConfidentialRequest);
        }
        
        // Research keywords
        if content_lower.contains("research") || content_lower.contains("analyze")
           || content_lower.contains("investigate") || content_lower.contains("study") {
            return Ok(IntentType::Research);
        }
        
        // Task management keywords
        if content_lower.contains("remind") || content_lower.contains("todo")
           || content_lower.contains("task") || content_lower.contains("deadline") {
            return Ok(IntentType::TaskManagement);
        }
        
        // Meeting prep keywords
        if content_lower.contains("meeting") || content_lower.contains("call")
           || content_lower.contains("presentation") || content_lower.contains("brief") {
            return Ok(IntentType::MeetingPrep);
        }
        
        // Decision support keywords
        if content_lower.contains("should") || content_lower.contains("decision")
           || content_lower.contains("options") || content_lower.contains("recommend") {
            return Ok(IntentType::DecisionSupport);
        }
        
        // Question indicators
        if content_lower.contains("?") || content_lower.starts_with("what")
           || content_lower.starts_with("how") || content_lower.starts_with("why") {
            return Ok(IntentType::QuickQuestion);
        }
        
        // Default to casual conversation
        Ok(IntentType::Casual)
    }
    
    /// Check if message requires identity verification
    async fn requires_verification(&self, intent: &IntentType, content: &str) -> Result<bool> {
        match intent {
            IntentType::ConfidentialRequest => Ok(true),
            _ => {
                // Check for sensitive keywords even in other intent types
                let content_lower = content.to_lowercase();
                let sensitive_keywords = ["financial", "salary", "contract", "legal", "board", "acquisition"];
                
                Ok(sensitive_keywords.iter().any(|&keyword| content_lower.contains(keyword)))
            }
        }
    }
    
    /// Handle verification flow for confidential requests
    async fn handle_verification_flow(
        &self,
        content: &str,
        intent: &IntentType,
    ) -> Result<ConversationalResponse> {
        Ok(ConversationalResponse {
            content: "I can help with that! For this type of request, I'll need to verify your identity quickly and privately. This keeps your information secure.\n\nTap here to verify: [Quick Verify]".to_string(),
            response_type: ResponseType::VerificationPrompt,
            action_items: vec![],
            follow_up_questions: vec![],
            background_research: None,
            requires_verification: true,
            urgency: UrgencyLevel::Medium,
        })
    }
    
    /// Generate strategic brief response
    async fn generate_strategic_brief(
        &self,
        content: &str,
        context: &[String],
    ) -> Result<ConversationalResponse> {
        let system_prompt = self.build_strategic_system_prompt(context);
        
        let conversation_id = format!("strategic-{}", Uuid::new_v4());
        
        // Initialize conversation
        self.hermes.create_conversation(
            conversation_id.clone(),
            Some(system_prompt),
        ).await?;
        
        // Generate response
        let user_message = HermesMessage {
            role: MessageRole::User,
            content: content.to_string(),
            metadata: None,
        };
        
        let ai_response = self.hermes.chat(&conversation_id, &user_message, None).await?;
        
        // Extract action items and questions
        let action_items = self.extract_action_items(&ai_response.content);
        let questions = self.extract_questions(&ai_response.content);
        
        Ok(ConversationalResponse {
            content: self.format_strategic_brief(&ai_response.content),
            response_type: ResponseType::StrategicBrief,
            action_items,
            follow_up_questions: questions,
            background_research: None,
            requires_verification: false,
            urgency: UrgencyLevel::Medium,
        })
    }
    
    /// Generate research response with background task
    async fn generate_research_response(&self, content: &str) -> Result<ConversationalResponse> {
        let research_task = ResearchTask {
            topic: self.extract_research_topic(content),
            research_type: self.determine_research_type(content),
            estimated_duration: Duration::from_secs(5 * 60),
            priority: Priority::Medium,
        };
        
        Ok(ConversationalResponse {
            content: format!(
                "I'll research {} for you. This will take about {} minutes.\n\nI'll start with a comprehensive analysis and get back to you with key insights, trends, and strategic implications.",
                research_task.topic,
                research_task.estimated_duration.as_secs() / 60
            ),
            response_type: ResponseType::ResearchSummary,
            action_items: vec![],
            follow_up_questions: vec![
                format!("What specific aspect of {} interests you most?", research_task.topic),
                "Do you need this for a particular decision or meeting?".to_string(),
            ],
            background_research: Some(research_task),
            requires_verification: false,
            urgency: UrgencyLevel::Low,
        })
    }
    
    /// Generate quick answer for simple questions
    async fn generate_quick_answer(
        &self,
        content: &str,
        context: &[String],
    ) -> Result<ConversationalResponse> {
        let system_prompt = "You are a knowledgeable executive assistant. Provide concise, accurate answers. Keep responses under 100 words unless the question requires detail.";
        
        let conversation_id = format!("quick-{}", Uuid::new_v4());
        
        self.hermes.create_conversation(
            conversation_id.clone(),
            Some(system_prompt.to_string()),
        ).await?;
        
        let user_message = HermesMessage {
            role: MessageRole::User,
            content: content.to_string(),
            metadata: None,
        };
        
        let ai_response = self.hermes.chat(&conversation_id, &user_message, None).await?;
        
        Ok(ConversationalResponse {
            content: ai_response.content,
            response_type: ResponseType::QuickAnswer,
            action_items: vec![],
            follow_up_questions: vec![],
            background_research: None,
            requires_verification: false,
            urgency: UrgencyLevel::Low,
        })
    }
    
    /// Generate task management response
    async fn generate_task_response(&self, content: &str) -> Result<ConversationalResponse> {
        let action_items = self.extract_tasks_from_content(content);
        
        let summary = if action_items.len() == 1 {
            format!("✓ Added task: {}", action_items[0].description)
        } else {
            format!("✓ Added {} tasks to your list", action_items.len())
        };
        
        Ok(ConversationalResponse {
            content: format!("{}\n\nI'll track these and remind you as needed. Want me to suggest optimal timing or dependencies?", summary),
            response_type: ResponseType::ActionableAdvice,
            action_items,
            follow_up_questions: vec![
                "Do any of these have specific deadlines?".to_string(),
                "Should I set up reminders for these tasks?".to_string(),
            ],
            background_research: None,
            requires_verification: false,
            urgency: UrgencyLevel::Low,
        })
    }
    
    /// Generate meeting preparation response
    async fn generate_meeting_prep(&self, content: &str) -> Result<ConversationalResponse> {
        Ok(ConversationalResponse {
            content: "I'll prepare a comprehensive brief for your meeting. Let me gather relevant context from your recent conversations and create talking points.\n\nThis will include key decisions, background context, and strategic recommendations.".to_string(),
            response_type: ResponseType::StrategicBrief,
            action_items: vec![
                ActionItem {
                    description: "Prepare meeting brief with context and talking points".to_string(),
                    priority: Priority::High,
                    due_date: None,
                    assigned_to: Some("AI Assistant".to_string()),
                    context: "Meeting preparation".to_string(),
                }
            ],
            follow_up_questions: vec![
                "Who are the key attendees I should research?".to_string(),
                "What outcomes are you hoping to achieve?".to_string(),
                "Do you need background on any specific topics?".to_string(),
            ],
            background_research: Some(ResearchTask {
                topic: "Meeting preparation and stakeholder analysis".to_string(),
                research_type: ResearchType::CompetitiveAnalysis,
                estimated_duration: Duration::from_secs(3 * 60),
                priority: Priority::High,
            }),
            requires_verification: false,
            urgency: UrgencyLevel::Medium,
        })
    }
    
    /// Generate decision support response
    async fn generate_decision_support(
        &self,
        content: &str,
        context: &[String],
    ) -> Result<ConversationalResponse> {
        Ok(ConversationalResponse {
            content: "I'll help you think through this decision systematically. Let me analyze the options, trade-offs, and strategic implications based on your context.\n\n**Decision Framework:**\n• Strategic alignment with goals\n• Risk assessment and mitigation\n• Resource requirements\n• Timeline considerations\n• Stakeholder impact".to_string(),
            response_type: ResponseType::ActionableAdvice,
            action_items: vec![
                ActionItem {
                    description: "Complete decision analysis framework".to_string(),
                    priority: Priority::Medium,
                    due_date: None,
                    assigned_to: Some("AI Assistant".to_string()),
                    context: "Decision support".to_string(),
                }
            ],
            follow_up_questions: vec![
                "What are the key criteria for this decision?".to_string(),
                "What's the timeline for making this decision?".to_string(),
                "Who else should be involved in this decision?".to_string(),
            ],
            background_research: None,
            requires_verification: false,
            urgency: UrgencyLevel::Medium,
        })
    }
    
    /// Generate general conversational response
    async fn generate_conversational_response(
        &self,
        content: &str,
        context: &[String],
    ) -> Result<ConversationalResponse> {
        let system_prompt = self.build_conversational_system_prompt(context);
        
        let conversation_id = format!("chat-{}", Uuid::new_v4());
        
        self.hermes.create_conversation(
            conversation_id.clone(),
            Some(system_prompt),
        ).await?;
        
        let user_message = HermesMessage {
            role: MessageRole::User,
            content: content.to_string(),
            metadata: None,
        };
        
        let ai_response = self.hermes.chat(&conversation_id, &user_message, None).await?;
        
        Ok(ConversationalResponse {
            content: ai_response.content,
            response_type: ResponseType::QuickAnswer,
            action_items: vec![],
            follow_up_questions: vec![],
            background_research: None,
            requires_verification: false,
            urgency: UrgencyLevel::Low,
        })
    }
    
    // Helper methods for content processing
    
    fn build_strategic_system_prompt(&self, context: &[String]) -> String {
        let context_section = if !context.is_empty() {
            format!("\n\nRelevant context from recent conversations:\n{}", context.join("\n"))
        } else {
            String::new()
        };
        
        format!(
            "You are a strategic executive assistant. Analyze business updates and provide executive-level insights.\n\n\
            Format responses as strategic briefs with:\n\
            - Executive Summary (key insights)\n\
            - Strategic Implications\n\
            - Recommended Actions\n\
            - Key Questions to Consider\n\n\
            Keep responses concise but comprehensive. Focus on actionable intelligence.{}",
            context_section
        )
    }
    
    fn build_conversational_system_prompt(&self, context: &[String]) -> String {
        let memory = self.memory.try_read().ok();
        let user_style = memory
            .as_ref()
            .map(|m| &m.user_preferences.preferred_response_style)
            .unwrap_or(&ResponseStyle::Executive);
        
        let style_instruction = match user_style {
            ResponseStyle::Executive => "Respond in a professional, strategic manner with clear actionable insights.",
            ResponseStyle::Casual => "Respond in a friendly, conversational tone that's approachable and helpful.",
            ResponseStyle::Academic => "Provide detailed, analytical responses with thorough explanations.",
            ResponseStyle::Technical => "Give precise, technical responses with specific implementation details.",
            ResponseStyle::Creative => "Respond in an engaging way that encourages creative thinking.",
        };
        
        format!(
            "You are a knowledgeable personal assistant. {}. \
            Provide helpful, contextual responses that add value to the conversation.",
            style_instruction
        )
    }
    
    fn format_strategic_brief(&self, content: &str) -> String {
        format!("# Strategic Brief\n\n{}", content)
    }
    
    fn extract_action_items(&self, content: &str) -> Vec<ActionItem> {
        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("- [ ]") || trimmed.starts_with("□") || trimmed.contains("action:") {
                    Some(ActionItem {
                        description: trimmed.trim_start_matches("- [ ]")
                                          .trim_start_matches("□")
                                          .trim_start_matches("action:")
                                          .trim()
                                          .to_string(),
                        priority: Priority::Medium,
                        due_date: None,
                        assigned_to: None,
                        context: "Generated from conversation".to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn extract_questions(&self, content: &str) -> Vec<String> {
        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.ends_with('?') && !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn extract_research_topic(&self, content: &str) -> String {
        // Simple keyword extraction for research topics
        content
            .split_whitespace()
            .skip(1) // Skip "research" or similar command word
            .take(5)  // Take next 5 words as topic
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn determine_research_type(&self, content: &str) -> ResearchType {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("competitor") || content_lower.contains("competition") {
            ResearchType::CompetitiveAnalysis
        } else if content_lower.contains("market") || content_lower.contains("industry") {
            ResearchType::MarketResearch
        } else if content_lower.contains("technology") || content_lower.contains("tech") {
            ResearchType::TechnologyTrends
        } else if content_lower.contains("financial") || content_lower.contains("revenue") {
            ResearchType::Financial
        } else if content_lower.contains("regulation") || content_lower.contains("compliance") {
            ResearchType::Regulatory
        } else {
            ResearchType::IndustryNews
        }
    }
    
    fn extract_tasks_from_content(&self, content: &str) -> Vec<ActionItem> {
        // Simple task extraction - in real implementation would use NLP
        vec![ActionItem {
            description: content.to_string(),
            priority: Priority::Medium,
            due_date: None,
            assigned_to: None,
            context: "Extracted from message".to_string(),
        }]
    }
    
    async fn search_conversation_history(&self, _query: &str) -> Result<Vec<String>> {
        // TODO: Implement semantic search through conversation history
        // For now, return empty context
        Ok(vec![])
    }
    
    async fn update_conversation_memory(
        &self,
        _message: &IncomingMessage,
        _response: &ConversationalResponse,
    ) -> Result<()> {
        // TODO: Update conversation memory with new message and response
        Ok(())
    }
    
    async fn update_topic_tracking(
        &self,
        _content: &str,
        _response: &ConversationalResponse,
    ) -> Result<()> {
        // TODO: Track topics and detect patterns for proactive insights
        Ok(())
    }
    
    /// Generate proactive insights based on conversation patterns
    pub async fn generate_periodic_insight(&self) -> Result<Option<ProactiveInsight>> {
        // TODO: Analyze conversation patterns and generate proactive insights
        Ok(None)
    }
    
    /// Process calendar events for proactive assistance
    pub async fn process_calendar_event(&self, _event: CalendarEvent) -> Result<Option<ConversationalResponse>> {
        // TODO: Process calendar events and generate proactive meeting prep
        Ok(None)
    }
    
    /// Handle completion of background research
    pub async fn background_research_completed(&mut self) -> Option<ResearchResult> {
        // TODO: Handle background research completion
        None
    }
    
    /// Format research results for user
    pub async fn format_research_result(&self, _result: ResearchResult) -> Result<String> {
        // TODO: Format research results
        Ok("Research completed.".to_string())
    }
}

// Placeholder types for future implementation
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub title: String,
    pub start_time: SystemTime,
    pub attendees: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResearchResult {
    pub topic: String,
    pub findings: String,
    pub sources: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_intent_detection() {
        let assistant = ConversationalAssistant::new().await.unwrap();
        
        // Test strategic intent
        let strategic_intent = assistant.analyze_intent("Here's my strategic update on Q1 revenue").await.unwrap();
        assert!(matches!(strategic_intent, IntentType::StrategicUpdate));
        
        // Test research intent
        let research_intent = assistant.analyze_intent("Please research the competitive landscape").await.unwrap();
        assert!(matches!(research_intent, IntentType::Research));
        
        // Test question intent
        let question_intent = assistant.analyze_intent("What's the status of the Tokyo project?").await.unwrap();
        assert!(matches!(question_intent, IntentType::QuickQuestion));
    }
    
    #[tokio::test]
    async fn test_verification_requirement() {
        let assistant = ConversationalAssistant::new().await.unwrap();
        
        // Should require verification for confidential content
        let confidential = assistant.requires_verification(
            &IntentType::ConfidentialRequest, 
            "I need the board presentation"
        ).await.unwrap();
        assert!(confidential);
        
        // Should not require verification for general questions
        let general = assistant.requires_verification(
            &IntentType::QuickQuestion,
            "What's the weather like?"
        ).await.unwrap();
        assert!(!general);
    }
    
    #[test]
    fn test_conversation_timing() {
        let timing = ConversationTiming::executive_assistant();
        
        // Short response should have minimal delay
        let short_delay = timing.calculate_response_delay(50);
        assert!(short_delay >= Duration::from_millis(500));
        assert!(short_delay <= Duration::from_millis(2000));
        
        // Long response should have thinking delay
        let long_delay = timing.calculate_response_delay(1000);
        assert!(long_delay > Duration::from_millis(2000));
        assert!(long_delay <= Duration::from_millis(5000));
    }
    
    #[test]
    fn test_interruption_management() {
        let manager = InterruptionManager::new();
        
        let critical_insight = ProactiveInsight {
            insight: "Critical issue detected".to_string(),
            urgency: UrgencyLevel::Critical,
            topic: "System".to_string(),
            suggested_action: "Immediate attention needed".to_string(),
            confidence: 0.9,
            generated_at: SystemTime::now(),
        };
        
        let decision = manager.should_send_proactive_message(&critical_insight);
        assert_eq!(decision, InterruptDecision::SendImmediately);
    }
}
