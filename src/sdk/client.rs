use anyhow::{Context, Result};
use pynthora_terminal::core::config::Config;
use reqwest::Client as HttpClient;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

pub struct Client {
    config: Arc<Config>,
    http_client: HttpClient,
}

impl Client {
    pub fn new(config: Config) -> Self {
        // Create HTTP client with optimized settings
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config: Arc::new(config),
            http_client,
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

    /// Stream a single event
    pub async fn stream_event(
        &self,
        event: &Value,
        pipeline: Option<&str>,
    ) -> Result<()> {
        let url = format!("{}/api/v1/ingest", self.base_url());
        
        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("X-Workspace", self.workspace())
            .header("Content-Type", "application/json")
            .json(event);

        if let Some(pipeline_id) = pipeline {
            request = request.header("X-Pipeline-Id", pipeline_id);
        }

        let response = request.send().await.context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Request failed with status {}: {}", status, error_text);
            anyhow::bail!("Request failed: {}", status);
        }

        debug!("Event streamed successfully");
        Ok(())
    }

    /// Stream a batch of events (v0.2.0 feature)
    pub async fn stream_batch(
        &self,
        events: &[Value],
        pipeline: Option<&str>,
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let url = format!("{}/api/v1/ingest/batch", self.base_url());
        
        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("X-Workspace", self.workspace())
            .header("Content-Type", "application/json")
            .json(events);

        if let Some(pipeline_id) = pipeline {
            request = request.header("X-Pipeline-Id", pipeline_id);
        }

        let response = request.send().await.context("Failed to send batch request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Batch request failed with status {}: {}", status, error_text);
            anyhow::bail!("Batch request failed: {}", status);
        }

        debug!("Batch of {} events streamed successfully", events.len());
        Ok(())
    }

    /// Get health status
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let url = format!("{}/api/v1/health", self.base_url());
        
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("X-Workspace", self.workspace())
            .send()
            .await
            .context("Failed to check health")?;

        if !response.status().is_success() {
            anyhow::bail!("Health check failed: {}", response.status());
        }

        let status: HealthStatus = response
            .json()
            .await
            .context("Failed to parse health response")?;

        Ok(status)
    }

    /// Push pipeline definition
    pub async fn push_pipeline(&self, pipeline: &Value) -> Result<PipelineResponse> {
        let url = format!("{}/api/v1/pipelines", self.base_url());
        
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("X-Workspace", self.workspace())
            .header("Content-Type", "application/json")
            .json(pipeline)
            .send()
            .await
            .context("Failed to push pipeline")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Pipeline push failed: {} - {}", status, error_text);
        }

        let result: PipelineResponse = response
            .json()
            .await
            .context("Failed to parse pipeline response")?;

        Ok(result)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: Option<String>,
    pub uptime: Option<u64>,
    pub metrics: Option<HealthMetrics>,
}

#[derive(Debug, serde::Deserialize)]
pub struct HealthMetrics {
    pub requests_total: Option<u64>,
    pub requests_per_second: Option<f64>,
    pub latency_ms: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PipelineResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: String,
}

