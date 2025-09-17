// MVP Core: Intelligence Compression Engine
// Transforms information overload into actionable 10-bit decisions

// Core compression only - industry specializations moved to research phase

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Core compression engine that transforms high-volume information into
/// human-optimized intelligence matching cognitive processing limits
pub struct CompressionEngine {
    #[cfg(feature = "analytics")]
    embeddings: Box<dyn crate::vault::embeddings::EmbeddingProvider>,
    #[cfg(feature = "analytics")]
    storage: crate::vault::storage::HybridStorageEngine,
    pattern_matcher: PatternMatcher,
    relevance_scorer: RelevanceScorer,
    cognitive_interface: CognitiveInterface,
}

/// Information intake from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationPacket {
    pub content: String,
    pub source: InformationSource,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InformationSource {
    Voice,
    Document,
    Signal,
    Meeting,
    Research,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Urgent,      // Immediate decision needed
    Important,   // Decision needed today
    Normal,      // Decision needed this week
    Background,  // For pattern building
}

/// Human-optimized output matching cognitive processing speed (under clocked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveOutput {
    pub decision_points: Vec<DecisionPoint>,     // 8 bits/second (under clocked)
    pub summary: Option<IntelligenceSummary>,    // 32 bits/second (under clocked)
    pub deep_context: Option<DeepContext>,       // 64 bits/second (under clocked)
    pub confidence: f32,
    pub processing_time: std::time::Duration,
}

/// Single decision point optimized for 8-bit human processing (under clocked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub question: String,           // Simple yes/no or choice
    pub options: Vec<String>,       // Max 3 options
    pub recommendation: String,     // AI recommendation
    pub confidence: f32,
    pub urgency: Priority,
}

/// 32-bit summary for conscious reading (under clocked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceSummary {
    pub headline: String,           // One sentence takeaway
    pub key_points: Vec<String>,    // Max 3 bullet points
    pub action_required: bool,
    pub deadline: Option<DateTime<Utc>>,
}

/// 64-bit deep context when user needs full picture (under clocked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepContext {
    pub analysis: String,           // Detailed analysis
    pub supporting_evidence: Vec<String>,
    pub related_patterns: Vec<String>,
    pub recommendations: Vec<String>,
}

impl CompressionEngine {
    pub async fn new() -> Result<Self, CompressionError> {
        Ok(Self {
            #[cfg(feature = "analytics")]
            embeddings: Box::new(crate::vault::embeddings::Embeddings::new().map_err(|e| CompressionError::EmbeddingError(e.to_string()))?),
            #[cfg(feature = "analytics")]
            storage: crate::vault::storage::HybridStorageEngine::new(std::path::PathBuf::from("./data")).await
                .map_err(|e| CompressionError::StorageError(e.to_string()))?,
            pattern_matcher: PatternMatcher::new(),
            relevance_scorer: RelevanceScorer::new(),
            cognitive_interface: CognitiveInterface::new(),
        })
    }

    /// Core compression function: Information → Intelligence
    pub async fn compress_intelligence(
        &mut self,
        information: Vec<InformationPacket>,
        context: CompressionContext,
    ) -> Result<CognitiveOutput, CompressionError> {
        let start_time = std::time::Instant::now();
        
        // Step 1: Generate embeddings for semantic understanding
        let mut enriched_info = Vec::new();
        #[cfg(feature = "analytics")]
        {
            for packet in information {
                let embedding = self.embeddings.embed(&packet.content, "default").await
                    .map_err(|e| CompressionError::EmbeddingError(e.to_string()))?;
                enriched_info.push(EnrichedInformation {
                    packet,
                    embedding,
                    patterns: Vec::new(),
                    relevance_score: 0.0,
                });
            }
        }
        #[cfg(not(feature = "analytics"))]
        {
            for packet in information {
                // Use fallback embedding for compression engine without analytics
                let embedding = vec![0.0; 384]; // Standard embedding dimension
                enriched_info.push(EnrichedInformation {
                    packet,
                    embedding,
                    patterns: Vec::new(),
                    relevance_score: 0.0,
                });
            }
        }

        // Step 2: Pattern recognition across information streams
        let patterns = self.pattern_matcher.identify_patterns(&enriched_info, &context).await?;
        
        // Step 3: Score relevance to user context
        for info in &mut enriched_info {
            info.relevance_score = self.relevance_scorer.score_relevance(&info.packet, &context);
            info.patterns = patterns.iter()
                .filter(|p| p.applies_to(&info.packet))
                .cloned()
                .collect();
        }

        // Step 4: Compress to human-optimized output
        let compressed = self.cognitive_interface.compress_to_cognitive_output(
            &enriched_info,
            &patterns,
            &context,
        ).await?;

        // Step 5: Store for pattern learning (simplified for MVP)
        // self.storage.store_compression_event(&information, &compressed, &context).await?;

        Ok(CognitiveOutput {
            processing_time: start_time.elapsed(),
            ..compressed
        })
    }

