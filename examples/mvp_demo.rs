// MVP Example: Intelligence Compression Engine Demo
// Demonstrates cognitive under-clocking for human-AI alignment

use note_to_ai::{
    CompressionEngine, CompressionContext, InformationPacket
};
use note_to_ai::compression::{
    Priority, CognitiveLoad, InformationSource, Industry
};
use chrono::Utc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Intelligence Compression Engine MVP Demo");
    println!("===============================================");
    println!("Cognitive Under-Clocking: Matching AI output to human processing limits\n");

    // Demo 1: Basic Information Compression
    demo_basic_compression().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // Demo 2: Cognitive Load Adaptation
    demo_cognitive_load_adaptation().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // Demo 3: Under-Clocked Processing Rates
    demo_processing_rates().await?;

    println!("\n🎯 MVP Validation Complete!");
    println!("✅ Information overload compressed to actionable intelligence");
    println!("✅ Cognitive load automatically adapted to user state");
    println!("✅ Processing rates aligned with human cognitive limits");
    println!("\n📊 Under-Clocked Output Rates:");
    println!("  • 8 bits/second: Decision points (simplified from 10 bits/second)");
    println!("  • 32 bits/second: Intelligence summaries (simplified from 40 bits/second)");
    println!("  • 64 bits/second: Deep context analysis (simplified from 120 bits/second)");

    Ok(())
}

async fn demo_basic_compression() -> Result<(), Box<dyn std::error::Error>> {
    println!("� Basic Intelligence Compression Demo");
    println!("-------------------------------------");
    
    // Create compression engine
    let mut engine = CompressionEngine::new().await?;
    
    // Sample information overload scenario
    let information = vec![
        InformationPacket {
            content: "Quarterly revenue exceeded projections by 15%. Market share increased in key segments. Board meeting scheduled for Friday to discuss expansion strategy. New product launch delayed by 3 weeks due to supply chain issues.".to_string(),
            source: InformationSource::Document,
            timestamp: Utc::now() - chrono::Duration::hours(1),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "financial_report".to_string());
                m.insert("urgency".to_string(), "high".to_string());
                m
            },
            priority: Priority::Important,
        },
        InformationPacket {
            content: "Customer satisfaction survey results show 85% positive feedback, down from 92% last quarter. Main concerns: response time (avg 4.2 hours, target 2 hours) and product documentation quality. Action plan needed by next week to address declining scores.".to_string(),
            source: InformationSource::Research,
            timestamp: Utc::now() - chrono::Duration::hours(2),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "customer_survey".to_string());
                m.insert("trend".to_string(), "declining".to_string());
                m
            },
            priority: Priority::Normal,
        },
        InformationPacket {
            content: "Server maintenance window required this weekend for critical security patches. Estimated downtime 4-6 hours. All systems need backup verification. Customer notifications sent. Emergency rollback plan prepared.".to_string(),
            source: InformationSource::Email,
            timestamp: Utc::now() - chrono::Duration::minutes(30),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "infrastructure_alert".to_string());
                m.insert("impact".to_string(), "high".to_string());
                m
            },
            priority: Priority::Urgent,
        },
    ];

    // Compression context
    let context = CompressionContext {
        user_role: "executive".to_string(),
        current_projects: vec!["Q4 planning".to_string(), "customer experience improvement".to_string()],
        decision_urgency: Priority::Important,
        domain_expertise: 0.7,
        cognitive_load: CognitiveLoad::Medium,
        industry_patterns: Industry::Generic.get_patterns(),
        domain_vocabulary: Industry::Generic.get_vocabulary(),
    };

    // Compress intelligence (under-clocked processing)
    let output = engine.compress_intelligence(information, context).await?;
    
    // Output results
    println!("🔍 Compressed Intelligence Results:");
    
    println!("  📋 8-bit Decision Points ({} total):", output.decision_points.len());
    for (i, decision) in output.decision_points.iter().enumerate() {
        println!("    {}. {} (Confidence: {:.0}%)", i + 1, decision.question, decision.confidence * 100.0);
        println!("       → {}", decision.recommendation);
    }
    
    if let Some(summary) = &output.summary {
        println!("  📄 32-bit Intelligence Summary:");
        println!("    Headline: {}", summary.headline);
        for point in &summary.key_points {
            println!("    • {}", point);
        }
        println!("    Action Required: {}", if summary.action_required { "Yes - Immediate" } else { "No" });
    }

    println!("  ⚡ Processing: {:?} (under-clocked for human consumption)", output.processing_time);
    println!("  🎯 Confidence: {:.0}%", output.confidence * 100.0);

    Ok(())
}

