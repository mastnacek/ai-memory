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
    }
}
