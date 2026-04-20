// smctl — the one binary all four sm modes dispatch from.
//
// See README.md §Modes and ../.claude/plans/i-want-to-redo-sequential-tiger.md
// for the design. Everything below is scaffolding: the subcommands compile
// and print a banner, but the real work is TODO.

use clap::{Parser, Subcommand};

mod agent;
mod config;
mod cookie;
mod ee_config;
mod factory;
mod gcloud;
mod ita;
mod provision_workload;
mod register;
mod watchdog;

#[derive(Parser, Debug)]
#[command(name = "smctl", version)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Run the factory HTTP service + watchdog (on the factory TDX VM).
    Factory,
    /// Run the sm-side HTTP agent: cookie-gated reverse proxy to ttyd + /health.
    Agent,
    /// One-shot: POST /sm/register to the factory with our ITA token.
    Register,
    /// CLI Claude Code inside sm invokes to ask the factory for a child VM.
    ProvisionWorkload {
        #[arg(long)]
        spec: std::path::PathBuf,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 7200)]
        ttl_seconds: u64,
    },
    /// CLI Claude Code inside sm invokes to tear down a child VM.
    TeardownWorkload {
        #[arg(long)]
        child_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    tracing::info!(mode = ?cli.mode, "smctl starting");

    match cli.mode {
        Mode::Factory => factory::run().await,
        Mode::Agent => agent::run().await,
        Mode::Register => register::run().await,
        Mode::ProvisionWorkload {
            spec,
            model,
            ttl_seconds,
        } => provision_workload::run(spec, model, ttl_seconds).await,
        Mode::TeardownWorkload { child_id } => provision_workload::teardown(child_id).await,
    }
}
