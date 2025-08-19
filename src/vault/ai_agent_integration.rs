use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::Vault;

/// AI Agent types for specialization in the knowledge system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    /// Research agents gather and validate information from various sources
    Research,
    /// Analysis agents process data and generate insights
    Analysis,
    /// Writing agents create documentation and reports
    Writing,
    /// Verification agents fact-check and quality assure content
    Verification,
    /// Coordination agents manage workflows and agent collaboration
    Coordination,
}

impl AgentType {
    pub fn workspace_prefix(&self) -> &'static str {
        match self {
            AgentType::Research => "research",
            AgentType::Analysis => "analysis", 
            AgentType::Writing => "writing",
            AgentType::Verification => "verification",
            AgentType::Coordination => "coordination",
        }
    }
    
    pub fn default_permissions(&self) -> Vec<Permission> {
        match self {
            AgentType::Research => vec![
                Permission::ReadSharedResources,
                Permission::WriteOwnWorkspace,
                Permission::CreateNotes,
                Permission::SearchVault,
            ],
            AgentType::Analysis => vec![
                Permission::ReadSharedResources,
                Permission::ReadResearchData,
                Permission::WriteOwnWorkspace,
                Permission::CreateNotes,
                Permission::SearchVault,
            ],
            AgentType::Writing => vec![
                Permission::ReadSharedResources,
                Permission::ReadResearchData,
                Permission::ReadAnalysisData,
                Permission::WriteOwnWorkspace,
                Permission::WriteSharedDrafts,
                Permission::CreateNotes,
                Permission::SearchVault,
            ],
            AgentType::Verification => vec![
                Permission::ReadAll,
                Permission::WriteOwnWorkspace,
                Permission::WriteVerificationReports,
                Permission::CreateNotes,
                Permission::SearchVault,
            ],
            AgentType::Coordination => vec![
                Permission::ReadAll,
                Permission::WriteAll,
                Permission::CreateNotes,
                Permission::SearchVault,
                Permission::ManageWorkflows,
            ],
        }
    }
}

/// Permissions for agent operations in the vault
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    ReadSharedResources,
    ReadResearchData,
    ReadAnalysisData,
    ReadAll,
    WriteOwnWorkspace,
    WriteSharedDrafts,
    WriteSharedResources,
    WriteVerificationReports,
    WriteAll,
    CreateNotes,
    SearchVault,
    ManageWorkflows,
}

/// Agent configuration and identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub name: String,
    pub description: String,
    pub version: String,
    pub permissions: Vec<Permission>,
    pub workspace_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub configuration: HashMap<String, Value>,
}

impl AgentConfig {
    pub fn new(
        agent_type: AgentType,
        name: String,
        description: String,
        vault_path: &Path,
    ) -> Self {
        let agent_id = Uuid::new_v4().to_string();
        let workspace_path = vault_path
            .join("agents")
            .join(agent_type.workspace_prefix())
            .join(&agent_id);
        
        Self {
            agent_id,
            agent_type: agent_type.clone(),
            name,
            description,
            version: "1.0.0".to_string(),
            permissions: agent_type.default_permissions(),
            workspace_path,
            created_at: Utc::now(),
            last_active: Utc::now(),
            configuration: HashMap::new(),
        }
    }
    
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    pub fn can_read_path(&self, path: &Path) -> bool {
        if self.has_permission(&Permission::ReadAll) {
            return true;
        }
        
        if path.starts_with(&self.workspace_path) {
            return true;
        }
        
        if self.has_permission(&Permission::ReadSharedResources) 
            && path.to_string_lossy().contains("shared") {
            return true;
        }
        
        false
    }
    
    pub fn can_write_path(&self, path: &Path) -> bool {
        if self.has_permission(&Permission::WriteAll) {
            return true;
        }
        
        if path.starts_with(&self.workspace_path) 
            && self.has_permission(&Permission::WriteOwnWorkspace) {
            return true;
        }
        
        false
    }
}

/// Message types for inter-agent communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    /// Data transfer between agents
    DataTransfer,
    /// Task assignment or request
    TaskRequest,
    /// Status update notification
    StatusUpdate,
    /// Quality gate checkpoint
    QualityGate,
    /// Error or issue report
    ErrorReport,
    /// Workflow coordination
    WorkflowControl,
}

