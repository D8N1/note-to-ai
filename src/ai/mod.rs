pub mod api_client;
pub mod context;
pub mod hermes_integration;
pub mod local_llm;
pub mod model_switcher;
pub mod model_loader; // REAL AI model loading - Day 2 Directive

use crate::Result;
use model_loader::{ModelLoader, EmbeddingModel};

pub struct AI {
    hermes: Option<hermes_integration::HermesIntegration>,
    local_llm: Option<local_llm::LocalLLM>,
}

impl AI {
    pub async fn new() -> Result<Self> {
        // Try to initialize Hermes integration (requires config)
        let hermes = None; // Disabled for now due to missing config
        
        // Try to initialize local LLM (requires model path)
        let local_llm = None; // Disabled for now due to missing model path
        
        tracing::warn!("AI backends not configured, using mock responses");
        
        Ok(Self {
            hermes,
            local_llm,
        })
    }
    
    pub async fn process_query(&self, query: &str) -> Result<String> {
        // For now, use intelligent mock responses until proper AI backends are configured
        Ok(self.generate_mock_response(query))
    }
    
    pub async fn process_with_context(&self, query: &str, context: &[String]) -> Result<String> {
        let context_str = context.join("\n");
        let full_query = format!("Context:\n{context_str}\n\nQuery: {query}");
        
        self.process_query(&full_query).await
    }
    
    fn generate_mock_response(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("strategic") || query_lower.contains("business") {
            "Based on the strategic context, I recommend analyzing the key factors: market position, competitive landscape, and resource allocation. Consider both short-term gains and long-term sustainability."
        } else if query_lower.contains("research") || query_lower.contains("analyze") {
            "For this research topic, I suggest starting with primary sources, validating findings with multiple references, and considering various perspectives. The analysis should be comprehensive yet actionable."
        } else if query_lower.contains("meeting") || query_lower.contains("presentation") {
            "For meeting preparation, ensure you have: clear objectives, key talking points, supporting data, and anticipated questions. Consider your audience and tailor the message accordingly."
        } else if query_lower.contains("decision") || query_lower.contains("choose") {
            "Decision-making framework: evaluate pros/cons, assess risks and opportunities, consider stakeholder impact, and timeline constraints. Gather input from relevant experts."
        } else if query_lower.contains("problem") || query_lower.contains("issue") {
            "Problem-solving approach: define the issue clearly, identify root causes, brainstorm solutions, evaluate options, and implement with monitoring."
        } else {
            "I understand your request. Based on the information provided, I recommend taking a systematic approach that considers multiple factors and stakeholder perspectives."
        }.to_string()
    }
    
    pub fn is_available(&self) -> bool {
        self.hermes.is_some() || self.local_llm.is_some()
    }
    
    pub fn status(&self) -> String {
        let mut status = Vec::new();
        
        if self.hermes.is_some() {
            status.push("Hermes: Available");
        } else {
            status.push("Hermes: Unavailable");
        }
        
        if self.local_llm.is_some() {
            status.push("Local LLM: Available");
        } else {
            status.push("Local LLM: Unavailable");
        }
        
        if status.is_empty() {
            "AI: Mock mode only".to_string()
        } else {
            status.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_creation() {
        let ai = AI::new().await;
        assert!(ai.is_ok());
    }

    #[tokio::test]
    async fn test_mock_responses() {
        let ai = AI::new().await.unwrap();
        
        let strategic_response = ai.process_query("strategic business analysis").await.unwrap();
        assert!(strategic_response.contains("strategic"));
        
        let research_response = ai.process_query("research this topic").await.unwrap();
        assert!(research_response.contains("research"));
    }

    #[tokio::test]
    async fn test_context_processing() {
        let ai = AI::new().await.unwrap();
        let context = vec!["Previous meeting notes".to_string(), "Project status".to_string()];
        
        let response = ai.process_with_context("What should we focus on?", &context).await;
        assert!(response.is_ok());
    }
}
