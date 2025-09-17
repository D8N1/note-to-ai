use anyhow::Result;
use note_to_ai::compression::{CompressionEngine, InformationPacket, InformationSource, Priority, CompressionContext, CognitiveLoad, IndustryPattern};
use chrono::Utc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the compression engine
    let mut engine = CompressionEngine::new().await?;

    // Create some sample information packets
    let packets = vec![
        InformationPacket {
            content: "The patient has a history of hypertension and diabetes.".to_string(),
            source: InformationSource::Voice,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            priority: Priority::Important,
        },
        InformationPacket {
            content: "Blood pressure reading: 140/90 mmHg".to_string(),
            source: InformationSource::Document,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            priority: Priority::Normal,
        },
        InformationPacket {
            content: "Patient reports feeling dizzy in the mornings".to_string(),
            source: InformationSource::Voice,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            priority: Priority::Normal,
        },
    ];

    // Create compression context
    let context = CompressionContext {
        user_role: "doctor".to_string(),
        current_projects: vec!["Patient Care".to_string()],
        decision_urgency: Priority::Important,
        domain_expertise: 0.8,
        cognitive_load: CognitiveLoad::Medium,
        industry_patterns: vec![
            IndustryPattern {
                name: "Medical Diagnosis".to_string(),
                description: "Pattern for medical diagnosis workflow".to_string(),
                triggers: vec!["symptoms".to_string(), "vital signs".to_string()],
                actions: vec!["assess condition".to_string(), "recommend treatment".to_string()],
            }
        ],
        domain_vocabulary: HashMap::from([
            ("BP".to_string(), "Blood Pressure".to_string()),
            ("mmHg".to_string(), "millimeters of mercury".to_string()),
        ]),
    };

    // Compress the information into intelligence
    let result = engine.compress_intelligence(packets, context).await?;

    println!("🧠 Intelligence Compression Engine Demo");
    println!("======================================");
    println!();
    println!("📊 Compression Results:");
    println!("• Processing time: {:?}", result.processing_time);
    println!("• Confidence: {:.1}%", result.confidence * 100.0);
    println!();
    println!("🎯 Decision Points:");
    for (i, decision) in result.decision_points.iter().enumerate() {
        println!("  {}. {}", i + 1, decision.question);
        for (j, option) in decision.options.iter().enumerate() {
            let marker = if j == 0 { "→" } else { " " };
            println!("     {} {}", marker, option);
        }
        println!("     Recommendation: {}", decision.recommendation);
        println!();
    }
    
    if let Some(summary) = &result.summary {
        println!("✨ Summary:");
        println!("• Headline: {}", summary.headline);
        println!("• Key Points:");
        for point in &summary.key_points {
            println!("  → {}", point);
        }
        println!("• Action Required: {}", if summary.action_required { "Yes" } else { "No" });
    }

    if let Some(deep_context) = &result.deep_context {
        println!();
        println!("🔍 Deep Context:");
        println!("• Analysis: {}", deep_context.analysis);
        println!("• Evidence:");
        for evidence in &deep_context.supporting_evidence {
            println!("  → {}", evidence);
        }
        println!("• Recommendations:");
        for rec in &deep_context.recommendations {
            println!("  → {}", rec);
        }
    }

    Ok(())
}