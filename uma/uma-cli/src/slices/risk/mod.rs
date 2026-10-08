//! Risk: the file pain score as a user-visible surface.
//!
//! **Read-only.** Gathering is the slice's job (memory lookups, git history),
//! scoring is the kernel's pure math. The score is advisory — it feeds
//! warnings in the interceptor and this report; nothing is ever blocked or
//! modified here.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use uma_core::domain::{Fact, FactType, Scope};
use uma_core::risk;
use uma_core::store::Store;

#[derive(Args, Debug, Clone)]
pub struct RiskArgs {
    #[command(subcommand)]
    pub command: RiskCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RiskCommand {
    /// Score a file's pain from its correction and git history (read-only)
    Pain(PainArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PainArgs {
    /// File to score (relative to the current directory)
    pub path: String,

    /// How many commits back to count churn
    #[arg(long = "max-commits", default_value_t = 200)]
    pub max_commits: usize,

    /// Emit the score as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Risk vertical slice.
pub fn run(args: RiskArgs) -> Result<()> {
    match args.command {
        RiskCommand::Pain(pain) => pain_score(pain),
    }
}

fn pain_score(args: PainArgs) -> Result<()> {
    let path = std::path::PathBuf::from(&args.path);
    if !path.exists() {
        anyhow::bail!("No such file: {} (run from the repository root)", args.path);
    }

    let mentions = correction_mentions(&path)?;
    let (reverts, churn) = git_history(&path, args.max_commits)?;

    let score = risk::compute(mentions, reverts, churn);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&score)?);
        return Ok(());
    }

    println!(
        "Pain score for {}: {}/100 [{}]",
        args.path,
        score.score,
        score.band.as_str()
    );
    println!(
        "  correction mentions: {} · reverts: {} · churn (last {} commits): {}",
        score.correction_mentions, score.reverts, args.max_commits, score.churn
    );
    println!("  guidance: {}", risk::guidance(score.band));
    Ok(())
}

/// Counts active correction facts that mention the file's path or stem.
///
/// Both the current project's store and the global store are scanned, since a
/// correction may live in either.
fn correction_mentions(path: &std::path::Path) -> Result<usize> {
    let needle = path.to_string_lossy().replace('\\', "/");
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let mut count = 0usize;
    for facts in project_and_global_corrections()? {
        for fact in facts {
            if fact_mentions(&fact, &needle, &stem) {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn project_and_global_corrections() -> Result<Vec<Vec<Fact>>> {
    let mut out = Vec::new();
    if let Some(project) = Store::current_project_name() {
        if let Ok(store) = Store::project(project) {
            out.push(store.list(&project_scope(&store)?, Some(&FactType::Correction))?);
        }
    }
    if let Ok(store) = Store::global() {
        out.push(store.list(&Scope::Global, Some(&FactType::Correction))?);
    }
    Ok(out)
}

fn project_scope(_store: &Store) -> Result<Scope> {
    Ok(Store::current_project_name()
        .map(Scope::Project)
        .unwrap_or(Scope::Global))
}

/// A correction mentions the file when its title or body carries the full
/// path or the bare stem (`ops.rs` matches `uma-core/src/indexer/ops.rs`).
fn fact_mentions(fact: &Fact, needle: &str, stem: &str) -> bool {
    if stem.is_empty() {
        return false;
    }
    let text = format!("{} {}", fact.title, fact.body).replace('\\', "/");
    text.contains(needle) || text.contains(stem)
}

/// Reads git history for the file: reverts by subject, churn by commit count.
fn git_history(path: &std::path::Path, max_commits: usize) -> Result<(usize, usize)> {
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--follow",
            "--oneline",
            &format!("-n {max_commits}"),
            "--",
            &path.to_string_lossy(),
        ])
        // Same fail-fast rule as sync: never hang on a credential prompt.
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .context("git not found — risk scoring needs git history")?;

    if !output.status.success() {
        // A file with no git history yet is not an error: it is simply new.
        return Ok((0, 0));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let subjects: Vec<&str> = stdout.lines().collect();
    let churn = subjects.len();
    let reverts = subjects
        .iter()
        .filter(|line| line.to_lowercase().contains("revert"))
        .count();
    Ok((reverts, churn))
}

#[cfg(test)]
mod tests;