    /// Industry-specific compression with domain patterns
    pub async fn compress_for_industry(
        &mut self,
        information: Vec<InformationPacket>,
        industry: Industry,
        context: CompressionContext,
    ) -> Result<CognitiveOutput, CompressionError> {
        let industry_context = CompressionContext {
            industry_patterns: industry.get_patterns(),
            domain_vocabulary: industry.get_vocabulary(),
            ..context
        };

        self.compress_intelligence(information, industry_context).await
    }
}

/// Context for intelligent compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionContext {
    pub user_role: String,                          // "lawyer", "doctor", "analyst"
    pub current_projects: Vec<String>,              // Active work context
    pub decision_urgency: Priority,                 // How fast decision needed
    pub domain_expertise: f32,                      // 0.0-1.0 expertise level
    pub cognitive_load: CognitiveLoad,              // Current mental capacity
    pub industry_patterns: Vec<IndustryPattern>,    // Domain-specific patterns
    pub domain_vocabulary: HashMap<String, String>, // Industry terminology
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitiveLoad {
    Low,      // Fresh, can handle complex analysis
    Medium,   // Normal working state
    High,     // Tired, need simple decisions only
    Critical, // Overloaded, emergency decisions only
}

/// Industry-specific pattern recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Industry {
    Legal,
    Medical,
    Government,
    Military,
    Finance,
    Generic,
}

impl Industry {
    pub fn get_patterns(&self) -> Vec<IndustryPattern> {
        match self {
            Industry::Legal => vec![
                IndustryPattern::new("precedent_analysis", "Legal precedent identification"),
                IndustryPattern::new("risk_assessment", "Legal risk evaluation"),
                IndustryPattern::new("deadline_tracking", "Court and filing deadlines"),
            ],
            Industry::Medical => vec![
                IndustryPattern::new("symptom_clustering", "Symptom pattern recognition"),
                IndustryPattern::new("treatment_protocols", "Evidence-based treatment"),
                IndustryPattern::new("contraindication_check", "Drug interaction safety"),
            ],
            Industry::Government => vec![
                IndustryPattern::new("stakeholder_analysis", "Policy stakeholder mapping"),
                IndustryPattern::new("impact_assessment", "Policy impact evaluation"),
                IndustryPattern::new("timeline_analysis", "Implementation timeline"),
            ],
            _ => vec![
                IndustryPattern::new("priority_detection", "Priority identification"),
                IndustryPattern::new("action_extraction", "Action item extraction"),
                IndustryPattern::new("deadline_detection", "Deadline identification"),
            ],
        }
    }