/// Priority levels for agent messages
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Inter-agent message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: MessageType,
    pub priority: Priority,
    pub content: Value,
    pub timestamp: DateTime<Utc>,
    pub response_required: bool,
    pub deadline: Option<DateTime<Utc>>,
    pub context: HashMap<String, Value>,
}

impl AgentMessage {
    pub fn new(
        from_agent: String,
        to_agent: String,
        message_type: MessageType,
        content: Value,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            from_agent,
            to_agent,
            message_type,
            priority: Priority::Medium,
            content,
            timestamp: Utc::now(),
            response_required: false,
            deadline: None,
            context: HashMap::new(),
        }
    }
    
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self.response_required = true;
        self
    }
    
    pub fn with_context(mut self, key: String, value: Value) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Workflow step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub dependencies: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub quality_gates: Vec<QualityGate>,
    pub estimated_duration: chrono::Duration,
}

/// Quality gate for workflow validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub description: String,
    pub validation_type: ValidationType,
    pub threshold: Option<f64>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    DataCompleteness,
    SourceValidation,
    StatisticalSignificance,
    FactCheck,
    CitationCheck,
    ReadabilityScore,
    CustomScript(String),
}

/// Workflow definition for agent collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub workflow_id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub version: String,
}

/// Agent workspace manager
pub struct AgentWorkspaceManager {
    vault: Vault,
    agents: HashMap<String, AgentConfig>,
    workflows: HashMap<String, Workflow>,
    message_queue: Vec<AgentMessage>,
}

impl AgentWorkspaceManager {
    pub fn new(vault: Vault) -> Self {
        Self {
            vault,
            agents: HashMap::new(),
            workflows: HashMap::new(),
            message_queue: Vec::new(),
        }
    }
    
    /// Register a new agent in the system
    pub async fn register_agent(
        &mut self,
        agent_type: AgentType,
        name: String,
        description: String,
    ) -> Result<String> {
        let vault_path = self.vault._vault_path.clone();
        let agent_config = AgentConfig::new(agent_type, name, description, &vault_path);
        let agent_id = agent_config.agent_id.clone();
        
        // Create agent workspace directory
        std::fs::create_dir_all(&agent_config.workspace_path)
            .context("Failed to create agent workspace")?;
        
        // Create initial workspace structure
        self.create_workspace_structure(&agent_config).await?;
        
        self.agents.insert(agent_id.clone(), agent_config);
        Ok(agent_id)
    }
    
