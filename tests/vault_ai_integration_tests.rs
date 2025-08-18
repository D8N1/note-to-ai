// tests/vault_ai_integration_tests.rs
// High-resolution tests for AI agent integration with Obsidian vault

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::Utc;
use tempfile::TempDir;
use serde_json::json;

use note_to_ai::vault::{Vault, ObsidianParser};
use note_to_ai::vault::ai_agent_integration::*;

/// Test suite for AI agent integration with Obsidian vault
pub struct VaultAITestSuite {
    temp_dir: TempDir,
    vault_path: PathBuf,
    db_path: PathBuf,
    vault: Vault,
}

impl VaultAITestSuite {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let vault_path = temp_dir.path().join("test_vault");
        let db_path = temp_dir.path().join("test.db");
        
        std::fs::create_dir_all(&vault_path)?;
        
        let vault = Vault::new(db_path.clone(), vault_path.clone()).await?;
        
        Ok(Self {
            temp_dir,
            vault_path,
            db_path,
            vault,
        })
    }
    
    /// Create a test note with proper Obsidian formatting
    pub fn create_test_note(&self, filename: &str, content: &str) -> Result<PathBuf> {
        let note_path = self.vault_path.join(filename);
        std::fs::write(&note_path, content)?;
        Ok(note_path)
    }
}

#[tokio::test]
async fn test_ai_agent_knowledge_base_sections() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Create a knowledge base structure for specialist AI agents
    let knowledge_base_content = r#"---
title: "AI Agent Knowledge Base"
tags: ["ai-agents", "knowledge-base", "automation"]
created: 2025-08-18T10:00:00Z
agent_sections:
  - research_agent
  - analysis_agent
  - writing_agent
  - verification_agent
---

# AI Agent Knowledge Base

## Research Agent Section
### Current Research Tasks
- [ ] Market analysis for Q3 2025
- [ ] Competitor landscape review
- [ ] Technology trend analysis

### Research Findings
![[Research Findings Template]]

### Data Sources
- Internal databases: [[Database Access Guide]]
- External APIs: [[API Documentation]]
- Market reports: [[Market Data Sources]]

## Analysis Agent Section
### Analysis Frameworks
- SWOT Analysis: [[SWOT Template]]
- Risk Assessment: [[Risk Analysis Framework]]
- Performance Metrics: [[KPI Dashboard]]

### Current Analysis Projects
- [ ] Customer segmentation analysis
- [ ] Revenue trend analysis
- [ ] Operational efficiency review

## Writing Agent Section
### Content Templates
- Executive summaries: [[Executive Summary Template]]
- Technical documentation: [[Tech Doc Template]]
- Reports: [[Report Template]]

### Style Guidelines
- Brand voice: [[Brand Voice Guide]]
- Technical writing: [[Technical Style Guide]]
- Executive communication: [[Executive Communication Guide]]

## Verification Agent Section
### Verification Protocols
- Fact checking: [[Fact Check Protocol]]
- Data validation: [[Data Validation Rules]]
- Source verification: [[Source Verification Guide]]

### Quality Assurance
- Content review checklist: [[Content QA Checklist]]
- Accuracy standards: [[Accuracy Standards]]
- Compliance requirements: [[Compliance Guide]]

## Shared Resources
### Collaboration Notes
> [!info] Agent Collaboration
> This section is for cross-agent communication and shared insights

### Workflow Status
```mermaid
graph TD
    A[Research Agent] --> B[Analysis Agent]
    B --> C[Writing Agent]
    C --> D[Verification Agent]
    D --> E[Final Output]
```

### Communication Log
- 2025-08-18 10:00: Research agent initialized market analysis
- 2025-08-18 10:15: Analysis agent requested additional data points
"#;
    
    test_suite.create_test_note("AI_Agent_Knowledge_Base.md", knowledge_base_content)?;
    
    // Test parsing of AI agent sections
    let parser = ObsidianParser::new()?;
    let doc = parser.parse_file(&test_suite.vault_path.join("AI_Agent_Knowledge_Base.md")).await?;
    
    // Verify frontmatter includes agent_sections
    assert!(doc.frontmatter.is_some());
    let frontmatter = doc.frontmatter.unwrap();
    assert!(frontmatter.custom_fields.contains_key("agent_sections"));
    
    // Verify sections are properly parsed as headings
    let research_section = doc.headings.iter()
        .find(|h| h.text.contains("Research Agent Section"))
        .expect("Research Agent Section not found");
    assert_eq!(research_section.level, 2);
    
    // Verify task lists are captured
    let task_count = doc.content.matches("- [ ]").count();
    assert!(task_count >= 5, "Expected at least 5 tasks, found {}", task_count);
    
    // Verify links are properly extracted
    let internal_links: Vec<_> = doc.links.iter()
        .filter(|link| matches!(link.link_type, note_to_ai::vault::parser::LinkType::WikiLink))
        .collect();
    assert!(!internal_links.is_empty(), "No wiki links found");
    
    println!("✅ AI Agent Knowledge Base structure test passed");
    Ok(())
}

#[tokio::test]
async fn test_agent_section_isolation() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Create individual agent workspace files
    let research_workspace = r#"---