    pub fn get_vocabulary(&self) -> HashMap<String, String> {
        let mut vocab = HashMap::new();
        match self {
            Industry::Legal => {
                vocab.insert("precedent".to_string(), "Legal case that establishes principle".to_string());
                vocab.insert("motion".to_string(), "Formal request to court".to_string());
                vocab.insert("discovery".to_string(), "Pre-trial evidence exchange".to_string());
            },
            Industry::Medical => {
                vocab.insert("diagnosis".to_string(), "Medical condition identification".to_string());
                vocab.insert("protocol".to_string(), "Standardized treatment procedure".to_string());
                vocab.insert("contraindication".to_string(), "Reason to avoid treatment".to_string());
            },
            Industry::Government => {
                vocab.insert("stakeholder".to_string(), "Affected party in policy".to_string());
                vocab.insert("mandate".to_string(), "Official requirement".to_string());
                vocab.insert("compliance".to_string(), "Adherence to regulations".to_string());
            },
            _ => {},
        }
        vocab
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryPattern {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub actions: Vec<String>,
}

impl IndustryPattern {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            triggers: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn applies_to(&self, packet: &InformationPacket) -> bool {
        self.triggers.iter().any(|trigger| 
            packet.content.to_lowercase().contains(&trigger.to_lowercase())
        )
    }
}

/// Internal structures for processing
#[derive(Debug, Clone)]
struct EnrichedInformation {
    packet: InformationPacket,
    embedding: Vec<f32>,
    patterns: Vec<IndustryPattern>,
    relevance_score: f32,
}

/// Pattern matching system
struct PatternMatcher;

impl PatternMatcher {
    fn new() -> Self {
        Self
    }

    async fn identify_patterns(
        &self,
        information: &[EnrichedInformation],
        context: &CompressionContext,
    ) -> Result<Vec<IndustryPattern>, CompressionError> {
        let mut patterns = context.industry_patterns.clone();
        
        // Add dynamic pattern detection based on information content
        for info in information {
            // Analyze content for patterns not in industry defaults
            if info.packet.content.contains("deadline") || info.packet.content.contains("due") {
                patterns.push(IndustryPattern::new("deadline_pattern", "Time-sensitive requirement"));
            }
            
            if info.packet.content.contains("decide") || info.packet.content.contains("choose") {
                patterns.push(IndustryPattern::new("decision_pattern", "Decision point identified"));
            }
        }
        
        Ok(patterns)
    }
}

/// Relevance scoring system
struct RelevanceScorer;

impl RelevanceScorer {
    fn new() -> Self {
        Self
    }

    fn score_relevance(&self, packet: &InformationPacket, context: &CompressionContext) -> f32 {
        let mut score = 0.0;
        
        // Priority weighting
        score += match packet.priority {
            Priority::Urgent => 1.0,
            Priority::Important => 0.8,
            Priority::Normal => 0.5,
            Priority::Background => 0.2,
        };
        
        // Recency weighting
        let age_hours = (Utc::now() - packet.timestamp).num_hours() as f32;
        score += (24.0 - age_hours.min(24.0)) / 24.0 * 0.5;
        
        // Project relevance
        for project in &context.current_projects {
            if packet.content.to_lowercase().contains(&project.to_lowercase()) {
                score += 0.3;
            }
        }
        
        score.min(1.0)
    }
}

/// Cognitive interface for human-optimized output
struct CognitiveInterface;

impl CognitiveInterface {
    fn new() -> Self {
        Self
    }

    async fn compress_to_cognitive_output(
        &self,
        information: &[EnrichedInformation],
        patterns: &[IndustryPattern],
        context: &CompressionContext,
    ) -> Result<CognitiveOutput, CompressionError> {
        // Sort by relevance and priority
        let mut sorted_info: Vec<_> = information.iter().collect();
        sorted_info.sort_by(|a, b| {
            b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.packet.priority.cmp(&a.packet.priority))
        });

        // Generate decision points (10-bit processing)
        let decision_points = self.generate_decision_points(&sorted_info, context).await?;
        
        // Generate summary (40-bit processing)
        let summary = if matches!(context.cognitive_load, CognitiveLoad::Critical) {
            None // Skip summary when overloaded
        } else {
            Some(self.generate_summary(&sorted_info, patterns).await?)
        };
        
        // Generate deep context (120-bit processing)
        let deep_context = if matches!(context.cognitive_load, CognitiveLoad::Low) {
            Some(self.generate_deep_context(&sorted_info, patterns).await?)
        } else {
            None // Skip deep analysis unless fresh
        };

        // Calculate confidence based on information quality
        let confidence = self.calculate_confidence(&sorted_info);

        Ok(CognitiveOutput {
            decision_points,
            summary,
            deep_context,
            confidence,
            processing_time: std::time::Duration::from_millis(0), // Set by caller
        })
    }

    async fn generate_decision_points(
        &self,
        information: &[&EnrichedInformation],
        context: &CompressionContext,
    ) -> Result<Vec<DecisionPoint>, CompressionError> {
        let mut decisions = Vec::new();
        
        // Extract urgent decisions first
        for info in information.iter().take(3) { // Max 3 decisions for 10-bit processing
            if info.packet.priority == Priority::Urgent {
                decisions.push(DecisionPoint {
                    question: format!("Action needed: {}", self.extract_key_question(&info.packet.content)),
                    options: vec!["Yes".to_string(), "No".to_string(), "More Info".to_string()],
                    recommendation: self.generate_recommendation(&info.packet.content, context),
                    confidence: info.relevance_score,
                    urgency: info.packet.priority.clone(),
                });
            }
        }
        
        // If no urgent decisions, create prioritized decisions
        if decisions.is_empty() {
            for info in information.iter().take(2) {
                decisions.push(DecisionPoint {
                    question: self.extract_key_question(&info.packet.content),
                    options: self.generate_options(&info.packet.content),
                    recommendation: self.generate_recommendation(&info.packet.content, context),
                    confidence: info.relevance_score,
                    urgency: info.packet.priority.clone(),
                });
            }
        }
        
        Ok(decisions)
    }

    async fn generate_summary(
        &self,
        information: &[&EnrichedInformation],
        _patterns: &[IndustryPattern],
    ) -> Result<IntelligenceSummary, CompressionError> {
        let headline = if information.is_empty() {
            "No new information".to_string()
        } else {
            format!("{} items processed, {} require attention", 
                information.len(),
                information.iter().filter(|i| i.packet.priority != Priority::Background).count()
            )
        };

        let key_points: Vec<String> = information.iter()
            .take(3)
            .map(|info| self.extract_key_point(&info.packet.content))
            .collect();

        let action_required = information.iter()
            .any(|info| matches!(info.packet.priority, Priority::Urgent | Priority::Important));

        let deadline = information.iter()
            .filter_map(|info| info.packet.metadata.get("deadline"))
            .filter_map(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc))
            .min();

        Ok(IntelligenceSummary {
            headline,
            key_points,
            action_required,
            deadline,
        })
    }