    /// Create the initial workspace structure for an agent
    async fn create_workspace_structure(&self, agent: &AgentConfig) -> Result<()> {
        let workspace = &agent.workspace_path;
        
        // Create subdirectories
        std::fs::create_dir_all(workspace.join("private"))?;
        std::fs::create_dir_all(workspace.join("drafts"))?;
        std::fs::create_dir_all(workspace.join("outputs"))?;
        std::fs::create_dir_all(workspace.join("templates"))?;
        
        // Create agent README
        let readme_content = format!(r#"---
title: "{} Workspace"
agent: "{}"
agent_type: "{:?}"
created: "{}"
version: "{}"
---

# {} Workspace

## Agent Information
- **Type**: {:?}
- **Version**: {}
- **Created**: {}
- **Description**: {}

## Workspace Structure
- `private/` - Agent-specific private notes and data
- `drafts/` - Work in progress documents
- `outputs/` - Completed work products
- `templates/` - Document templates for this agent type

## Permissions
{:#?}

## Getting Started
This workspace is automatically managed by the AI Agent system. 
Files created here will be indexed and searchable within the agent's permission scope.

## Communication
Messages from other agents will appear in the shared communication channels.
Use the standard message format for inter-agent coordination.
"#,
            agent.name,
            agent.agent_id,
            agent.agent_type,
            agent.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            agent.version,
            agent.name,
            agent.agent_type,
            agent.version,
            agent.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            agent.description,
            agent.permissions
        );
        
        std::fs::write(workspace.join("README.md"), readme_content)?;
        
        // Create agent-specific templates based on type
        self.create_agent_templates(agent).await?;
        
        Ok(())
    }
    
    /// Create agent-specific templates
    async fn create_agent_templates(&self, agent: &AgentConfig) -> Result<()> {
        let templates_dir = agent.workspace_path.join("templates");
        
        match agent.agent_type {
            AgentType::Research => {
                let research_template = r#"---
title: "Research Report Template"
agent: "research_agent"
template: true
---

# Research Report: {{title}}

## Executive Summary
<!-- Brief overview of findings -->

## Research Questions
1. 
2. 
3. 

## Methodology
### Data Sources
- **Primary Sources**: 
- **Secondary Sources**: 
- **Data Collection Period**: 

### Research Methods
- 

## Key Findings
### Finding 1: {{finding_title}}
**Confidence Level**: High/Medium/Low
**Supporting Evidence**: 
**Sources**: 

## Data Quality Assessment
- **Completeness**: %
- **Reliability**: High/Medium/Low
- **Recency**: 
- **Bias Assessment**: 

## Recommendations for Further Research
1. 
2. 

## Raw Data References
- [[Data File 1]]
- [[Data File 2]]

---
*Research completed by: {{agent_name}}*
*Date: {{completion_date}}*
"#;
                std::fs::write(templates_dir.join("research_report_template.md"), research_template)?;
            },
            
            AgentType::Analysis => {
                let analysis_template = r#"---
title: "Analysis Report Template"
agent: "analysis_agent"
template: true
---

# Analysis Report: {{title}}

## Executive Summary
<!-- Key insights and recommendations -->

## Input Data
**Sources**: {{input_sources}}
**Data Quality Score**: {{quality_score}}/10
**Analysis Period**: {{analysis_period}}

## Methodology
### Analytical Framework
- **Approach**: 
- **Tools Used**: 
- **Statistical Methods**: 

### Assumptions
1. 
2. 

## Key Insights
### Insight 1: {{insight_title}}
**Statistical Significance**: p < {{p_value}}
**Confidence Interval**: 
**Practical Significance**: 

## Visualizations
- [[Chart 1: {{chart_title}}]]
- [[Dashboard: {{dashboard_title}}]]

## Risk Assessment
| Risk Factor | Probability | Impact | Mitigation |
|-------------|-------------|---------|------------|
|             |             |        |            |

## Recommendations
### Immediate Actions (0-30 days)
1. 
2. 

### Medium Term (1-6 months)
1. 
2. 

### Long Term (6+ months)
1. 
2. 

## Supporting Data
- [[Raw Analysis Results]]
- [[Statistical Output]]

---
*Analysis completed by: {{agent_name}}*
*Quality assured: {{qa_status}}*
"#;
                std::fs::write(templates_dir.join("analysis_report_template.md"), analysis_template)?;
            },
            
            AgentType::Writing => {
                let writing_template = r#"---
title: "Document Template"
agent: "writing_agent"
template: true
---

# {{document_title}}

## Document Information
- **Author**: {{agent_name}}
- **Audience**: {{target_audience}}
- **Purpose**: {{document_purpose}}
- **Classification**: {{classification}}

## Executive Summary
<!-- 2-3 sentences summarizing key points -->

## Main Content
### Section 1: {{section_title}}


### Section 2: {{section_title}}


## Key Takeaways
1. 
2. 
3. 

## Supporting References
- [[Source Document 1]]
- [[Analysis Report]]
- [[Research Findings]]

## Review Status
- [ ] Fact-checking complete
- [ ] Style review complete
- [ ] Stakeholder review complete
- [ ] Final approval

---
*Document prepared by: {{agent_name}}*
*Review cycle: {{review_status}}*
"#;
                std::fs::write(templates_dir.join("document_template.md"), writing_template)?;
            },
            
            AgentType::Verification => {
                let verification_template = r#"---
title: "Verification Report Template"
agent: "verification_agent"
template: true
---

# Verification Report: {{document_title}}

## Verification Summary
- **Document Reviewed**: [[{{source_document}}]]
- **Verification Date**: {{verification_date}}
- **Accuracy Score**: {{accuracy_score}}%
- **Recommendation**: {{recommendation}}

## Fact Checking Results
### Claims Verified: {{verified_count}}/{{total_claims}}

| Claim | Source | Verification Status | Notes |
|-------|--------|-------------------|-------|
|       |        |                   |       |

## Data Validation
### Statistical Claims
- [ ] Calculations verified
- [ ] Data sources confirmed
- [ ] Methodology appropriate
- [ ] Confidence intervals correct

### Citations and References
- [ ] All sources accessible
- [ ] Citations properly formatted
- [ ] Attribution accurate
- [ ] No plagiarism detected

## Quality Assessment
### Strengths
- 
- 

### Areas for Improvement
- 
- 

### Critical Issues
- 
- 

## Compliance Review
- [ ] Privacy requirements met
- [ ] Confidentiality maintained
- [ ] Regulatory compliance
- [ ] Ethical guidelines followed

## Final Recommendation
**Status**: Approved/Approved with Changes/Rejected
**Confidence Level**: High/Medium/Low

### Required Changes
1. 
2. 

---
*Verification completed by: {{agent_name}}*
*Next review date: {{next_review}}*
"#;
                std::fs::write(templates_dir.join("verification_report_template.md"), verification_template)?;
            },
            
            AgentType::Coordination => {
                let coordination_template = r#"---
title: "Workflow Coordination Template"
agent: "coordination_agent"
template: true
---

# Workflow: {{workflow_name}}

## Workflow Overview
- **Workflow ID**: {{workflow_id}}
- **Coordinator**: {{agent_name}}
- **Started**: {{start_date}}
- **Expected Completion**: {{end_date}}
- **Status**: {{workflow_status}}

## Participating Agents
| Agent | Type | Role | Status |
|-------|------|------|---------|
|       |      |      |         |

## Workflow Steps
### Step 1: {{step_name}}
- **Responsible Agent**: {{agent_type}}
- **Status**: {{step_status}}
- **Dependencies**: {{dependencies}}
- **Expected Duration**: {{duration}}

#### Quality Gates
- [ ] {{quality_gate_1}}
- [ ] {{quality_gate_2}}

## Communication Log
### {{date}} - {{time}}
**From**: {{from_agent}} → **To**: {{to_agent}}
**Type**: {{message_type}}
**Content**: {{message_content}}

## Resource Allocation
| Resource | Agent | Status | Notes |
|----------|-------|---------|-------|
|          |       |        |       |

## Risk Monitoring
| Risk | Probability | Impact | Mitigation Status |
|------|-------------|---------|------------------|
|      |             |        |                  |

## Performance Metrics
- **On-time Completion**: {{completion_rate}}%
- **Quality Score**: {{quality_score}}/10
- **Agent Utilization**: {{utilization_rate}}%

---
*Workflow coordinated by: {{agent_name}}*
*Last updated: {{last_update}}*
"#;
                std::fs::write(templates_dir.join("workflow_coordination_template.md"), coordination_template)?;
            },
        }
        
        Ok(())
    }
    
    /// Send a message between agents
    pub fn send_message(&mut self, message: AgentMessage) -> Result<()> {
        // Validate sender and receiver exist
        if !self.agents.contains_key(&message.from_agent) {
            return Err(anyhow::anyhow!("Sender agent {} not found", message.from_agent));
        }
        
        if !self.agents.contains_key(&message.to_agent) {
            return Err(anyhow::anyhow!("Receiver agent {} not found", message.to_agent));
        }
        
        // Add to message queue
        self.message_queue.push(message);
        
        // Sort queue by priority and timestamp
        self.message_queue.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });
        