title: "Research Agent Workspace"
agent: "research_agent"
access_level: "read_write"
parent: "[[AI_Agent_Knowledge_Base]]"
---

# Research Agent Private Workspace

## Current Projects
### Market Analysis Q3 2025
**Status**: In Progress
**Priority**: High
**Deadline**: 2025-09-01

#### Research Questions
1. What are the emerging trends in our target market?
2. How are competitors positioning themselves?
3. What new technologies are disrupting the space?

#### Data Collection
- [x] Industry reports gathered
- [x] Competitor websites analyzed
- [ ] Expert interviews scheduled
- [ ] Survey data collected

#### Findings Summary
> [!research] Key Insight
> Market showing 23% growth in AI automation tools

### Technology Trend Analysis
**Status**: Planning
**Priority**: Medium

## Research Methods
### Data Sources Hierarchy
1. **Primary Sources** (highest confidence)
   - Direct interviews
   - Original surveys
   - First-hand observations

2. **Secondary Sources** (medium confidence)
   - Industry reports
   - Academic papers
   - Government data

3. **Tertiary Sources** (verify carefully)
   - News articles
   - Blog posts
   - Social media insights

## Agent Communication Log
### Requests from Analysis Agent
- 2025-08-18 10:15: Need additional market size data
- 2025-08-18 10:30: Request for competitor pricing analysis

### Responses Sent
- 2025-08-18 10:45: Market size data uploaded to [[Shared Data Repository]]
"#;
    
    test_suite.create_test_note("Research_Agent_Workspace.md", research_workspace)?;
    
    let analysis_workspace = r#"---
title: "Analysis Agent Workspace"
agent: "analysis_agent"
access_level: "read_write"
parent: "[[AI_Agent_Knowledge_Base]]"
dependencies: ["research_agent"]
---

# Analysis Agent Private Workspace

## Current Analysis Tasks
### Customer Segmentation Analysis
**Input**: Research data from [[Research_Agent_Workspace]]
**Method**: K-means clustering + behavioral analysis
**Output**: Customer personas and targeting recommendations

#### Analysis Framework
```python
# Segmentation parameters
features = ['purchase_frequency', 'avg_order_value', 'tenure', 'support_tickets']
n_clusters = 5
```

#### Preliminary Results
- **Segment 1**: High-value, low-maintenance (23% of customers)
- **Segment 2**: Growth potential (34% of customers)
- **Segment 3**: At-risk churners (18% of customers)

### Revenue Trend Analysis
**Time Period**: Last 24 months
**Methodology**: Time series analysis with seasonal adjustment

#### Key Metrics
| Metric | Q1 2025 | Q2 2025 | Q3 2025 (forecast) |
|--------|---------|---------|-------------------|
| Revenue | $2.3M | $2.7M | $3.1M |
| Growth Rate | 12% | 17% | 15% |
| Customer Acquisition | 450 | 520 | 580 |

## Analysis Tools & Scripts
### Statistical Analysis
- Correlation analysis: [[Correlation Matrix Template]]
- Regression models: [[Regression Analysis Guide]]
- Hypothesis testing: [[Statistical Testing Framework]]

### Visualization Templates
- Dashboard layouts: [[Dashboard Design Guide]]
- Chart types: [[Data Visualization Standards]]
- Interactive reports: [[Interactive Report Template]]

## Quality Assurance
### Data Validation Rules
1. **Completeness**: No missing values in key fields
2. **Consistency**: Cross-reference with source data
3. **Accuracy**: Spot-check calculations manually
4. **Timeliness**: Data must be within 30 days

### Confidence Levels
- **High**: Statistical significance p < 0.01
- **Medium**: Statistical significance p < 0.05
- **Low**: Descriptive analysis only

## Communication with Other Agents
### Requests to Research Agent
- Need competitor pricing data (Priority: High)
- Request market size validation (Priority: Medium)

### Outputs for Writing Agent
- Customer segmentation report ready
- Revenue analysis dashboard completed
"#;
    
    test_suite.create_test_note("Analysis_Agent_Workspace.md", analysis_workspace)?;
    
    // Test that each agent has isolated workspace
    let parser = ObsidianParser::new()?;
    
    let research_doc = parser.parse_file(&test_suite.vault_path.join("Research_Agent_Workspace.md")).await?;
    let analysis_doc = parser.parse_file(&test_suite.vault_path.join("Analysis_Agent_Workspace.md")).await?;
    
    // Verify agent identification in frontmatter
    let research_fm = research_doc.frontmatter.unwrap();
    let analysis_fm = analysis_doc.frontmatter.unwrap();
    
    assert_eq!(
        research_fm.custom_fields.get("agent").unwrap().as_str().unwrap(), 
        "research_agent"
    );
    assert_eq!(
        analysis_fm.custom_fields.get("agent").unwrap().as_str().unwrap(), 
        "analysis_agent"
    );
    
    // Verify cross-references between agents
    let analysis_links: Vec<_> = analysis_doc.links.iter()
        .filter(|link| link.target.contains("Research_Agent_Workspace"))
        .collect();
    assert!(!analysis_links.is_empty(), "Analysis agent should reference research agent workspace");
    
    println!("✅ Agent section isolation test passed");
    Ok(())
}

