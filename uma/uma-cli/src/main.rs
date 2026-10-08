use anyhow::Result;
use clap::{Parser, Subcommand};

mod shared;
mod slices;

#[derive(Parser)]
#[command(
    name = "uma",
    version,
    about = "Universal Memory Architecture - Local-first agent memory system"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write a new fact
    Write(slices::write::WriteArgs),

    /// Read a fact by ID
    Read(slices::read::ReadArgs),

    /// List facts
    List(slices::list::ListArgs),

    /// Search facts using BM25 keyword, semantic vectors, or hybrid RRF
    Search(slices::search::SearchArgs),

    /// Supersede an existing fact with a new revision (chains supersession)
    Supersede(slices::supersede::SupersedeArgs),

    /// Rewrite existing markdown files into the OKF v0.2 frontmatter format
    Migrate(slices::migrate::MigrateArgs),

    /// Propose merges for near-duplicate facts and flag contradicting facts
    Consolidate(slices::consolidate::ConsolidateArgs),

    /// Procedural skill memory: store and expand invocation templates
    Skill(slices::skill::SkillArgs),

    /// Serve memory over the Model Context Protocol (stdio JSON-RPC)
    Mcp(slices::mcp::McpArgs),

    /// Reconstruct how facts evolved: supersession chains, oldest revision first
    Timeline(slices::timeline::TimelineArgs),

    /// Export memory as an OKF bundle or JSON
    Export(slices::export::ExportArgs),

    /// Read-only health report on the store and its index cache
    Doctor(slices::doctor::DoctorArgs),

    /// Carry memory between machines with git (the global store)
    Sync(slices::sync::SyncArgs),

    /// List the memory scopes that exist (cross-project discovery)
    Scopes(slices::scopes::ScopesArgs),

    /// File pain score from correction and git history (read-only)
    Risk(slices::risk::RiskArgs),

    /// Decide whether a message needs memory recall (read-only; S3 gate)
    Recall(slices::recall::RecallArgs),

    /// Scan text for credentials and injection signatures (read-only)
    Secrets(slices::secrets::SecretsArgs),

    /// Browse the pi and Claude Code session stores (read-only)
    Sessions(slices::sessions::SessionsArgs),

    /// Extract memory candidates from agent sessions (proposals only)
    Import(slices::import::ImportArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Write(args) => slices::write::run(args),
        Commands::Read(args) => slices::read::run(args),
        Commands::List(args) => slices::list::run(args),
        Commands::Search(args) => slices::search::run(args),
        Commands::Supersede(args) => slices::supersede::run(args),
        Commands::Migrate(args) => slices::migrate::run(args),
        Commands::Consolidate(args) => slices::consolidate::run(args),
        Commands::Skill(args) => slices::skill::run(args),
        Commands::Mcp(args) => slices::mcp::run(args),
        Commands::Timeline(args) => slices::timeline::run(args),
        Commands::Export(args) => slices::export::run(args),
        Commands::Doctor(args) => slices::doctor::run(args),
        Commands::Sync(args) => slices::sync::run(args),
        Commands::Scopes(args) => slices::scopes::run(args),
        Commands::Risk(args) => slices::risk::run(args),
        Commands::Recall(args) => slices::recall::run(args),
        Commands::Secrets(args) => slices::secrets::run(args),
        Commands::Sessions(args) => slices::sessions::run(args),
        Commands::Import(args) => slices::import::run(args),
    }
}