        Ok(())
    }
    
    /// Get pending messages for an agent
    pub fn get_messages_for_agent(&self, agent_id: &str) -> Vec<&AgentMessage> {
        self.message_queue
            .iter()
            .filter(|msg| msg.to_agent == agent_id)
            .collect()
    }
    
    /// Create a note in an agent's workspace
    pub async fn create_agent_note(
        &self,
        agent_id: &str,
        filename: &str,
        content: &str,
        subfolder: Option<&str>,
    ) -> Result<PathBuf> {
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", agent_id))?;
        
        if !agent.has_permission(&Permission::CreateNotes) {
            return Err(anyhow::anyhow!("Agent {} does not have permission to create notes", agent_id));
        }
        
        let note_dir = if let Some(subfolder) = subfolder {
            agent.workspace_path.join(subfolder)
        } else {
            agent.workspace_path.join("private")
        };
        
        std::fs::create_dir_all(&note_dir)?;
        
        let note_path = note_dir.join(filename);
        
        // Add agent metadata to frontmatter
        let enhanced_content = if content.starts_with("---") {
            // Insert agent info into existing frontmatter
            let mut lines: Vec<&str> = content.lines().collect();
            if lines.len() > 1 && lines[0] == "---" {
                if let Some(end_idx) = lines.iter().skip(1).position(|&line| line == "---") {
                    let agent_line = format!("agent: \"{agent_id}\"");
                    let agent_type_line = format!("agent_type: \"{:?}\"", agent.agent_type);
                    let created_line = "created_by_ai: true".to_string();
                    
                    lines.insert(end_idx + 1, &agent_line);
                    lines.insert(end_idx + 2, &agent_type_line);
                    lines.insert(end_idx + 3, &created_line);
                    lines.join("\n")
                } else {
                    content.to_string()
                }
            } else {
                content.to_string()
            }
        } else {
            // Add frontmatter
            format!(r#"---
agent: "{}"
agent_type: "{:?}"
created_by_ai: true
created_at: "{}"
---

{}"#, agent_id, agent.agent_type, Utc::now().format("%Y-%m-%d %H:%M:%S UTC"), content)
        };
        
        std::fs::write(&note_path, enhanced_content)?;
        
        Ok(note_path)
    }
    
    /// Search the vault with agent permission filtering
    pub async fn search_for_agent(
        &self,
        agent_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::vault::SearchResult>> {
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", agent_id))?;
        
        if !agent.has_permission(&Permission::SearchVault) {
            return Err(anyhow::anyhow!("Agent {} does not have search permission", agent_id));
        }
        
        // Perform search
        let mut results = self.vault.search(query, limit * 2, true).await?;
        
        // Filter results based on agent permissions
        results.retain(|result| {
            let file_path = &result.document.path;
            agent.can_read_path(file_path)
        });
        
        // Limit to requested number
        results.truncate(limit);
        
        Ok(results)
    }
    
    /// Get agent statistics and status
    pub fn get_agent_status(&self, agent_id: &str) -> Option<AgentStatus> {
        let agent = self.agents.get(agent_id)?;
        
        let pending_messages = self.get_messages_for_agent(agent_id).len();
        
        Some(AgentStatus {
            agent_id: agent_id.to_string(),
            agent_type: agent.agent_type.clone(),
            name: agent.name.clone(),
            version: agent.version.clone(),
            last_active: agent.last_active,
            pending_messages,
            workspace_files: self.count_workspace_files(&agent.workspace_path).unwrap_or(0),
            status: if pending_messages > 5 { 
                AgentActivityStatus::Busy 
            } else { 
                AgentActivityStatus::Active 
            },
        })
    }
    
    /// Count files in agent workspace
    fn count_workspace_files(&self, workspace_path: &Path) -> Result<usize> {
        let mut count = 0;
        for entry in std::fs::read_dir(workspace_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                count += 1;
            } else if entry.file_type()?.is_dir() {
                count += self.count_workspace_files(&entry.path())?;
            }
        }
        Ok(count)
    }
    
    /// Register a new workflow
    pub fn register_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.workflow_id.clone(), workflow);
    }
    
    /// Get all registered agents
    pub fn list_agents(&self) -> Vec<&AgentConfig> {
        self.agents.values().collect()
    }
    
    /// Get all registered workflows
    pub fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }
}

