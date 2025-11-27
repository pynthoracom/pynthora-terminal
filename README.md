# 🛰️ pynthora Terminal

> Data ingestion SDK + CLI for connecting real-world robot fleets to the pynthora network.

**Current Version: v0.2.0**

## 🚀 What is this?

pynthora Terminal is a developer-focused toolkit that lets you:

- Connect sensors, robots, or legacy systems to the pynthora ingestion gateway
- Normalize telemetry using declarative pipelines (YAML or JSON)
- Stream signed events with ZK-friendly metadata builders
- Manage API keys and workspace secrets from a command-line interface
- Inspect ingestion health with a local terminal dashboard

The repo ships with both:

1. **CLI** – `pynthora-terminal` executable for daily ops
2. **SDK** – Rust library for embedding ingestion flows inside services

## 📂 Structure

```
pynthora-terminal/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── core/
│   │   ├── config.rs    # Environment + project config loader
│   │   ├── logger.rs    # Colored logger
│   │   └── telemetry.rs # Shared telemetry helpers
│   ├── sdk/
│   │   ├── client.rs    # High-level ingestion client
│   │   ├── pipelines/   # Declarative pipeline parser
│   │   └── signing.rs   # Proof + signature helpers
│   └── commands/
│       ├── init.rs      # `pynthora-terminal init`
│       ├── pipeline.rs # `pynthora-terminal pipeline push`
│       └── stream.rs    # `pynthora-terminal stream`
├── tests/
└── Cargo.toml
```

## 🛠️ Quick start

### Prerequisites

- [Rust](https://rustup.rs/) >= 1.70.0
- Cargo (comes with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/pynthora/pynthora-terminal.git
cd pynthora-terminal

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Usage

```bash
# Run CLI directly
cargo run -- init

# Or if installed globally
pynthora-terminal init
pynthora-terminal pipeline push pipeline.yaml
pynthora-terminal stream --file data.json
```

## 🔐 Configuration

pynthora Terminal reads a `.pynthorarc` file (JSON/YAML) or env vars:

- `PYNTHORA_API_KEY` – project-scoped key
- `PYNTHORA_INGEST_URL` – ingestion endpoint (default: `https://api.pynthora.network/ingest`)
- `PYNTHORA_WORKSPACE` – workspace slug for namespacing resources

Use `pynthora-terminal init` to scaffold the config interactively.

## 🧩 Commands

| Command                                     | Description                                            |
| ------------------------------------------- | ------------------------------------------------------ |
| `pynthora-terminal init`                    | Create `.pynthorarc`, generate keys, test connectivity |
| `pynthora-terminal pipeline push <file>`    | Upload or update pipeline definitions                  |
| `pynthora-terminal pipeline list`           | List all pipelines                                     |
| `pynthora-terminal pipeline show <id>`      | Show pipeline details                                  |
| `pynthora-terminal stream --file data.json` | Replay local dataset into ingestion gateway            |
| `pynthora-terminal status`                  | View ingestion metrics + health check                  |
| `pynthora-terminal keys rotate`             | Rotate API keys with automated revocation              |
| `pynthora-terminal keys show`               | Show current API key info                              |

## 🧪 Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- init

# Build for release
cargo build --release

# Check code
cargo clippy

# Format code
cargo fmt
```

## 📦 Using as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
pynthora-terminal = { path = "../pynthora-terminal" }
```

Example usage:

```rust
use pynthora_terminal::core::config::Config;
use pynthora_terminal::sdk::client::Client;

let config = Config::load(None)?;
let client = Client::new(config);
// Use client to interact with pynthora network
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📄 License

MIT © pynthora Foundation