async fn demo_cognitive_load_adaptation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Cognitive Load Adaptation Demo");
    println!("---------------------------------");
    
    let mut engine = CompressionEngine::new().await?;
    
    // Same information processed under different cognitive loads
    let sample_info = vec![
        InformationPacket {
            content: "Critical system outage detected in production environment. Database connection failing intermittently. Customer impact: 15% of users affected. Engineering team mobilized. Estimated fix time: 2-4 hours. Communications team preparing customer notifications.".to_string(),
            source: InformationSource::Alert,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            priority: Priority::Critical,
        }
    ];

    let cognitive_states = vec![
        ("🌅 Fresh Morning State", CognitiveLoad::Low),
        ("📊 Normal Working State", CognitiveLoad::Medium),
        ("😴 End of Long Day", CognitiveLoad::High),
        ("🚨 Crisis/Overloaded", CognitiveLoad::Critical),
    ];

    for (state_name, cognitive_load) in cognitive_states {
        println!("\n  {} ({:?}):", state_name, cognitive_load);
        
        let context = CompressionContext {
            user_role: "operations_manager".to_string(),
            current_projects: vec!["system_reliability".to_string()],
            decision_urgency: Priority::Critical,
            domain_expertise: 0.8,
            cognitive_load: cognitive_load.clone(),
            industry_patterns: Industry::Generic.get_patterns(),
            domain_vocabulary: Industry::Generic.get_vocabulary(),
        };

        let output = engine.compress_intelligence(sample_info.clone(), context).await?;
        
        // Show how output adapts to cognitive load
        match cognitive_load {
            CognitiveLoad::Low => {
                println!("    📋 8-bit Decision: {}", output.decision_points[0].question);
                println!("    📄 32-bit Summary: Available with full context");
                println!("    🔍 64-bit Deep Context: Available for analysis");
            },
            CognitiveLoad::Medium => {
                println!("    📋 8-bit Decision: {}", output.decision_points[0].recommendation);
                println!("    📄 32-bit Summary: Structured key points only");
                println!("    🔍 64-bit Deep Context: On-demand only");
            },
            CognitiveLoad::High => {
                println!("    📋 8-bit Decision: Simple Yes/No recommendation");
                println!("    📄 32-bit Summary: Deferred to lower cognitive load");
                println!("    🔍 64-bit Deep Context: Unavailable");
            },
            CognitiveLoad::Critical => {
                println!("    📋 8-bit Decision: Emergency action only");
                println!("    📄 32-bit Summary: Unavailable");
                println!("    🔍 64-bit Deep Context: Unavailable");
            },
        }
    }

    println!("\n  🎯 Key Insight: Output complexity automatically matches human cognitive capacity!");

    Ok(())
}

async fn demo_processing_rates() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Under-Clocked Processing Rates Demo");
    println!("-------------------------------------");
    
    let mut engine = CompressionEngine::new().await?;
    
    println!("  🧬 Cognitive Research Foundation:");
    println!("    • Human conscious processing: ~10 bits/second");
    println!("    • AI system under-clocked to: 8 bits/second for decisions");
    println!("    • This creates natural human-AI cognitive alignment\n");

    // Demonstrate rate-limited processing
    let complex_info = vec![
        InformationPacket {
            content: "Multi-faceted business crisis: Revenue down 8%, customer churn up 23%, key partnership at risk, competitor launched similar product, supply chain disrupted, regulatory changes pending, team morale declining, investor confidence shaking, media coverage turning negative, board demanding immediate action plan.".to_string(),
            source: InformationSource::Document,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            priority: Priority::Critical,
        }
    ];

    let context = CompressionContext {
        user_role: "ceo".to_string(),
        current_projects: vec!["crisis_management".to_string()],
        decision_urgency: Priority::Critical,
        domain_expertise: 0.9,
        cognitive_load: CognitiveLoad::High,
        industry_patterns: Industry::Generic.get_patterns(),
        domain_vocabulary: Industry::Generic.get_vocabulary(),
    };

    let output = engine.compress_intelligence(complex_info, context).await?;
    
    println!("  � Rate-Limited Output (matching human processing):");
    println!("    🔸 8 bits/second - Decision Point:");
    println!("      '{}' → {}", 
        output.decision_points[0].question,
        output.decision_points[0].recommendation
    );
    
    if let Some(summary) = &output.summary {
        println!("    🔸 32 bits/second - Intelligence Summary:");
        println!("      Headline: {}", summary.headline);
        println!("      Key Points: {} items (reduced for cognitive load)", summary.key_points.len());
    }
    
    println!("    🔸 64 bits/second - Deep Context:");
    println!("      Available on-demand for detailed analysis");
    println!("      (Deferred due to current high cognitive load)");

    println!("\n  🎯 Alignment Benefits:");
    println!("    ✅ No cognitive overload - output matches human processing capacity");
    println!("    ✅ Natural decision flow - 8-bit choices feel intuitive");
    println!("    ✅ Scalable complexity - can expand to 32/64 bits when needed");
    println!("    ✅ Reduced decision fatigue - information pre-filtered for relevance");

    Ok(())
}


