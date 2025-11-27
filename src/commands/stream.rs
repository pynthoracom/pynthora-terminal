use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use pynthora_terminal::core::config::Config;
use pynthora_terminal::core::retry::{retry_with_backoff, RetryConfig};
use pynthora_terminal::core::validation::validate_batch;
use pynthora_terminal::core::telemetry::TelemetryEvent;
use pynthora_terminal::sdk::client::Client;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use tracing::{debug, info, warn};

const DEFAULT_BATCH_SIZE: usize = 100;

pub async fn run(file: &str, pipeline: Option<&str>) -> Result<()> {
    let config = Config::load(None)?;
    let client = Client::new(config);

    println!("{} Reading data from {}...", "ℹ".blue(), file);

    let file_handle = File::open(file)
        .with_context(|| format!("Failed to open file: {}", file))?;

    let reader = BufReader::new(file_handle);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to read file")?;

    let total_lines = lines.len();
    let pb = ProgressBar::new(total_lines as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Streaming data...");

    // Parse all events first
    let mut events = Vec::new();
    let mut parse_errors = 0;

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<Value>(line) {
            Ok(event) => events.push(event),
            Err(e) => {
                warn!("Failed to parse line {}: {}", idx + 1, e);
                parse_errors += 1;
            }
        }
    }

    if parse_errors > 0 {
        println!(
            "{} {} lines failed to parse",
            "⚠".yellow(),
            parse_errors
        );
    }

    // Validate batch
    let validation = validate_batch(&events);
    if !validation.is_valid {
        println!("{} Validation errors found:", "⚠".yellow());
        for error in &validation.errors {
            println!("  - {}", error);
        }
        if !validation.errors.is_empty() {
            anyhow::bail!("Batch validation failed");
        }
    }

    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            println!("{} {}", "⚠".yellow(), warning);
        }
    }

    // Process in batches
    let batch_size = DEFAULT_BATCH_SIZE;
    let mut successful = 0;
    let mut failed = 0;

    for batch in events.chunks(batch_size) {
        let batch_num = (batch.len() + batch_size - 1) / batch_size;
        pb.set_message(&format!("Processing batch {}...", batch_num));

        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        };

        match retry_with_backoff(&retry_config, || async {
            client.stream_batch(batch, pipeline).await
        })
        .await
        {
            Ok(_) => {
                successful += batch.len();
                debug!("Batch {} processed successfully", batch_num);
            }
            Err(e) => {
                failed += batch.len();
                warn!("Batch {} failed: {}", batch_num, e);
                // Continue with next batch instead of failing completely
            }
        }

        pb.inc(batch.len() as u64);
    }

    pb.finish_with_message("Complete");

    if successful > 0 {
        println!(
            "{} Streamed {} events successfully!",
            "✓".green(),
            successful
        );
    }

    if failed > 0 {
        println!("{} {} events failed to stream", "✗".red(), failed);
    }

    Ok(())
}

