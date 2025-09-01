// MVP Example: Intelligence Compression Engine Demo
// Demonstrates cross-industry application of the compression engine

use note_to_ai::{
    CompressionEngine, CompressionContext, InformationPacket, 
    LegalCompressionEngine, MedicalCompressionEngine,
};
use note_to_ai::compression::{
    Priority, CognitiveLoad, InformationSource, Industry, 
    legal::*, medical::*
};
use chrono::Utc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Intelligence Compression Engine MVP Demo");
    println!("===============================================");
    println!("Transforming 44TB of information into 10-bit decisions\n");

    // Demo 1: Legal Intelligence Compression
    demo_legal_compression().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // Demo 2: Medical Intelligence Compression  
    demo_medical_compression().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // Demo 3: Generic Cross-Industry Compression
    demo_generic_compression().await?;

    println!("\n🎯 MVP Validation Complete!");
    println!("✅ Legal: Case analysis compressed to 10-bit decisions");
    println!("✅ Medical: Clinical data compressed to diagnostic insights");
    println!("✅ Generic: Information overload compressed to actionable intelligence");
    println!("\n📊 Cognitive Load Matching: All outputs optimized for human 10 bits/second processing");

    Ok(())
}

async fn demo_legal_compression() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 Legal Intelligence Engine Demo");
    println!("----------------------------------");
    
    // Create legal compression engine
    let mut legal_engine = LegalCompressionEngine::new().await?;
    
    // Sample legal documents
    let legal_documents = vec![
        LegalDocument {
            document_type: LegalDocumentType::CaseLaw,
            content: "Smith v. Jones establishes precedent for contract interpretation in cases involving ambiguous terms. Court held that extrinsic evidence may be considered when contract language is ambiguous.".to_string(),
            date: Utc::now() - chrono::Duration::days(30),
            source: "Westlaw".to_string(),
            parties: vec!["Smith".to_string(), "Jones".to_string()],
            jurisdiction: Some("9th Circuit".to_string()),
            case_number: Some("21-cv-1234".to_string()),
        },
        LegalDocument {
            document_type: LegalDocumentType::Motion,
            content: "Motion for summary judgment filed. Defendant argues no genuine issue of material fact exists regarding contract formation. Deadline for response is Friday.".to_string(),
            date: Utc::now() - chrono::Duration::hours(2),
            source: "Court Filing".to_string(), 
            parties: vec!["Plaintiff Corp".to_string(), "Defendant LLC".to_string()],
            jurisdiction: Some("Superior Court".to_string()),
            case_number: Some("CV-2024-5678".to_string()),
        },
    ];

    // Legal context
    let legal_context = LegalContext {
        attorney_role: "plaintiff attorney".to_string(),
        practice_areas: vec!["contract law".to_string(), "business litigation".to_string()],
        active_cases: vec!["CV-2024-5678".to_string()],
        court_deadlines: vec![
            CourtDeadline {
                case: "CV-2024-5678".to_string(),
                deadline_type: "motion response".to_string(),
                due_date: Utc::now() + chrono::Duration::days(3),
                priority: Priority::Urgent,
            }
        ],
        experience_level: 0.8,
        current_workload: CognitiveLoad::Medium,
        jurisdiction: "California".to_string(),
    };

    // Compress legal intelligence
    let legal_intelligence = legal_engine.analyze_legal_case(legal_documents, legal_context).await?;
    
    // Output results
    println!("🔍 Legal Analysis Results:");
    println!("  10-bit Decision: {}", legal_intelligence.to_decision_summary());
    
    if let Some(summary) = legal_intelligence.to_legal_summary() {
        println!("  40-bit Summary:\n{}", summary);
    }

    println!("  Precedents Found: {}", legal_intelligence.legal_analysis.precedent_analysis.supporting_precedents.len());
    println!("  Risks Identified: {}", legal_intelligence.legal_analysis.risk_assessment.identified_risks.len());
    println!("  Strategies Available: {}", legal_intelligence.legal_analysis.recommended_strategies.len());

    Ok(())
}

async fn demo_medical_compression() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏥 Medical Intelligence Engine Demo");
    println!("-----------------------------------");
    
    // Create medical compression engine
    let mut medical_engine = MedicalCompressionEngine::new().await?;
    
    // Sample medical records
    let medical_records = vec![
        MedicalRecord {
            record_type: MedicalRecordType::ChiefComplaint,
            content: "Patient presents with chest pain, shortness of breath, and diaphoresis. Pain started 2 hours ago, radiates to left arm.".to_string(),
            timestamp: Utc::now() - chrono::Duration::minutes(30),
            provider: "Dr. Smith".to_string(),
            patient_id: "PT-001".to_string(),
            vitals: Some(VitalSigns {
                temperature_f: Some(98.6),
                blood_pressure_systolic: Some(160),
                blood_pressure_diastolic: Some(95),
                heart_rate: Some(110),
                respiratory_rate: Some(22),
                oxygen_saturation: Some(94.0),
                weight_lbs: Some(180.0),
                height_inches: Some(70.0),
            }),
            medications: vec![
                Medication {
                    name: "Aspirin".to_string(),
                    dosage: "81mg".to_string(),
                    frequency: "daily".to_string(),
                    route: "oral".to_string(),
                    start_date: Utc::now() - chrono::Duration::days(365),
                    end_date: None,
                }
            ],
            allergies: vec!["Penicillin".to_string()],
        },
        MedicalRecord {
            record_type: MedicalRecordType::LabResults,
            content: "Troponin elevated at 2.1 ng/mL (normal <0.04). ECG shows ST elevation in leads II, III, aVF.".to_string(),
            timestamp: Utc::now() - chrono::Duration::minutes(15),
            provider: "Lab Tech".to_string(),
            patient_id: "PT-001".to_string(),
            vitals: None,
            medications: vec![],
            allergies: vec![],
        },
    ];

    // Clinical context
    let clinical_context = ClinicalContext {
        provider_role: "emergency physician".to_string(),
        specialty: Some("emergency medicine".to_string()),
        active_cases: vec!["PT-001".to_string()],
        clinical_workload: CognitiveLoad::High,
        experience_level: 0.9,
        current_shift_hours: 8.0,
        emergency_status: true,
    };

    // Compress medical intelligence
    let medical_intelligence = medical_engine.analyze_patient_case(medical_records, clinical_context).await?;
    
    // Output results
    println!("🔍 Medical Analysis Results:");
    println!("  10-bit Decision: {}", medical_intelligence.to_clinical_summary());
    
    if let Some(summary) = medical_intelligence.to_medical_summary() {
        println!("  40-bit Summary:\n{}", summary);
    }

    println!("  Critical Alerts: {}", if medical_intelligence.has_critical_alerts() { "YES - Immediate attention required" } else { "None" });
    println!("  Treatment Options: {}", medical_intelligence.medical_analysis.treatment_recommendations.len());
    println!("  Safety Score: {:.0}%", medical_intelligence.medical_analysis.safety_assessment.overall_safety_score * 100.0);

    Ok(())
}

