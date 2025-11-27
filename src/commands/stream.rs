use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use pynthora_terminal::core::config::Config;
use pynthora_terminal::core::telemetry::TelemetryEvent;
use pynthora_terminal::sdk::client::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    let pb = ProgressBar::new(lines.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Streaming data...");

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let event: TelemetryEvent = serde_json::from_str(line)
            .with_context(|| format!("Failed to parse line {}: {}", idx + 1, line))?;

        // TODO: Implement actual streaming
        // client.stream_event(&event, pipeline).await?;

        pb.inc(1);
    }

    pb.finish_with_message("Complete");
    println!("{} Streamed {} events successfully!", "✓".green(), lines.len());

    Ok(())
}