    async fn generate_deep_context(
        &self,
        information: &[&EnrichedInformation],
        patterns: &[IndustryPattern],
    ) -> Result<DeepContext, CompressionError> {
        let analysis = format!(
            "Analyzed {} information packets across {} patterns. Key themes: {}",
            information.len(),
            patterns.len(),
            patterns.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
        );

        let supporting_evidence: Vec<String> = information.iter()
            .take(5)
            .map(|info| format!("• {}", self.extract_evidence(&info.packet.content)))
            .collect();

        let related_patterns: Vec<String> = patterns.iter()
            .map(|p| format!("• {}: {}", p.name, p.description))
            .collect();

        let recommendations: Vec<String> = information.iter()
            .filter(|info| info.packet.priority != Priority::Background)
            .take(3)
            .map(|info| format!("• {}", self.generate_recommendation(&info.packet.content, &CompressionContext::default())))
            .collect();

        Ok(DeepContext {
            analysis,
            supporting_evidence,
            related_patterns,
            recommendations,
        })
    }

    fn calculate_confidence(&self, information: &[&EnrichedInformation]) -> f32 {
        if information.is_empty() {
            return 0.0;
        }

        let avg_relevance: f32 = information.iter()
            .map(|info| info.relevance_score)
            .sum::<f32>() / information.len() as f32;

        let recency_factor: f32 = information.iter()
            .map(|info| {
                let age_hours = (Utc::now() - info.packet.timestamp).num_hours() as f32;
                (24.0 - age_hours.min(24.0)) / 24.0
            })
            .sum::<f32>() / information.len() as f32;

        (avg_relevance * 0.7 + recency_factor * 0.3).min(1.0)
    }

    // Helper methods for text processing
    fn extract_key_question(&self, content: &str) -> String {
        // Simple extraction - in real implementation, use NLP
        if content.contains("?") {
            content.split('?').next().unwrap_or(content).trim().to_string() + "?"
        } else {
            format!("Review: {}", content.split_whitespace().take(10).collect::<Vec<_>>().join(" "))
        }
    }

    fn generate_options(&self, content: &str) -> Vec<String> {
        // Simple option generation - in real implementation, use AI
        if content.to_lowercase().contains("approve") {
            vec!["Approve".to_string(), "Reject".to_string(), "Modify".to_string()]
        } else if content.to_lowercase().contains("schedule") {
            vec!["Today".to_string(), "This Week".to_string(), "Next Week".to_string()]
        } else {
            vec!["Yes".to_string(), "No".to_string(), "More Info".to_string()]
        }
    }

    fn generate_recommendation(&self, _content: &str, context: &CompressionContext) -> String {
        // Simple recommendation - in real implementation, use AI with context
        match context.cognitive_load {
            CognitiveLoad::Critical => "Defer non-urgent".to_string(),
            CognitiveLoad::High => "Quick decision recommended".to_string(),
            _ => "Analyze and decide".to_string(),
        }
    }

    fn extract_key_point(&self, content: &str) -> String {
        // Extract first sentence or key phrase
        content.split('.').next().unwrap_or(content).trim().to_string()
    }

    fn extract_evidence(&self, content: &str) -> String {
        // Extract supporting evidence
        content.split_whitespace().take(15).collect::<Vec<_>>().join(" ")
    }
}

impl Default for CompressionContext {
    fn default() -> Self {
        Self {
            user_role: "user".to_string(),
            current_projects: Vec::new(),
            decision_urgency: Priority::Normal,
            domain_expertise: 0.5,
            cognitive_load: CognitiveLoad::Medium,
            industry_patterns: Vec::new(),
            domain_vocabulary: HashMap::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Pattern recognition failed: {0}")]
    PatternError(String),
    
    #[error("Cognitive interface error: {0}")]
    InterfaceError(String),
}
