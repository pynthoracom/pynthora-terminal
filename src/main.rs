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

// Placeholder modules - will be implemented
mod status {
    use anyhow::Result;
    use tracing::info;

    pub async fn run(_verbose: bool) -> Result<()> {
        info!("Status command - coming soon");
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