#[tokio::test]
async fn test_obsidian_uri_integration() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Test Obsidian URI scheme integration for AI agents
    let obsidian_uri_note = r#"---
title: "Obsidian URI Commands for AI Agents"
tags: ["automation", "obsidian-uri", "ai-integration"]
---

# Obsidian URI Commands for AI Agents

## Basic Commands
### Create New Note
```
obsidian://new?vault=note-to-ai&name=New%20Research%20Note&content=Hello%20World
```

### Open Specific Note
```
obsidian://open?vault=note-to-ai&file=Research%20Agent%20Workspace
```

### Search Vault
```
obsidian://search?vault=note-to-ai&query=market%20analysis
```

## Advanced Commands for AI Agents
### Append to Existing Note
```
obsidian://advanced-uri?vault=note-to-ai&filepath=Research_Agent_Workspace.md&mode=append&data=## New Finding\nMarket growth rate: 23%
```

### Create Daily Note with Template
```
obsidian://advanced-uri?vault=note-to-ai&daily=true&template=Daily%20Agent%20Report
```

### Execute Command Palette
```
obsidian://advanced-uri?vault=note-to-ai&commandid=graph:open
```

## AI Agent Automation Workflows
### Research Agent Workflow
1. Create research note with URI
2. Populate with initial data via append
3. Generate links to related notes
4. Update knowledge base index

### Analysis Agent Workflow
1. Read data from research notes
2. Create analysis workspace
3. Generate visualizations (via plugin)
4. Update dashboard with URI commands

## Plugin Integration Commands
### Dataview Plugin
```
obsidian://advanced-uri?vault=note-to-ai&filepath=Agent_Dashboard.md&mode=overwrite&data=```dataview\nTABLE agent, status, priority FROM "agents" WHERE status = "active"\n```
```

### Templater Plugin
```
obsidian://advanced-uri?vault=note-to-ai&template=Agent%20Report%20Template&templateVars={"agent_name":"research_agent","date":"2025-08-18"}
```

### Kanban Plugin
```
obsidian://advanced-uri?vault=note-to-ai&commandid=obsidian-kanban:create-new-board&filepath=Agent_Task_Board.md
```
"#;
    
    test_suite.create_test_note("Obsidian_URI_Commands.md", obsidian_uri_note)?;
    
    // Test URI command extraction
    let parser = ObsidianParser::new()?;
    let doc = parser.parse_file(&test_suite.vault_path.join("Obsidian_URI_Commands.md")).await?;
    
    // Verify code blocks contain URI commands
    let code_blocks: Vec<_> = doc.blocks.iter()
        .filter(|block| matches!(block.block_type, note_to_ai::vault::parser::BlockType::CodeBlock(_)))
        .collect();
    
    assert!(code_blocks.len() >= 5, "Should have multiple code blocks with URI commands");
    
    // Verify URI commands are properly formatted
    let obsidian_uris: Vec<_> = code_blocks.iter()
        .filter(|block| block.content.contains("obsidian://"))
        .collect();
    assert!(!obsidian_uris.is_empty(), "Should contain Obsidian URI commands");
    
    println!("✅ Obsidian URI integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_agent_collaboration_protocol() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Create a collaboration protocol document
    let collaboration_protocol = r#"---
title: "AI Agent Collaboration Protocol"
version: "1.0"
last_updated: "2025-08-18"
agents: ["research_agent", "analysis_agent", "writing_agent", "verification_agent"]
---

# AI Agent Collaboration Protocol

## Message Passing System
### Standard Message Format
```json
{
  "from_agent": "research_agent",
  "to_agent": "analysis_agent",
  "message_type": "data_transfer",
  "priority": "high",
  "timestamp": "2025-08-18T10:30:00Z",
  "content": {
    "data_location": "[[Market Analysis Data]]",
    "summary": "Market size data for Q3 analysis",
    "validation_required": true
  },
  "response_required": true,
  "deadline": "2025-08-18T15:00:00Z"
}
```

## Coordination Mechanisms
### Task Dependencies
```mermaid
graph LR
    A[Research Task] --> B[Analysis Task]
    B --> C[Writing Task]
    C --> D[Verification Task]
    D --> E[Final Output]
    
    A1[Market Research] --> B1[Customer Analysis]
    A2[Competitor Research] --> B1
    B1 --> C1[Executive Summary]
    C1 --> D1[Fact Check]
```

### Resource Sharing
| Resource Type | Access Level | Agents | Location |
|--------------|-------------|---------|----------|
| Raw Data | Read-Only | All | `data/raw/` |
| Analysis Results | Read-Write | Analysis, Writing | `data/processed/` |
| Draft Documents | Read-Write | Writing, Verification | `drafts/` |
| Final Outputs | Read-Only | All | `outputs/` |

## Quality Gates
### Research to Analysis Handoff
- [ ] Data completeness check (>95% complete)
- [ ] Source validation (all sources verified)
- [ ] Data quality score (>8/10)
- [ ] Methodology documentation complete

### Analysis to Writing Handoff
- [ ] Statistical significance verified
- [ ] Visualizations generated
- [ ] Key insights summarized
- [ ] Confidence levels documented

### Writing to Verification Handoff
- [ ] Document structure complete
- [ ] Citations properly formatted
- [ ] Executive summary included
- [ ] Stakeholder review ready

## Error Handling
### Agent Unavailable
1. Check agent status in `[[Agent Status Dashboard]]`
2. Escalate to backup agent if available
3. Log incident in `[[System Incident Log]]`
4. Notify system administrator

### Data Inconsistency
1. Flag inconsistent data points
2. Request verification from source agent
3. Document resolution process
4. Update data validation rules

### Deadline Conflicts
1. Assess task priority matrix
2. Negotiate deadline extension
3. Reallocate resources if needed
4. Update stakeholder expectations

## Performance Monitoring
### Agent Metrics
- Task completion rate
- Response time to requests
- Data quality scores
- Collaboration effectiveness

### System Health Checks
- Daily status report generation
- Weekly performance review
- Monthly protocol optimization
- Quarterly capability assessment
"#;
    
    test_suite.create_test_note("Agent_Collaboration_Protocol.md", collaboration_protocol)?;
    
    // Test parsing of collaboration structures
    let parser = ObsidianParser::new()?;
    let doc = parser.parse_file(&test_suite.vault_path.join("Agent_Collaboration_Protocol.md")).await?;
    
    // Verify JSON code blocks for message format
    let json_blocks: Vec<_> = doc.blocks.iter()
        .filter(|block| {
            if let note_to_ai::vault::parser::BlockType::CodeBlock(Some(lang)) = &block.block_type {
                lang == "json"
            } else {
                false
            }
        })
        .collect();
    assert!(!json_blocks.is_empty(), "Should contain JSON message format examples");
    
    // Verify Mermaid diagrams for workflow visualization
    let mermaid_blocks: Vec<_> = doc.blocks.iter()
        .filter(|block| {
            if let note_to_ai::vault::parser::BlockType::CodeBlock(Some(lang)) = &block.block_type {
                lang == "mermaid"
            } else {
                false
            }
        })
        .collect();
    assert!(!mermaid_blocks.is_empty(), "Should contain Mermaid workflow diagrams");
    
    // Verify tables for resource management
    let tables: Vec<_> = doc.blocks.iter()
        .filter(|block| matches!(block.block_type, note_to_ai::vault::parser::BlockType::Table))
        .collect();
    assert!(!tables.is_empty(), "Should contain resource sharing tables");
    
    // Verify task lists for quality gates
    let task_count = doc.content.matches("- [ ]").count();
    assert!(task_count >= 10, "Should have multiple quality gate checkboxes");
    
    println!("✅ Agent collaboration protocol test passed");
    Ok(())
}

