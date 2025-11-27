use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub action: String,
    pub config: serde_json::Value,
}

pub fn parse_yaml(content: &str) -> anyhow::Result<PipelineDefinition> {
    serde_yaml::from_str(content).map_err(Into::into)
}

pub fn parse_json(content: &str) -> anyhow::Result<PipelineDefinition> {
    serde_json::from_str(content).map_err(Into::into)
}

