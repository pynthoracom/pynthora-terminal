use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use pynthora_terminal::core::config::Config;
use std::io::{self, Write};

pub async fn run(force: bool) -> Result<()> {
    let config_path = Config::default_path();

    if config_path.exists() && !force {
        print!(
            "{} Configuration file already exists at {}. Overwrite? (y/N): ",
            "⚠".yellow(),
            config_path.display()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{} Cancelled.", "ℹ".blue());
            return Ok(());
        }
    }

    println!("{} Initializing pynthora Terminal configuration...", "ℹ".blue());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message("Setting up configuration...");

    // Collect configuration
    let api_key = prompt_input("Enter your API key: ")?;
    let workspace = prompt_input("Enter workspace name: ")?;
    let ingest_url = prompt_input_with_default(
        "Ingestion URL",
        "https://api.pynthora.network/ingest",
    )?;

    pb.finish_with_message("Configuration collected");

    let config = Config {
        api_key,
        ingest_url,
        workspace,
    };

    config.validate().context("Invalid configuration")?;
    config.save(&config_path)?;

    println!(
        "{} Configuration saved to {}",
        "✓".green(),
        config_path.display()
    );

    // Test connectivity
    println!("{} Testing connectivity...", "ℹ".blue());
    // TODO: Implement connectivity test
    println!("{} Configuration complete!", "✓".green());

    Ok(())
}

fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt.cyan());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn prompt_input_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{} (default: {}): ", prompt.cyan(), default.bright_black());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

