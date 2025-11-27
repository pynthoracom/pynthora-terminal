use anyhow::Result;
use pynthora_terminal::core::config::Config;
use reqwest::Client as HttpClient;
use std::sync::Arc;

pub struct Client {
    config: Arc<Config>,
    http_client: HttpClient,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            http_client: HttpClient::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.config.ingest_url
    }

    pub fn api_key(&self) -> &str {
        &self.config.api_key
    }

    pub fn workspace(&self) -> &str {
        &self.config.workspace
    }

    // TODO: Implement actual API methods
    // pub async fn push_pipeline(&self, pipeline: &serde_json::Value) -> Result<PipelineResponse> {
    //     let url = format!("{}/api/v1/pipelines", self.base_url());
    //     let response = self.http_client
    //         .post(&url)
    //         .header("Authorization", format!("Bearer {}", self.api_key()))
    //         .header("X-Workspace", self.workspace())
    //         .json(pipeline)
    //         .send()
    //         .await?;
    //     Ok(response.json().await?)
    // }
}