async fn demo_generic_compression() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Generic Cross-Industry Compression Demo");
    println!("------------------------------------------");
    
    // Create generic compression engine
    let mut engine = CompressionEngine::new().await?;
    
    // Sample information from various sources
    let information = vec![
        InformationPacket {
            content: "Quarterly revenue exceeded projections by 15%. Market share increased in key segments. Board meeting scheduled for Friday to discuss expansion strategy.".to_string(),
            source: InformationSource::Document,
            timestamp: Utc::now() - chrono::Duration::hours(1),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source_type".to_string(), "financial_report".to_string());
                m.insert("department".to_string(), "finance".to_string());
                m
            },
            priority: Priority::Important,
        },
        InformationPacket {
            content: "Customer satisfaction survey results: 85% positive feedback. Main concerns: response time and product documentation. Action plan needed by next week.".to_string(),
            source: InformationSource::Research,
            timestamp: Utc::now() - chrono::Duration::hours(3),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source_type".to_string(), "customer_survey".to_string());
                m.insert("department".to_string(), "customer_success".to_string());
                m
            },
            priority: Priority::Normal,
        },
        InformationPacket {
            content: "Server maintenance window required this weekend. Estimated downtime 4 hours. All critical systems need backup verification before proceeding.".to_string(),
            source: InformationSource::Email,
            timestamp: Utc::now() - chrono::Duration::minutes(45),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source_type".to_string(), "system_maintenance".to_string());
                m.insert("department".to_string(), "infrastructure".to_string());
                m
            },
            priority: Priority::Urgent,
        },
    ];

    // Generic compression context
    let context = CompressionContext {
        user_role: "executive".to_string(),
        current_projects: vec!["Q4 planning".to_string(), "customer experience".to_string()],
        decision_urgency: Priority::Important,
        domain_expertise: 0.7,
        cognitive_load: CognitiveLoad::Medium,
        industry_patterns: Industry::Generic.get_patterns(),
        domain_vocabulary: Industry::Generic.get_vocabulary(),
    };

    // Compress intelligence
    let output = engine.compress_intelligence(information, context).await?;
    
    // Output results
    println!("🔍 Generic Compression Results:");
    
    println!("  Decision Points ({} total):", output.decision_points.len());
    for (i, decision) in output.decision_points.iter().enumerate() {
        println!("    {}. {} (Confidence: {:.0}%)", i + 1, decision.question, decision.confidence * 100.0);
        println!("       Options: {}", decision.options.join(" / "));
        println!("       Recommendation: {}", decision.recommendation);
    }
    
    if let Some(summary) = &output.summary {
        println!("  40-bit Summary:");
        println!("    • {}", summary.headline);
        for point in &summary.key_points {
            println!("    • {}", point);
        }
        println!("    • Action Required: {}", if summary.action_required { "Yes" } else { "No" });
    }

    println!("  Processing Time: {:?}", output.processing_time);
    println!("  Overall Confidence: {:.0}%", output.confidence * 100.0);

    Ok(())
}

// Helper function to demonstrate cognitive load matching
fn demonstrate_cognitive_load_adaptation() {
    println!("\n🧠 Cognitive Load Adaptation Demo");
    println!("--------------------------------");
    
    let scenarios = vec![
        ("Fresh start of day", CognitiveLoad::Low, "Complex analysis with deep context available"),
        ("Normal working state", CognitiveLoad::Medium, "Structured summaries with key decisions"),
        ("End of long day", CognitiveLoad::High, "Simple decisions only, minimal cognitive overhead"),
        ("Crisis/Overloaded", CognitiveLoad::Critical, "Emergency decisions only, defer non-urgent items"),
    ];

    for (scenario, load, output_description) in scenarios {
        println!("  {} ({:?}): {}", scenario, load, output_description);
    }
    
    println!("\n📊 Output automatically adapts to match human cognitive processing limits:");
    println!("  • 10 bits/second: Decision points (Yes/No/More Info)");
    println!("  • 40 bits/second: Structured summaries");  
    println!("  • 120 bits/second: Deep context analysis");
}
