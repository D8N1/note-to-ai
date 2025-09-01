// Legal Intelligence Engine
// Specialized compression for legal professionals
// Output: "3 precedents support argument. 1 risk identified. Decision needed."

use super::{CompressionEngine, InformationPacket, CompressionContext, CognitiveOutput};
use super::{Industry, Priority, CognitiveLoad};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct LegalCompressionEngine {
    core: CompressionEngine,
    legal_patterns: LegalPatternRecognizer,
    case_law_index: CaseLawEmbeddings,
    precedent_analyzer: PrecedentAnalyzer,
}

impl LegalCompressionEngine {
    pub async fn new() -> Result<Self, LegalCompressionError> {
        Ok(Self {
            core: CompressionEngine::new().await?,
            legal_patterns: LegalPatternRecognizer::new(),
            case_law_index: CaseLawEmbeddings::new().await?,
            precedent_analyzer: PrecedentAnalyzer::new(),
        })
    }

    /// Compress legal information into actionable intelligence
    pub async fn analyze_legal_case(
        &mut self,
        documents: Vec<LegalDocument>,
        case_context: LegalContext,
    ) -> Result<LegalIntelligence, LegalCompressionError> {
        // Convert legal documents to information packets
        let information: Vec<InformationPacket> = documents.into_iter()
            .map(|doc| doc.into_information_packet())
            .collect();

        // Create legal-specific compression context
        let compression_context = CompressionContext {
            user_role: case_context.attorney_role.clone(),
            current_projects: case_context.active_cases.clone(),
            decision_urgency: case_context.court_deadline_priority(),
            domain_expertise: case_context.experience_level,
            cognitive_load: case_context.current_workload.clone(),
            industry_patterns: Industry::Legal.get_patterns(),
            domain_vocabulary: Industry::Legal.get_vocabulary(),
        };

        // Perform core compression
        let base_output = self.core.compress_for_industry(
            information,
            Industry::Legal,
            compression_context,
        ).await?;

        // Add legal-specific analysis
        let legal_analysis = self.perform_legal_analysis(&base_output, &case_context).await?;

        Ok(LegalIntelligence {
            base_intelligence: base_output,
            legal_analysis,
            case_context,
        })
    }