/// Agent status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub name: String,
    pub version: String,
    pub last_active: DateTime<Utc>,
    pub pending_messages: usize,
    pub workspace_files: usize,
    pub status: AgentActivityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentActivityStatus {
    Active,
    Busy,
    Idle,
    Offline,
}

/// Convenience functions for creating common agent messages
impl AgentMessage {
    pub fn data_transfer(
        from: String,
        to: String,
        data_location: String,
        summary: String,
    ) -> Self {
        let content = serde_json::json!({
            "data_location": data_location,
            "summary": summary,
            "type": "data_transfer"
        });
        
        Self::new(from, to, MessageType::DataTransfer, content)
    }
    
    pub fn task_request(
        from: String,
        to: String,
        task_description: String,
        deadline: Option<DateTime<Utc>>,
    ) -> Self {
        let content = serde_json::json!({
            "task_description": task_description,
            "type": "task_request"
        });
        
        let mut message = Self::new(from, to, MessageType::TaskRequest, content);
        if let Some(deadline) = deadline {
            message = message.with_deadline(deadline);
        }
        message
    }
    
    pub fn status_update(
        from: String,
        to: String,
        status: String,
        details: Option<String>,
    ) -> Self {
        let content = serde_json::json!({
            "status": status,
            "details": details,
            "type": "status_update"
        });
        
        Self::new(from, to, MessageType::StatusUpdate, content)
    }
}