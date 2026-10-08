//! Git-backed sync for the global memory store.
//!
//! `push` commits changed fact files and publishes them; `pull` takes remote
//! changes and rebuilds the search index; `status` shows where you stand.
//!
//! **The global store only.** Project memory lives inside that project's own
//! repository and travels with it when `.uma/` is tracked — this slice must not
//! commit *or push* an operator's work repository, because a memory tool that
//! pushes someone's branch is a surprise waiting to happen.

mod git;
mod ops;
mod reports;

#[cfg(test)]
mod tests;

pub use reports::{PullOutcome, PullReport, PushReport, StatusReport};

use anyhow::Result;
use clap::{Args, Subcommand};
use uma_core::store::Store;

#[derive(Args, Debug, Clone)]
pub struct SyncArgs {
    #[command(subcommand)]
    pub command: SyncCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SyncCommand {
    /// Commit changed memory facts and push them to the configured remote
    Push(PushArgs),
    /// Take remote changes and rebuild the search index
    Pull,
    /// Show the state of the memory repository
    Status,
}

#[derive(Args, Debug, Clone)]
pub struct PushArgs {
    /// Custom commit message (a fact-count summary is generated otherwise)
    #[arg(long = "message", value_name = "TEXT")]
    pub message: Option<String>,
}

/// Executes the Sync vertical slice: carries memory between machines with git.
///
/// The repository lives at the global store root, so the committed content is
/// exactly the Markdown facts. `index.db` sits inside that directory but is a
/// rebuildable, machine-specific cache — `.gitignore` keeps it unpublished.
pub fn run(args: SyncArgs) -> Result<()> {
    let root = Store::global_root_path()?;

    match args.command {
        SyncCommand::Push(push) => {
            let report = ops::push_at(&root, push.message)?;
            print_push(&report);
        }
        SyncCommand::Pull => {
            let report = ops::pull_at(&root)?;
            print_pull(&report);
            reconcile_index(&report);
        }
        SyncCommand::Status => print_status(&ops::status_at(&root)?, &root),
    }
    Ok(())
}

/// The search index is a cache: after files change underneath it, rebuild it.
fn reconcile_index(report: &PullReport) {
    if !matches!(report.outcome, PullOutcome::Merged | PullOutcome::Restored) {
        return;
    }
    match Store::reindex_all() {
        Ok(count) => println!("  index:   rebuilt ({count} fact(s) indexed)"),
        Err(err) => println!("  index:   rebuild failed ({err}) — run `uma search \"\" --reindex`"),
    }
}

fn print_push(report: &PushReport) {
    if report.repo_created {
        println!("  repo:    initialised at the global store");
        if let Some(identity) = report.identity_set {
            println!("  identity: {identity} (this repository only)");
        }
    }
    for item in &report.housekeeping {
        println!("  fixup:   {item}");
    }
    println!(
        "  staged:  {} added, {} updated, {} removed",
        report.added, report.updated, report.removed
    );
    for (idx, path) in report.changed_paths.iter().enumerate() {
        if idx == 5 {
            println!("             … and {} more", report.changed_paths.len() - 5);
            break;
        }
        println!("             - {path}");
    }
    match &report.commit {
        Some(hash) => println!("  commit:  {hash}"),
        None => println!("  commit:  nothing to commit"),
    }
    match &report.pushed_to {
        Some(where_to) => println!("  push:    → {where_to}"),
        None => {
            if let Some(note) = &report.note {
                println!("  push:    {note}");
            }
        }
    }
}

fn print_pull(report: &PullReport) {
    println!("  pull:    {}", report.note.as_deref().unwrap_or("done"));

    if report.outcome == PullOutcome::Conflict {
        println!("  ⚠ resolve the markers in the files above, then run `uma sync push`.");
        println!("  ⚠ your local version is preserved inside each conflicted file;");
        println!("            nothing was overwritten and nothing was published.");
    }

    if !report.conflict_markers.is_empty() {
        println!(
            "  ⚠ {} file(s) still contain conflict markers; until they are fixed, \
             those facts cannot be parsed and search will skip them:",
            report.conflict_markers.len()
        );
        for path in &report.conflict_markers {
            println!("       - {}", path.display());
        }
    }
}

fn print_status(report: &StatusReport, root: &std::path::Path) {
    if !report.repo_exists {
        println!("  repo:    not initialised — `uma sync push` creates it");
        println!("  store:   {}", root.display());
        return;
    }
    println!("  branch:  {}", report.branch.as_deref().unwrap_or("?"));
    match &report.remote {
        Some(remote) => println!("  remote:  {remote}"),
        None => println!("  remote:  none configured"),
    }
    match (&report.upstream, report.ahead_behind) {
        (Some(upstream), Some((ahead, behind))) => {
            println!("  sync:    {upstream} ({ahead} ahead, {behind} behind)")
        }
        (None, _) => println!("  sync:    no upstream — `uma sync push` sets it"),
        _ => {}
    }
    println!(
        "  tree:    {}",
        if report.changed == 0 {
            "clean".to_string()
        } else {
            format!("{} changed fact(s)", report.changed)
        }
    );
    if let Some(commit) = &report.last_commit {
        println!("  last:    {commit}");
    }
}

#[cfg(test)]
mod cli_shape {
    use super::*;
    use clap::{Parser, Subcommand};

    #[test]
    fn test_sync_subcommands_parse() -> Result<(), clap::Error> {
        // Mirrors main.rs: the slice hangs off a `sync` subcommand, so argv must
        // include it — parsing `uma push` here would test the wrong shape.
        #[derive(Parser)]
        #[command(name = "uma")]
        struct Cli {
            #[command(subcommand)]
            command: Top,
        }

        #[derive(Subcommand)]
        enum Top {
            Sync(SyncArgs),
        }

        let pushed = Cli::try_parse_from(["uma", "sync", "push", "--message", "hello"])?;
        match pushed.command {
            Top::Sync(SyncArgs {
                command: SyncCommand::Push(args),
            }) => assert_eq!(args.message.as_deref(), Some("hello")),
            _ => panic!("expected sync push"),
        }
        matches!(
            Cli::try_parse_from(["uma", "sync", "pull"])?.command,
            Top::Sync(SyncArgs {
                command: SyncCommand::Pull
            })
        );
        matches!(
            Cli::try_parse_from(["uma", "sync", "status"])?.command,
            Top::Sync(SyncArgs {
                command: SyncCommand::Status
            })
        );
        Ok(())
    }
}