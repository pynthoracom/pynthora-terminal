use anyhow::Result;
use clap::{Parser, Subcommand};
use pynthora_terminal::commands::{init, pipeline, stream};
use pynthora_terminal::core::config::Config;
use std::process;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "pynthora-terminal")]
#[command(about = "pynthora Terminal CLI - data ingestion toolkit", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to custom .pynthorarc file
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize pynthora terminal configuration
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
    /// Manage data ingestion pipelines
    Pipeline {
        #[command(subcommand)]
        subcommand: PipelineCommands,
    },
    /// Stream data to ingestion gateway
    Stream {
        /// Input file path
        #[arg(short, long)]
        file: String,
        /// Pipeline ID to use
        #[arg(short, long)]
        pipeline: Option<String>,
    },
    /// Check ingestion status and health
    Status {
        /// Show detailed metrics
        #[arg(short, long)]
        verbose: bool,
    },
    /// Manage API keys
    Keys {
        #[command(subcommand)]
        subcommand: KeyCommands,
    },
}

#[derive(Subcommand)]
enum PipelineCommands {
    /// Push pipeline definition to server
    Push {
        /// Pipeline definition file (YAML or JSON)
        file: String,
    },
    /// List all pipelines
    List,
    /// Show pipeline details
    Show {
        /// Pipeline ID
        id: String,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// Rotate API key
    Rotate {
        /// Force rotation without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Show current API key info
    Show,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();

    // Load config if needed (skip for init command)
    if !matches!(cli.command, Commands::Init { .. }) {
        if let Err(e) = Config::load(cli.config.as_deref()) {
            error!("Failed to load configuration: {}", e);
            error!("Run 'pynthora-terminal init' to create a configuration file");
            process::exit(1);
        }
    }

    let result = match cli.command {
        Commands::Init { force } => init::run(force).await,
        Commands::Pipeline { subcommand } => match subcommand {
            PipelineCommands::Push { file } => pipeline::push(&file).await,
            PipelineCommands::List => pipeline::list().await,
            PipelineCommands::Show { id } => pipeline::show(&id).await,
        },
        Commands::Stream { file, pipeline } => stream::run(&file, pipeline.as_deref()).await,
        Commands::Status { verbose } => status::run(verbose).await,
        Commands::Keys { subcommand } => match subcommand {
            KeyCommands::Rotate { force } => keys::rotate(force).await,
            KeyCommands::Show => keys::show().await,
        },
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        process::exit(1);
    }
}

// Status command with real-time health monitoring (v0.2.0)
mod status {
    use anyhow::{Context, Result};
    use colored::*;
    use indicatif::{ProgressBar, ProgressStyle};
    use pynthora_terminal::core::config::Config;
    use pynthora_terminal::sdk::client::Client;
    use std::time::Duration;
    use tokio::time::sleep;

    pub async fn run(verbose: bool) -> Result<()> {
        let config = Config::load(None)?;
        let client = Client::new(config);

        println!("{} Checking pynthora terminal health...", "ℹ".blue());

        let health = client.health_check().await
            .context("Failed to check health status")?;

        println!("\n{} Health Status", "=".cyan().bold());
        println!("  Status: {}", 
            if health.status == "healthy" { 
                "✓ Healthy".green() 
            } else { 
                format!("✗ {}", health.status).red() 
            }
        );

        if let Some(version) = health.version {
            println!("  Version: {}", version);
        }

        if let Some(uptime) = health.uptime {
            let hours = uptime / 3600;
            let minutes = (uptime % 3600) / 60;
            println!("  Uptime: {}h {}m", hours, minutes);
        }

        if verbose {
            if let Some(metrics) = health.metrics {
                println!("\n{} Metrics", "=".cyan().bold());
                if let Some(total) = metrics.requests_total {
                    println!("  Total Requests: {}", total);
                }
                if let Some(rps) = metrics.requests_per_second {
                    println!("  Requests/sec: {:.2}", rps);
                }
                if let Some(latency) = metrics.latency_ms {
                    println!("  Avg Latency: {:.2}ms", latency);
                }
            }
        }

        // Real-time monitoring mode (v0.2.0)
        if verbose {
            println!("\n{} Starting real-time monitoring (Ctrl+C to stop)...", "ℹ".blue());
            
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} Monitoring... {msg}")
                    .unwrap(),
            );

            loop {
                match client.health_check().await {
                    Ok(health) => {
                        let status_icon = if health.status == "healthy" { "✓" } else { "✗" };
                        let status_color = if health.status == "healthy" { "green" } else { "red" };
                        
                        let mut msg = format!("{} Status: {}", status_icon, health.status);
                        if let Some(metrics) = &health.metrics {
                            if let Some(rps) = metrics.requests_per_second {
                                msg.push_str(&format!(" | RPS: {:.2}", rps));
                            }
                        }
                        
                        pb.set_message(msg);
                    }
                    Err(e) => {
                        pb.set_message(format!("✗ Error: {}", e));
                    }
                }
                
                sleep(Duration::from_secs(2)).await;
            }
        }

        Ok(())
    }
}

mod keys {
    use anyhow::Result;
    use tracing::info;

    pub async fn rotate(_force: bool) -> Result<()> {
        info!("Key rotation - coming soon");
        Ok(())
    }

    pub async fn show() -> Result<()> {
        info!("Show key - coming soon");
        Ok(())
    }
}

