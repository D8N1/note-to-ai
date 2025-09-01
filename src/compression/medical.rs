// Medical Intelligence Engine
// Specialized compression for medical professionals
// Output: "Primary diagnosis 85% confident. Two treatments. One contraindication."

use super::{CompressionEngine, InformationPacket, CompressionContext, CognitiveOutput};
use super::{Industry, Priority, CognitiveLoad};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct MedicalCompressionEngine {
    core: CompressionEngine,
    medical_patterns: MedicalPatternRecognizer,
    diagnostic_engine: DiagnosticEngine,
    treatment_protocols: TreatmentProtocolEngine,
    drug_interactions: DrugInteractionChecker,
}

impl MedicalCompressionEngine {
    pub async fn new() -> Result<Self, MedicalCompressionError> {
        Ok(Self {
            core: CompressionEngine::new().await?,
            medical_patterns: MedicalPatternRecognizer::new(),
            diagnostic_engine: DiagnosticEngine::new().await?,
            treatment_protocols: TreatmentProtocolEngine::new().await?,
            drug_interactions: DrugInteractionChecker::new().await?,
        })
    }

    /// Compress medical information into clinical intelligence
    pub async fn analyze_patient_case(
        &mut self,
        medical_data: Vec<MedicalRecord>,
        clinical_context: ClinicalContext,
    ) -> Result<MedicalIntelligence, MedicalCompressionError> {
        // Convert medical records to information packets
        let information: Vec<InformationPacket> = medical_data.into_iter()
            .map(|record| record.into_information_packet())
            .collect();

        // Create medical-specific compression context
        let compression_context = CompressionContext {
            user_role: clinical_context.provider_role.clone(),
            current_projects: clinical_context.active_cases.clone(),
            decision_urgency: clinical_context.clinical_priority(),
            domain_expertise: clinical_context.experience_level,
            cognitive_load: clinical_context.clinical_workload.clone(),
            industry_patterns: Industry::Medical.get_patterns(),
            domain_vocabulary: Industry::Medical.get_vocabulary(),
        };

        // Perform core compression
        let base_output = self.core.compress_for_industry(
            information,
            Industry::Medical,
            compression_context,
        ).await?;

        // Add medical-specific analysis
        let medical_analysis = self.perform_medical_analysis(&base_output, &clinical_context).await?;

        Ok(MedicalIntelligence {
            base_intelligence: base_output,
            medical_analysis,
            clinical_context,
        })
    }