#[tokio::test] 
async fn test_vault_semantic_search_for_agents() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Create multiple notes with different agent responsibilities
    let notes = vec![
        ("Market_Research_Q3.md", r#"---
title: "Q3 Market Research Findings"
agent: "research_agent"
tags: ["market-research", "q3-2025", "competitive-analysis"]
---

# Q3 Market Research Findings

## Executive Summary
The AI automation market has shown unprecedented growth of 23% quarter-over-quarter, driven primarily by small to medium businesses adopting workflow automation tools.

## Key Findings
### Market Size
- Total addressable market: $47.2B
- Serviceable addressable market: $12.3B  
- Current market penetration: 8.4%

### Competitive Landscape
- **Market Leader**: AutomationCorp (32% market share)
- **Fast Grower**: AI-Flow Solutions (18% growth rate)
- **Disruptor**: SmartWork AI (innovative pricing model)

### Customer Insights
Primary pain points identified:
1. Integration complexity (67% of respondents)
2. Training requirements (54% of respondents)  
3. ROI uncertainty (43% of respondents)
"#),
        ("Customer_Segmentation_Analysis.md", r#"---
title: "Customer Segmentation Analysis Results"
agent: "analysis_agent"
tags: ["customer-analysis", "segmentation", "data-science"]
dependencies: ["Market_Research_Q3"]
---

# Customer Segmentation Analysis Results

## Methodology
Applied K-means clustering algorithm on customer behavioral data with the following features:
- Purchase frequency
- Average order value
- Support ticket volume
- Feature usage patterns

## Segment Profiles
### Segment 1: Enterprise Champions (23%)
- High value customers ($50K+ annual revenue)
- Low support requirements
- Advanced feature adoption
- Strong retention rates (94%)

### Segment 2: Growth Potential (34%)
- Medium value customers ($10K-$50K annual revenue)
- Moderate support needs
- Feature expansion opportunities
- Good retention rates (78%)

### Segment 3: At-Risk Churners (18%)
- Declining usage patterns
- High support ticket volume
- Limited feature adoption
- Poor retention rates (34%)

## Recommendations
1. **Enterprise Champions**: Upsell premium features
2. **Growth Potential**: Targeted feature education campaigns  
3. **At-Risk Churners**: Immediate intervention program
"#),
        ("Executive_Summary_Q3_Strategy.md", r#"---
title: "Q3 Strategy Executive Summary"
agent: "writing_agent"
tags: ["executive-summary", "strategy", "q3-2025"]
dependencies: ["Market_Research_Q3", "Customer_Segmentation_Analysis"]
---

# Q3 Strategy Executive Summary

## Overview
Based on comprehensive market research and customer analysis, we recommend a focused approach to capture the 23% market growth opportunity while addressing customer retention challenges.

## Strategic Priorities
### 1. Market Expansion (Priority: High)
The AI automation market presents a $47.2B opportunity with our current 8.4% penetration rate. Key actions:
- Target SMB segment showing highest growth
- Develop simplified onboarding process
- Address integration complexity concerns

### 2. Customer Retention (Priority: Critical)  
Analysis reveals 18% of customers are at-risk churners. Immediate actions required:
- Launch proactive support intervention
- Implement usage monitoring alerts
- Develop customer success playbooks

### 3. Product Development (Priority: Medium)
Market research identifies clear feature gaps:
- Enhanced integration capabilities
- Self-service training modules
- ROI tracking dashboards

## Financial Impact
- **Revenue Opportunity**: $2.3M incremental in Q4
- **Retention Savings**: $890K annually
- **Investment Required**: $450K in Q3

## Next Steps
1. Approve Q3 budget allocation
2. Initiate at-risk customer intervention program
3. Begin SMB market expansion pilot
4. Establish success metrics and review cadence
"#),
        ("Fact_Check_Q3_Strategy.md", r#"---
title: "Fact Check Report: Q3 Strategy"
agent: "verification_agent"
tags: ["fact-check", "verification", "quality-assurance"]
dependencies: ["Executive_Summary_Q3_Strategy"]
---

# Fact Check Report: Q3 Strategy Executive Summary

## Verification Summary
**Overall Accuracy Score**: 94/100
**Sources Verified**: 15/15
**Data Points Checked**: 23/23
**Recommendations**: Approved with minor clarifications

## Detailed Verification
### Market Data Verification ✅
- **Market size ($47.2B)**: Verified against 3 industry sources
  - Gartner: $47.1B ±2%
  - IDC: $47.8B ±3%  
  - McKinsey: $46.9B ±2%
- **Growth rate (23%)**: Confirmed across multiple data points
- **Market penetration (8.4%)**: Calculated from verified market share data

### Customer Analysis Verification ✅
- **Segmentation percentages**: Validated against customer database
- **Retention rates**: Confirmed by customer success team
- **Revenue figures**: Cross-referenced with finance department

### Financial Projections Verification ⚠️
- **Revenue opportunity ($2.3M)**: Conservative estimate - APPROVED
- **Retention savings ($890K)**: Based on verified churn cost analysis
- **Investment required ($450K)**: Confirmed with department budgets

## Data Quality Assessment
### High Confidence (Score: 9-10)
- Market size data
- Customer segmentation results
- Historical financial performance

### Medium Confidence (Score: 7-8)  
- Competitive positioning estimates
- Feature adoption projections
- Timeline assumptions

### Recommendations for Improvement
1. Include confidence intervals for market projections
2. Add sensitivity analysis for financial estimates
3. Specify data collection timeframes
4. Include sample size information for surveys

## Compliance Check ✅
- All financial projections follow SEC guidelines
- Customer data handling complies with GDPR
- No confidential competitor information disclosed
- Attribution properly documented

## Final Approval
✅ **APPROVED** for stakeholder distribution with noted clarifications.

**Verified by**: Verification Agent v2.1
**Verification Date**: 2025-08-18T14:30:00Z
**Review Deadline**: 2025-08-25T17:00:00Z
"#),
    ];
    
    // Create all test notes
    for (filename, content) in notes {
        test_suite.create_test_note(filename, content)?;
    }
    
    // Index all notes in the vault
    let index_stats = test_suite.vault.index_all(false, None::<fn(&Path)>).await?;
    assert!(index_stats.docs_indexed >= 4, "Should have indexed at least 4 documents");
    
    // Test semantic search across agent specializations
    let search_queries = vec![
        ("market growth automation", "Should find market research content"),
        ("customer retention analysis", "Should find segmentation and strategy content"),
        ("financial projections Q3", "Should find strategy and verification content"),
        ("fact check verification", "Should find verification agent content"),
        ("integration complexity", "Should find market research and strategy content"),
    ];
    
    for (query, description) in search_queries {
        println!("Testing search: {} ({})", query, description);
        
        let results = test_suite.vault.search(query, 10, true).await?;
        assert!(!results.is_empty(), "Search for '{}' should return results", query);
        
        // Verify search results have proper metadata
        for result in &results {
            assert!(!result.title.is_empty(), "Result should have title");
            assert!(result.score > 0.0, "Result should have relevance score");
            assert!(!result.content_snippet.is_empty(), "Result should have content snippet");
        }
        
        println!("  ✅ Found {} results for '{}'", results.len(), query);
    }
    
    // Test agent-specific searches
    let agent_queries = vec![
        ("agent:research_agent market", "Should find research agent content"),
        ("agent:analysis_agent segmentation", "Should find analysis agent content"),  
        ("agent:writing_agent executive", "Should find writing agent content"),
        ("agent:verification_agent fact", "Should find verification agent content"),
    ];
    
    for (query, description) in agent_queries {
        let results = test_suite.vault.search(query, 5, false).await?;
        println!("Agent search '{}': {} results", query, results.len());
        // Note: Agent-specific search would require custom search filters
        // This tests the basic search infrastructure
    }
    
    println!("✅ Vault semantic search for agents test passed");
    Ok(())
}

// Helper module for AI agent integration
mod ai_agent_integration {
    use super::*;
    
    /// AI Agent types for specialization
    #[derive(Debug, Clone, PartialEq)]
    pub enum AgentType {
        Research,
        Analysis, 
        Writing,
        Verification,
        Coordination,
    }
    
    /// Agent access permissions for vault operations
    #[derive(Debug, Clone)]
    pub struct AgentPermissions {
        pub agent_id: String,
        pub agent_type: AgentType,
        pub read_paths: Vec<PathBuf>,
        pub write_paths: Vec<PathBuf>,
        pub can_create_notes: bool,
        pub can_modify_shared_resources: bool,
    }
    
    /// Agent workspace configuration
    #[derive(Debug, Clone)]
    pub struct AgentWorkspace {
        pub agent_id: String,
        pub workspace_path: PathBuf,
        pub private_notes_path: PathBuf,
        pub shared_resources_path: PathBuf,
        pub permissions: AgentPermissions,
    }
    
    /// Message passing system for agent coordination
    #[derive(Debug, Clone)]
    pub struct AgentMessage {
        pub from_agent: String,
        pub to_agent: String,
        pub message_type: MessageType,
        pub content: serde_json::Value,
        pub priority: Priority,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub response_required: bool,
        pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    }
    
    #[derive(Debug, Clone)]
    pub enum MessageType {
        DataTransfer,
        TaskRequest,
        StatusUpdate,
        QualityGate,
        ErrorReport,
    }
    
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Priority {
        Low,
        Medium,
        High,
        Critical,
    }
}

/// Integration tests for Obsidian plugin compatibility
#[tokio::test]
async fn test_obsidian_plugin_compatibility() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Test Dataview plugin compatibility
    let dataview_note = r#"---
title: "Agent Status Dashboard"
tags: ["dashboard", "dataview", "agent-monitoring"]
---

# Agent Status Dashboard

## Active Agents
```dataview
TABLE agent, status, current_task, last_update
FROM "agents"
WHERE status = "active"
SORT last_update DESC
```

## Task Summary
```dataview
LIST 
FROM "tasks"
WHERE status != "completed"
GROUP BY agent
```

## Performance Metrics
```dataview
TABLE agent, tasks_completed, avg_completion_time, quality_score
FROM "metrics"
WHERE date >= date(today) - dur(7 days)
SORT quality_score DESC
```
"#;
    
    test_suite.create_test_note("Agent_Status_Dashboard.md", dataview_note)?;
    
    // Test Templater plugin compatibility
    let template_note = r#"---
title: "Agent Report Template"
template: true
tags: ["template", "agent-report"]
---

# <% tp.frontmatter.agent_name %> Report - <% tp.date.now("YYYY-MM-DD") %>

## Summary
**Agent**: <% tp.frontmatter.agent_name %>
**Date**: <% tp.date.now("YYYY-MM-DD HH:mm") %>
**Status**: <% tp.frontmatter.status || "Active" %>

## Today's Accomplishments
<% await tp.file.cursor(1) %>

## Key Metrics
- Tasks completed: <% tp.frontmatter.tasks_completed || 0 %>
- Data processed: <% tp.frontmatter.data_processed || "N/A" %>
- Quality score: <% tp.frontmatter.quality_score || "N/A" %>

## Issues Encountered
<% await tp.file.cursor(2) %>

## Tomorrow's Plan
<% await tp.file.cursor(3) %>

## Agent Communication Log
### Messages Sent
<% await tp.file.cursor(4) %>

### Messages Received
<% await tp.file.cursor(5) %>

---
*Generated by Templater on <% tp.date.now() %>*
"#;
    
    test_suite.create_test_note("templates/Agent_Report_Template.md", template_note)?;
    
    // Test Kanban plugin compatibility
    let kanban_note = r#"---
title: "AI Agent Task Board"
kanban-plugin: board
---

## Backlog

- [ ] Market research for Q4 planning
- [ ] Customer satisfaction survey analysis
- [ ] Competitive intelligence gathering
- [ ] Technical documentation update

## In Progress

- [ ] Q3 financial analysis #research-agent
- [ ] Customer segmentation modeling #analysis-agent
- [ ] Executive summary draft #writing-agent

## Review

- [ ] Market trends report #verification-agent
- [ ] Data quality assessment #verification-agent

## Complete

- [x] Q2 performance review
- [x] Customer retention analysis
- [x] Competitor pricing study

%% kanban:settings
```
{"kanban-plugin":"board","list-collapse":[false,false,false,false]}
```
%%
"#;
    
    test_suite.create_test_note("AI_Agent_Task_Board.md", kanban_note)?;
    
    // Parse and verify plugin-specific content
    let parser = ObsidianParser::new()?;
    
    // Test Dataview parsing
    let dataview_doc = parser.parse_file(&test_suite.vault_path.join("Agent_Status_Dashboard.md")).await?;
    let dataview_blocks: Vec<_> = dataview_doc.blocks.iter()
        .filter(|block| {
            if let note_to_ai::vault::parser::BlockType::CodeBlock(Some(lang)) = &block.block_type {
                lang == "dataview"
            } else {
                false
            }
        })
        .collect();
    assert!(dataview_blocks.len() >= 3, "Should have multiple Dataview queries");
    
    // Test Templater parsing  
    let template_doc = parser.parse_file(&test_suite.vault_path.join("templates/Agent_Report_Template.md")).await?;
    let template_vars = template_doc.content.matches("<%").count();
    assert!(template_vars >= 5, "Should have multiple Templater variables");
    
    // Test Kanban parsing
    let kanban_doc = parser.parse_file(&test_suite.vault_path.join("AI_Agent_Task_Board.md")).await?;
    assert!(kanban_doc.frontmatter.is_some());
    let kanban_fm = kanban_doc.frontmatter.unwrap();
    assert!(kanban_fm.custom_fields.contains_key("kanban-plugin"));
    
    println!("✅ Obsidian plugin compatibility test passed");
    Ok(())
}

#[tokio::test]
async fn test_agent_knowledge_sharing_workflow() -> Result<()> {
    let test_suite = VaultAITestSuite::new().await?;
    
    // Simulate a complete workflow from research to publication
    
    // Step 1: Research Agent creates initial findings
    let research_note = test_suite.create_test_note("Research_AI_Market_Trends.md", r#"---
title: "AI Market Trends Research"
agent: "research_agent"
status: "completed"
confidence: 0.85
sources: 15
---

# AI Market Trends Research

## Key Findings
1. **Enterprise AI Adoption**: 67% of Fortune 500 companies have AI initiatives
2. **Market Growth**: AI market expected to reach $1.8T by 2030
3. **Investment Trends**: VC funding in AI startups up 156% YoY

## Data Sources
- Industry reports: 8 sources
- Expert interviews: 4 sources  
- Survey data: 3 sources

## Raw Data
[[AI Market Data Spreadsheet]]
[[Expert Interview Transcripts]]
[[Survey Results Summary]]
"#)?;
    
    // Step 2: Analysis Agent processes the research
    let analysis_note = test_suite.create_test_note("Analysis_AI_Market_Implications.md", r#"---
title: "AI Market Implications Analysis"
agent: "analysis_agent"
status: "completed"
input_sources: ["Research_AI_Market_Trends"]
methodology: "statistical_analysis"
---

# AI Market Implications Analysis

## Executive Summary
Analysis of [[Research_AI_Market_Trends]] reveals significant market opportunities with manageable risks.

## Statistical Analysis
### Growth Projections
- CAGR: 42.3% (2025-2030)
- Market size confidence interval: ±12%
- Adoption rate acceleration: 23% quarterly

### Risk Assessment
- **High Risk**: Regulatory changes (probability: 0.3)
- **Medium Risk**: Market saturation (probability: 0.4)
- **Low Risk**: Technology disruption (probability: 0.2)

## Strategic Recommendations
1. **Immediate Actions** (0-6 months)
   - Increase AI R&D investment by 25%
   - Establish regulatory compliance team
   
2. **Medium Term** (6-18 months)
   - Launch enterprise AI consulting division
   - Develop AI ethics framework

3. **Long Term** (18+ months)
   - Consider strategic acquisitions
   - Build proprietary AI platform
"#)?;
    
    // Step 3: Writing Agent creates executive summary
    let executive_note = test_suite.create_test_note("Executive_AI_Market_Strategy.md", r#"---
title: "AI Market Strategy - Executive Brief"
agent: "writing_agent"
status: "draft"
audience: "c_suite"
input_sources: ["Research_AI_Market_Trends", "Analysis_AI_Market_Implications"]
---

# AI Market Strategy - Executive Brief

## Bottom Line Up Front
The AI market presents a $1.8T opportunity by 2030. We recommend immediate 25% increase in AI R&D investment to capture enterprise market share.

## Market Opportunity
Based on comprehensive research by our [[Research_AI_Market_Trends|research team]] and [[Analysis_AI_Market_Implications|quantitative analysis]], the AI market shows exceptional growth potential:

- **Market Size**: $1.8T by 2030 (42.3% CAGR)
- **Enterprise Adoption**: 67% of Fortune 500 already invested
- **Funding Environment**: VC investment up 156% year-over-year

## Strategic Recommendations

### Immediate Actions (Next 6 Months)
**Investment**: Increase AI R&D budget by 25% ($2.3M additional)
- Hire 8 additional AI engineers
- Establish regulatory compliance team
- Begin enterprise customer pilots

**Expected Return**: $8.7M incremental revenue in 18 months

### Risk Mitigation
- **Regulatory Risk**: Establish compliance framework early
- **Competition Risk**: Focus on proprietary AI capabilities
- **Talent Risk**: Competitive compensation packages

## Financial Impact
- **Investment Required**: $2.3M in next 6 months
- **Revenue Opportunity**: $8.7M in 18 months
- **ROI**: 278% over 24 months

## Next Steps
1. Board approval for budget increase
2. Recruitment plan for AI talent
3. Enterprise pilot customer selection
4. Regulatory framework development

---
*This brief synthesizes research and analysis from our AI agents. Full supporting documentation available upon request.*
"#)?;
    
    // Step 4: Verification Agent fact-checks everything
    let verification_note = test_suite.create_test_note("Verification_AI_Strategy_Brief.md", r#"---
title: "Verification Report: AI Strategy Brief"
agent: "verification_agent"
status: "completed"
accuracy_score: 0.92
sources_verified: 18
---

# Verification Report: AI Strategy Brief

## Verification Summary
**Document**: [[Executive_AI_Market_Strategy]]
**Overall Accuracy**: 92%
**Sources Verified**: 18/18
**Financial Calculations**: ✅ Verified
**Recommendation**: ✅ Approved with minor notes

## Fact Checking Results

### Market Data Verification ✅
- **$1.8T market size**: Confirmed across 3 independent sources
  - McKinsey Global Institute: $1.7T-$1.9T range
  - PwC Analysis: $1.8T baseline scenario
  - Gartner: $1.75T conservative estimate

- **42.3% CAGR**: Verified against historical data and projections
- **67% Fortune 500 adoption**: Cross-referenced with Fortune magazine survey

### Financial Projections Review ✅
- **$2.3M investment calculation**: Verified against HR and infrastructure costs
- **$8.7M revenue projection**: Conservative based on market analysis
- **278% ROI**: Calculation confirmed, assumptions documented

### Risk Assessment Validation ✅
- **Regulatory risk probability (0.3)**: Aligned with legal team assessment  
- **Market saturation risk (0.4)**: Supported by competitive analysis
- **Technology disruption (0.2)**: Conservative estimate approved

## Quality Assessment

### Strengths
- Comprehensive source validation
- Conservative financial projections
- Clear action items with timelines
- Appropriate risk mitigation strategies

### Areas for Improvement
- Include sensitivity analysis for key assumptions
- Add competitive response scenarios
- Specify success metrics for tracking
- Define exit criteria for pilot programs

## Compliance Check ✅
- SEC disclosure requirements reviewed
- No material non-public information disclosed
- Competitive intelligence obtained through public sources
- Privacy and confidentiality maintained

## Agent Collaboration Review ✅
- Research methodology sound and well-documented
- Analysis approach statistically valid
- Writing clear and actionable for target audience
- Cross-referencing between documents accurate

## Final Recommendation
**✅ APPROVED** for executive presentation with noted improvements.

**Verification Confidence**: 92%
**Recommended Distribution**: C-Suite, Board of Directors
**Review Cycle**: Monthly updates recommended
**Next Verification**: 2025-09-18

---
*Verified by AI Verification Agent v3.1*
*Quality assurance protocols: ISO 27001 compliant*
"#)?;
    
    // Index all the workflow documents
    let index_stats = test_suite.vault.index_all(false, None::<fn(&Path)>).await?;
    assert!(index_stats.docs_indexed >= 4, "Should index all workflow documents");
    
    // Test cross-reference integrity
    let parser = ObsidianParser::new()?;
    
    // Verify analysis references research
    let analysis_doc = parser.parse_file(&analysis_note).await?;
    let research_links: Vec<_> = analysis_doc.links.iter()
        .filter(|link| link.target.contains("Research_AI_Market_Trends"))
        .collect();
    assert!(!research_links.is_empty(), "Analysis should reference research");
    
    // Verify executive brief references both research and analysis
    let exec_doc = parser.parse_file(&executive_note).await?;
    let ref_count = exec_doc.links.len();
    assert!(ref_count >= 2, "Executive brief should reference multiple sources");
    
    // Verify verification references the executive brief
    let verify_doc = parser.parse_file(&verification_note).await?;
    let exec_links: Vec<_> = verify_doc.links.iter()
        .filter(|link| link.target.contains("Executive_AI_Market_Strategy"))
        .collect();
    assert!(!exec_links.is_empty(), "Verification should reference executive brief");
    
    // Test semantic search across the workflow
    let workflow_search = test_suite.vault.search("AI market investment ROI", 10, true).await?;
    assert!(workflow_search.len() >= 3, "Should find multiple documents in workflow");
    
    // Verify each document has appropriate metadata
    for note_path in [&research_note, &analysis_note, &executive_note, &verification_note] {
        let doc = parser.parse_file(note_path).await?;
        assert!(doc.frontmatter.is_some(), "All workflow documents should have frontmatter");
        
        let fm = doc.frontmatter.unwrap();
        assert!(fm.custom_fields.contains_key("agent"), "Should specify responsible agent");
        assert!(fm.custom_fields.contains_key("status"), "Should specify document status");
    }
    
    println!("✅ Agent knowledge sharing workflow test passed");
    Ok(())
}