    async fn perform_legal_analysis(
        &self,
        base_output: &CognitiveOutput,
        context: &LegalContext,
    ) -> Result<LegalAnalysis, LegalCompressionError> {
        let mut analysis = LegalAnalysis::default();

        // Analyze precedents
        analysis.precedent_analysis = self.precedent_analyzer
            .find_supporting_precedents(&base_output, context).await?;

        // Identify legal risks
        analysis.risk_assessment = self.legal_patterns
            .assess_legal_risks(&base_output, context).await?;

        // Extract legal strategies
        analysis.recommended_strategies = self.legal_patterns
            .suggest_legal_strategies(&base_output, context).await?;

        // Check deadlines and procedural requirements
        analysis.procedural_requirements = self.legal_patterns
            .check_procedural_deadlines(&base_output, context).await?;

        Ok(analysis)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalDocument {
    pub document_type: LegalDocumentType,
    pub content: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub parties: Vec<String>,
    pub jurisdiction: Option<String>,
    pub case_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegalDocumentType {
    CaseLaw,
    Statute,
    Regulation,
    Contract,
    Pleading,
    Motion,
    Brief,
    Deposition,
    Discovery,
    CourtOrder,
    Opinion,
}

impl LegalDocument {
    fn into_information_packet(self) -> InformationPacket {
        let priority = match self.document_type {
            LegalDocumentType::CourtOrder => Priority::Urgent,
            LegalDocumentType::Motion | LegalDocumentType::Pleading => Priority::Important,
            LegalDocumentType::CaseLaw | LegalDocumentType::Opinion => Priority::Normal,
            _ => Priority::Background,
        };

        let mut metadata = HashMap::new();
        metadata.insert("document_type".to_string(), format!("{:?}", self.document_type));
        metadata.insert("source".to_string(), self.source);
        if let Some(jurisdiction) = self.jurisdiction {
            metadata.insert("jurisdiction".to_string(), jurisdiction);
        }
        if let Some(case_number) = self.case_number {
            metadata.insert("case_number".to_string(), case_number);
        }

        InformationPacket {
            content: self.content,
            source: crate::compression::InformationSource::Document,
            timestamp: self.date,
            metadata,
            priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalContext {
    pub attorney_role: String,          // "prosecutor", "defense", "corporate", "plaintiff"
    pub practice_areas: Vec<String>,    // "criminal", "civil", "corporate", etc.
    pub active_cases: Vec<String>,
    pub court_deadlines: Vec<CourtDeadline>,
    pub experience_level: f32,          // 0.0-1.0
    pub current_workload: CognitiveLoad,
    pub jurisdiction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtDeadline {
    pub case: String,
    pub deadline_type: String,  // "filing", "discovery", "trial"
    pub due_date: chrono::DateTime<chrono::Utc>,
    pub priority: Priority,
}

impl LegalContext {
    fn court_deadline_priority(&self) -> Priority {
        let now = chrono::Utc::now();
        let urgent_threshold = chrono::Duration::days(3);
        let important_threshold = chrono::Duration::weeks(1);

        for deadline in &self.court_deadlines {
            let time_until = deadline.due_date - now;
            if time_until < urgent_threshold {
                return Priority::Urgent;
            } else if time_until < important_threshold {
                return Priority::Important;
            }
        }
        Priority::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalIntelligence {
    pub base_intelligence: CognitiveOutput,
    pub legal_analysis: LegalAnalysis,
    pub case_context: LegalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegalAnalysis {
    pub precedent_analysis: PrecedentAnalysis,
    pub risk_assessment: RiskAssessment,
    pub recommended_strategies: Vec<LegalStrategy>,
    pub procedural_requirements: Vec<ProceduralRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrecedentAnalysis {
    pub supporting_precedents: Vec<Precedent>,
    pub opposing_precedents: Vec<Precedent>,
    pub strength_score: f32,  // 0.0-1.0
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precedent {
    pub case_name: String,
    pub citation: String,
    pub relevance_score: f32,
    pub key_holding: String,
    pub jurisdiction: String,
    pub year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskAssessment {
    pub identified_risks: Vec<LegalRisk>,
    pub overall_risk_level: RiskLevel,
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalRisk {
    pub risk_type: String,
    pub description: String,
    pub probability: f32,  // 0.0-1.0
    pub impact: RiskLevel,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalStrategy {
    pub strategy_name: String,
    pub description: String,
    pub success_probability: f32,
    pub required_resources: Vec<String>,
    pub timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralRequirement {
    pub requirement_type: String,
    pub description: String,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub status: RequirementStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementStatus {
    Pending,
    InProgress,
    Complete,
    Overdue,
}

// Legal pattern recognition system
struct LegalPatternRecognizer;

impl LegalPatternRecognizer {
    fn new() -> Self {
        Self
    }

    async fn assess_legal_risks(
        &self,
        output: &CognitiveOutput,
        _context: &LegalContext,
    ) -> Result<RiskAssessment, LegalCompressionError> {
        let mut risks = Vec::new();
        
        // Check for statute of limitations issues
        if let Some(summary) = &output.summary {
            if summary.headline.to_lowercase().contains("statute") {
                risks.push(LegalRisk {
                    risk_type: "Statute of Limitations".to_string(),
                    description: "Potential time limitation issue identified".to_string(),
                    probability: 0.7,
                    impact: RiskLevel::High,
                    suggested_action: "Verify filing deadlines immediately".to_string(),
                });
            }
        }

        // Check for jurisdiction issues
        for decision in &output.decision_points {
            if decision.question.to_lowercase().contains("jurisdiction") {
                risks.push(LegalRisk {
                    risk_type: "Jurisdictional".to_string(),
                    description: "Jurisdiction question may affect case strategy".to_string(),
                    probability: 0.6,
                    impact: RiskLevel::Medium,
                    suggested_action: "Clarify jurisdictional authority".to_string(),
                });
            }
        }

        let overall_risk_level = if risks.iter().any(|r| matches!(r.impact, RiskLevel::Critical)) {
            RiskLevel::Critical
        } else if risks.iter().any(|r| matches!(r.impact, RiskLevel::High)) {
            RiskLevel::High
        } else if !risks.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(RiskAssessment {
            identified_risks: risks,
            overall_risk_level,
            mitigation_strategies: vec![
                "Review all deadlines".to_string(),
                "Verify jurisdiction".to_string(),
                "Check precedent strength".to_string(),
            ],
        })
    }

    async fn suggest_legal_strategies(
        &self,
        output: &CognitiveOutput,
        _context: &LegalContext,
    ) -> Result<Vec<LegalStrategy>, LegalCompressionError> {
        let mut strategies = Vec::new();

        // Analyze decision points for strategy opportunities
        for decision in &output.decision_points {
            if decision.question.to_lowercase().contains("motion") {
                strategies.push(LegalStrategy {
                    strategy_name: "Motion Strategy".to_string(),
                    description: "File strategic motion based on case analysis".to_string(),
                    success_probability: decision.confidence,
                    required_resources: vec!["Legal research".to_string(), "Brief writing".to_string()],
                    timeline: "2-3 weeks".to_string(),
                });
            }

            if decision.question.to_lowercase().contains("settlement") {
                strategies.push(LegalStrategy {
                    strategy_name: "Settlement Negotiation".to_string(),
                    description: "Pursue settlement based on case strength".to_string(),
                    success_probability: decision.confidence * 0.8,
                    required_resources: vec!["Negotiation preparation".to_string(), "Valuation analysis".to_string()],
                    timeline: "1-4 weeks".to_string(),
                });
            }
        }

        Ok(strategies)
    }

    async fn check_procedural_deadlines(
        &self,
        _output: &CognitiveOutput,
        context: &LegalContext,
    ) -> Result<Vec<ProceduralRequirement>, LegalCompressionError> {
        let mut requirements = Vec::new();

        // Convert court deadlines to procedural requirements
        for deadline in &context.court_deadlines {
            let status = if deadline.due_date < chrono::Utc::now() {
                RequirementStatus::Overdue
            } else if deadline.due_date - chrono::Utc::now() < chrono::Duration::days(7) {
                RequirementStatus::InProgress
            } else {
                RequirementStatus::Pending
            };

            requirements.push(ProceduralRequirement {
                requirement_type: deadline.deadline_type.clone(),
                description: format!("Case: {}", deadline.case),
                deadline: deadline.due_date,
                status,
            });
        }

        Ok(requirements)
    }
}

// Case law embeddings and search
struct CaseLawEmbeddings;

impl CaseLawEmbeddings {
    async fn new() -> Result<Self, LegalCompressionError> {
        Ok(Self)
    }
}

// Precedent analysis system
struct PrecedentAnalyzer;

impl PrecedentAnalyzer {
    fn new() -> Self {
        Self
    }

    async fn find_supporting_precedents(
        &self,
        output: &CognitiveOutput,
        context: &LegalContext,
    ) -> Result<PrecedentAnalysis, LegalCompressionError> {
        // In a real implementation, this would search case law databases
        // For MVP, return simplified analysis
        
        let supporting_precedents = vec![
            Precedent {
                case_name: "Sample v. Case".to_string(),
                citation: "123 F.3d 456 (9th Cir. 2020)".to_string(),
                relevance_score: 0.85,
                key_holding: "Key legal principle supporting argument".to_string(),
                jurisdiction: context.jurisdiction.clone(),
                year: 2020,
            }
        ];

        let opposing_precedents = vec![
            Precedent {
                case_name: "Counter v. Example".to_string(),
                citation: "789 F.3d 101 (9th Cir. 2019)".to_string(),
                relevance_score: 0.60,
                key_holding: "Principle that may limit argument".to_string(),
                jurisdiction: context.jurisdiction.clone(),
                year: 2019,
            }
        ];

        let strength_score = if supporting_precedents.len() > opposing_precedents.len() {
            0.75
        } else {
            0.45
        };

        Ok(PrecedentAnalysis {
            supporting_precedents,
            opposing_precedents,
            strength_score,
            confidence: output.confidence,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LegalCompressionError {
    #[error("Compression engine error: {0}")]
    CompressionError(#[from] crate::compression::CompressionError),
    
    #[error("Legal analysis error: {0}")]
    AnalysisError(String),
    
    #[error("Precedent search error: {0}")]
    PrecedentError(String),
    
    #[error("Case law database error: {0}")]
    DatabaseError(String),
}

// Example usage function
impl LegalIntelligence {
    /// Generate 10-bit decision output for legal professionals
    pub fn to_decision_summary(&self) -> String {
        let precedent_count = self.legal_analysis.precedent_analysis.supporting_precedents.len();
        let risk_count = self.legal_analysis.risk_assessment.identified_risks.len();
        let needs_decision = !self.base_intelligence.decision_points.is_empty();

        format!(
            "{} precedents support argument. {} risk{} identified. {}",
            precedent_count,
            risk_count,
            if risk_count == 1 { "" } else { "s" },
            if needs_decision { "Decision needed." } else { "No immediate action required." }
        )
    }

    /// Generate 40-bit summary for legal review
    pub fn to_legal_summary(&self) -> Option<String> {
        self.base_intelligence.summary.as_ref().map(|summary| {
            format!(
                "Legal Analysis: {}\n• Precedent strength: {:.0}%\n• Risk level: {:?}\n• Action required: {}",
                summary.headline,
                self.legal_analysis.precedent_analysis.strength_score * 100.0,
                self.legal_analysis.risk_assessment.overall_risk_level,
                if summary.action_required { "Yes" } else { "No" }
            )
        })
    }
}