    async fn perform_medical_analysis(
        &self,
        base_output: &CognitiveOutput,
        context: &ClinicalContext,
    ) -> Result<MedicalAnalysis, MedicalCompressionError> {
        let mut analysis = MedicalAnalysis::default();

        // Generate diagnostic suggestions
        analysis.diagnostic_assessment = self.diagnostic_engine
            .assess_differential_diagnosis(&base_output, context).await?;

        // Analyze treatment options
        analysis.treatment_recommendations = self.treatment_protocols
            .suggest_treatments(&base_output, context).await?;

        // Check for drug interactions and contraindications
        analysis.safety_assessment = self.drug_interactions
            .check_safety_concerns(&base_output, context).await?;

        // Identify urgent medical issues
        analysis.urgent_concerns = self.medical_patterns
            .identify_urgent_issues(&base_output, context).await?;

        Ok(analysis)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalRecord {
    pub record_type: MedicalRecordType,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub provider: String,
    pub patient_id: String,
    pub vitals: Option<VitalSigns>,
    pub medications: Vec<Medication>,
    pub allergies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MedicalRecordType {
    ChiefComplaint,
    HistoryOfPresentIllness,
    PhysicalExam,
    LabResults,
    ImagingResults,
    Assessment,
    Plan,
    ProgressNote,
    DischargeNote,
    Prescription,
    Consultation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSigns {
    pub temperature_f: Option<f32>,
    pub blood_pressure_systolic: Option<i32>,
    pub blood_pressure_diastolic: Option<i32>,
    pub heart_rate: Option<i32>,
    pub respiratory_rate: Option<i32>,
    pub oxygen_saturation: Option<f32>,
    pub weight_lbs: Option<f32>,
    pub height_inches: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub route: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl MedicalRecord {
    fn into_information_packet(self) -> InformationPacket {
        let priority = match self.record_type {
            MedicalRecordType::Assessment | MedicalRecordType::Plan => Priority::Urgent,
            MedicalRecordType::LabResults | MedicalRecordType::ImagingResults => Priority::Important,
            MedicalRecordType::PhysicalExam | MedicalRecordType::ChiefComplaint => Priority::Important,
            _ => Priority::Normal,
        };

        let mut metadata = HashMap::new();
        metadata.insert("record_type".to_string(), format!("{:?}", self.record_type));
        metadata.insert("provider".to_string(), self.provider);
        metadata.insert("patient_id".to_string(), self.patient_id);
        
        if let Some(vitals) = &self.vitals {
            if let Some(temp) = vitals.temperature_f {
                metadata.insert("temperature".to_string(), temp.to_string());
            }
            if let Some(bp_sys) = vitals.blood_pressure_systolic {
                metadata.insert("bp_systolic".to_string(), bp_sys.to_string());
            }
        }

        InformationPacket {
            content: self.content,
            source: crate::compression::InformationSource::Document,
            timestamp: self.timestamp,
            metadata,
            priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalContext {
    pub provider_role: String,          // "physician", "nurse", "pa", "resident"
    pub specialty: Option<String>,      // "cardiology", "emergency", etc.
    pub active_cases: Vec<String>,      // Patient IDs or case numbers
    pub clinical_workload: CognitiveLoad,
    pub experience_level: f32,          // 0.0-1.0
    pub current_shift_hours: f32,       // Fatigue factor
    pub emergency_status: bool,         // In emergency situation
}

impl ClinicalContext {
    fn clinical_priority(&self) -> Priority {
        if self.emergency_status {
            Priority::Urgent
        } else if self.current_shift_hours > 12.0 {
            Priority::Important // Tired, need clear decisions
        } else {
            Priority::Normal
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalIntelligence {
    pub base_intelligence: CognitiveOutput,
    pub medical_analysis: MedicalAnalysis,
    pub clinical_context: ClinicalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MedicalAnalysis {
    pub diagnostic_assessment: DiagnosticAssessment,
    pub treatment_recommendations: Vec<TreatmentRecommendation>,
    pub safety_assessment: SafetyAssessment,
    pub urgent_concerns: Vec<UrgentConcern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticAssessment {
    pub primary_diagnosis: Option<Diagnosis>,
    pub differential_diagnoses: Vec<Diagnosis>,
    pub confidence_level: f32,
    pub additional_testing_needed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    pub icd_code: Option<String>,
    pub name: String,
    pub confidence: f32,          // 0.0-1.0
    pub supporting_evidence: Vec<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Mild,
    Moderate,
    Severe,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentRecommendation {
    pub treatment_type: TreatmentType,
    pub description: String,
    pub evidence_level: EvidenceLevel,
    pub contraindications: Vec<String>,
    pub expected_outcome: String,
    pub monitoring_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreatmentType {
    Medication,
    Procedure,
    Lifestyle,
    Monitoring,
    Referral,
    Surgery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceLevel {
    HighQuality,    // Multiple RCTs
    Moderate,       // Limited RCTs or observational
    LowQuality,     // Expert opinion or case series
    Experimental,   // Investigational
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafetyAssessment {
    pub drug_interactions: Vec<DrugInteraction>,
    pub allergic_reactions: Vec<AllergyRisk>,
    pub contraindications: Vec<Contraindication>,
    pub overall_safety_score: f32,  // 0.0-1.0, higher is safer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugInteraction {
    pub drug_a: String,
    pub drug_b: String,
    pub interaction_type: InteractionType,
    pub severity: Severity,
    pub description: String,
    pub management: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Pharmacokinetic,  // Absorption, distribution, metabolism, excretion
    Pharmacodynamic,  // Additive, synergistic, antagonistic effects
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergyRisk {
    pub allergen: String,
    pub reaction_type: String,
    pub severity: Severity,
    pub cross_reactivity: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contraindication {
    pub treatment: String,
    pub contraindication_type: String,
    pub reason: String,
    pub absolute: bool,  // true = absolute, false = relative
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrgentConcern {
    pub concern_type: String,
    pub description: String,
    pub time_sensitive: bool,
    pub immediate_action: String,
    pub escalation_needed: bool,
}

// Medical pattern recognition system
struct MedicalPatternRecognizer;

impl MedicalPatternRecognizer {
    fn new() -> Self {
        Self
    }

    async fn identify_urgent_issues(
        &self,
        output: &CognitiveOutput,
        context: &ClinicalContext,
    ) -> Result<Vec<UrgentConcern>, MedicalCompressionError> {
        let mut concerns = Vec::new();

        // Check for critical vital signs
        for decision in &output.decision_points {
            if decision.question.to_lowercase().contains("fever") {
                concerns.push(UrgentConcern {
                    concern_type: "High Fever".to_string(),
                    description: "Elevated temperature requiring assessment".to_string(),
                    time_sensitive: true,
                    immediate_action: "Check vitals, consider antipyretics".to_string(),
                    escalation_needed: false,
                });
            }

            if decision.question.to_lowercase().contains("chest pain") {
                concerns.push(UrgentConcern {
                    concern_type: "Chest Pain".to_string(),
                    description: "Potential cardiac or pulmonary emergency".to_string(),
                    time_sensitive: true,
                    immediate_action: "ECG, cardiac enzymes, vital signs".to_string(),
                    escalation_needed: true,
                });
            }
        }

        // Check for drug allergy alerts
        if let Some(summary) = &output.summary {
            if summary.headline.to_lowercase().contains("allergy") {
                concerns.push(UrgentConcern {
                    concern_type: "Allergy Alert".to_string(),
                    description: "Potential allergic reaction or contraindication".to_string(),
                    time_sensitive: true,
                    immediate_action: "Review allergy history and current medications".to_string(),
                    escalation_needed: false,
                });
            }
        }

        Ok(concerns)
    }
}

// Diagnostic engine for differential diagnosis
struct DiagnosticEngine;

impl DiagnosticEngine {
    async fn new() -> Result<Self, MedicalCompressionError> {
        Ok(Self)
    }

    async fn assess_differential_diagnosis(
        &self,
        output: &CognitiveOutput,
        context: &ClinicalContext,
    ) -> Result<DiagnosticAssessment, MedicalCompressionError> {
        // In a real implementation, this would use medical knowledge bases
        // For MVP, provide simplified diagnostic suggestions
        
        let mut differential_diagnoses = Vec::new();
        
        // Analyze symptoms mentioned in decision points
        for decision in &output.decision_points {
            if decision.question.to_lowercase().contains("fever") {
                differential_diagnoses.push(Diagnosis {
                    icd_code: Some("R50.9".to_string()),
                    name: "Fever".to_string(),
                    confidence: decision.confidence,
                    supporting_evidence: vec!["Elevated temperature reported".to_string()],
                    severity: Severity::Moderate,
                });
            }

            if decision.question.to_lowercase().contains("pain") {
                differential_diagnoses.push(Diagnosis {
                    icd_code: Some("R52".to_string()),
                    name: "Pain, unspecified".to_string(),
                    confidence: decision.confidence * 0.8,
                    supporting_evidence: vec!["Pain symptoms reported".to_string()],
                    severity: Severity::Mild,
                });
            }
        }

        let primary_diagnosis = differential_diagnoses.first().cloned();
        let confidence_level = primary_diagnosis
            .as_ref()
            .map(|d| d.confidence)
            .unwrap_or(0.0);

        Ok(DiagnosticAssessment {
            primary_diagnosis,
            differential_diagnoses,
            confidence_level,
            additional_testing_needed: vec![
                "Complete Blood Count".to_string(),
                "Basic Metabolic Panel".to_string(),
            ],
        })
    }
}

// Treatment protocol engine
struct TreatmentProtocolEngine;

impl TreatmentProtocolEngine {
    async fn new() -> Result<Self, MedicalCompressionError> {
        Ok(Self)
    }

    async fn suggest_treatments(
        &self,
        output: &CognitiveOutput,
        context: &ClinicalContext,
    ) -> Result<Vec<TreatmentRecommendation>, MedicalCompressionError> {
        let mut recommendations = Vec::new();

        // Analyze decision points for treatment opportunities
        for decision in &output.decision_points {
            if decision.question.to_lowercase().contains("fever") {
                recommendations.push(TreatmentRecommendation {
                    treatment_type: TreatmentType::Medication,
                    description: "Antipyretic therapy (acetaminophen or ibuprofen)".to_string(),
                    evidence_level: EvidenceLevel::HighQuality,
                    contraindications: vec!["Liver disease (acetaminophen)".to_string()],
                    expected_outcome: "Temperature reduction within 1-2 hours".to_string(),
                    monitoring_requirements: vec!["Temperature every 4 hours".to_string()],
                });
            }

            if decision.question.to_lowercase().contains("pain") {
                recommendations.push(TreatmentRecommendation {
                    treatment_type: TreatmentType::Medication,
                    description: "Analgesic therapy based on pain severity".to_string(),
                    evidence_level: EvidenceLevel::HighQuality,
                    contraindications: vec!["Known allergy to analgesics".to_string()],
                    expected_outcome: "Pain reduction within 30-60 minutes".to_string(),
                    monitoring_requirements: vec!["Pain scale assessment every 2 hours".to_string()],
                });
            }
        }

        // Always include monitoring recommendation
        recommendations.push(TreatmentRecommendation {
            treatment_type: TreatmentType::Monitoring,
            description: "Continue monitoring vital signs and symptoms".to_string(),
            evidence_level: EvidenceLevel::HighQuality,
            contraindications: vec![],
            expected_outcome: "Early detection of changes in condition".to_string(),
            monitoring_requirements: vec!["Vital signs every 4 hours".to_string()],
        });

        Ok(recommendations)
    }
}

// Drug interaction checker
struct DrugInteractionChecker;

impl DrugInteractionChecker {
    async fn new() -> Result<Self, MedicalCompressionError> {
        Ok(Self)
    }

    async fn check_safety_concerns(
        &self,
        output: &CognitiveOutput,
        context: &ClinicalContext,
    ) -> Result<SafetyAssessment, MedicalCompressionError> {
        // In a real implementation, this would check against drug databases
        // For MVP, provide basic safety assessment
        
        let drug_interactions = Vec::new(); // Would be populated from drug database
        let allergic_reactions = Vec::new(); // Would be populated from allergy database
        let contraindications = Vec::new();  // Would be populated from contraindication database

        let overall_safety_score = if drug_interactions.is_empty() && 
                                     allergic_reactions.is_empty() && 
                                     contraindications.is_empty() {
            0.9 // High safety
        } else {
            0.6 // Medium safety
        };

        Ok(SafetyAssessment {
            drug_interactions,
            allergic_reactions,
            contraindications,
            overall_safety_score,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MedicalCompressionError {
    #[error("Compression engine error: {0}")]
    CompressionError(#[from] crate::compression::CompressionError),
    
    #[error("Diagnostic analysis error: {0}")]
    DiagnosticError(String),
    
    #[error("Treatment protocol error: {0}")]
    TreatmentError(String),
    
    #[error("Drug interaction database error: {0}")]
    DrugDatabaseError(String),
    
    #[error("Medical knowledge base error: {0}")]
    KnowledgeBaseError(String),
}

// Example usage functions
impl MedicalIntelligence {
    /// Generate 10-bit decision output for medical professionals
    pub fn to_clinical_summary(&self) -> String {
        let diagnosis_confidence = self.medical_analysis.diagnostic_assessment.confidence_level;
        let treatment_count = self.medical_analysis.treatment_recommendations.len();
        let contraindication_count = self.medical_analysis.safety_assessment.contraindications.len();

        format!(
            "Primary diagnosis {:.0}% confident. {} treatment{}. {} contraindication{}.",
            diagnosis_confidence * 100.0,
            treatment_count,
            if treatment_count == 1 { "" } else { "s" },
            contraindication_count,
            if contraindication_count == 1 { "" } else { "s" }
        )
    }

    /// Generate 40-bit summary for clinical review
    pub fn to_medical_summary(&self) -> Option<String> {
        self.base_intelligence.summary.as_ref().map(|summary| {
            let primary_dx = self.medical_analysis.diagnostic_assessment.primary_diagnosis
                .as_ref()
                .map(|d| d.name.as_str())
                .unwrap_or("No diagnosis");
            
            format!(
                "Clinical Assessment: {}\n• Primary diagnosis: {}\n• Safety score: {:.0}%\n• Urgent concerns: {}",
                summary.headline,
                primary_dx,
                self.medical_analysis.safety_assessment.overall_safety_score * 100.0,
                self.medical_analysis.urgent_concerns.len()
            )
        })
    }

    /// Check for critical alerts requiring immediate attention
    pub fn has_critical_alerts(&self) -> bool {
        self.medical_analysis.urgent_concerns.iter()
            .any(|concern| concern.time_sensitive || concern.escalation_needed)
    }
}
