use anyhow::{Context, Result};
use colored::*;
use pynthora_terminal::core::config::Config;
use pynthora_terminal::core::validation::validate_pipeline;
use pynthora_terminal::sdk::client::Client;
use serde_json::Value;
use std::fs;

pub async fn push(file: &str) -> Result<()> {
    let config = Config::load(None)?;
    let client = Client::new(config);

    println!("{} Reading pipeline from {}...", "ℹ".blue(), file);

    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file))?;

    let pipeline: Value = if file.ends_with(".yaml") || file.ends_with(".yml") {
        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML: {}", file))?
    } else {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON: {}", file))?
    };

    println!("{} Validating pipeline...", "ℹ".blue());
    
    // Enhanced validation (v0.2.0)
    let validation = validate_pipeline(&pipeline);
    
    if !validation.is_valid {
        println!("{} Validation failed:", "✗".red());
        for error in &validation.errors {
            println!("  - {}", error);
        }
        anyhow::bail!("Pipeline validation failed");
    }

    if !validation.warnings.is_empty() {
        println!("{} Validation warnings:", "⚠".yellow());
        for warning in &validation.warnings {
            println!("  - {}", warning);
        }
    }

    println!("{} Pipeline validation passed!", "✓".green());
    println!("{} Pushing pipeline to server...", "ℹ".blue());
    
    let result = client.push_pipeline(&pipeline).await
        .with_context(|| "Failed to push pipeline")?;

    println!("{} Pipeline pushed successfully!", "✓".green());
    println!("  ID: {}", result.id);
    println!("  Name: {}", result.name);
    println!("  Version: {}", result.version);
    println!("  Status: {}", result.status);

    Ok(())
}

pub async fn list() -> Result<()> {
    let _config = Config::load(None)?;
    println!("{} Listing pipelines...", "ℹ".blue());
    // TODO: Implement list with actual API call
    println!("{} No pipelines found", "ℹ".yellow());
    Ok(())
}

pub async fn show(id: &str) -> Result<()> {
    let _config = Config::load(None)?;
    println!("{} Showing pipeline: {}", "ℹ".blue(), id);
    // TODO: Implement show with actual API call
    println!("{} Pipeline not found", "✗".red());
    Ok(())
}
