use anyhow::Result;
use clap::Args;
use uma_core::{
    domain::{ActorEvent, Scope},
    serialization::{fact_to_markdown, markdown_to_fact},
    store::Store,
};
use walkdir::WalkDir;

#[derive(Args, Debug, Clone)]
pub struct MigrateArgs {
    /// Only report what would change without writing files
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Also rebuild the centralized FTS5 index after reformatting
    #[arg(long = "reindex")]
    pub reindex: bool,
}

/// Executes the Migrate vertical slice: rewrites existing markdown files into OKF v0.2 frontmatter.
pub fn run(args: MigrateArgs) -> Result<()> {
    let mut roots: Vec<(Scope, std::path::PathBuf)> = Vec::new();

    if let Ok(global) = Store::global() {
        roots.push((Scope::Global, global.root.clone()));
    }

    if let Some(project_name) = Store::current_project_name() {
        if let Ok(project) = Store::project(project_name.clone()) {
            roots.push((Scope::Project(project_name), project.root.clone()));
        }
    }

    let mut scanned = 0usize;
    let mut migrated = 0usize;
    let mut failed = 0usize;

    for (scope, root) in &roots {
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(root).into_iter().flatten() {
            let path = entry.path();
            if !entry.file_type().is_file() || path.extension().is_none_or(|e| e != "md") {
                continue;
            }

            scanned += 1;
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("  ! read failed {:?}: {}", path, e);
                    failed += 1;
                    continue;
                }
            };

            let mut fact = match markdown_to_fact(&content) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("  ! parse failed {:?}: {}", path, e);
                    failed += 1;
                    continue;
                }
            };

            // Migration: reattach to the canonical store root (it no longer
            // nests a project directory) and backfill OKF v0.2 provenance keys.
            fact.scope = scope.clone();

            if fact.generated.is_none() {
                fact.generated = Some(ActorEvent {
                    by: "pi-agent/1.1".to_string(),
                    at: fact.validity.since,
                });
            }
            if fact.verified.is_empty() {
                fact.verified = vec![ActorEvent {
                    by: "human:operator".to_string(),
                    at: fact.validity.since,
                }];
            }

            let rendered = match fact_to_markdown(&fact) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("  ! serialize failed {:?}: {}", path, e);
                    failed += 1;
                    continue;
                }
            };

            if rendered == content {
                continue;
            }

            if args.dry_run {
                println!("  ~ would reformat {:?}", path);
                migrated += 1;
                continue;
            }

            match std::fs::write(path, rendered) {
                Ok(()) => {
                    println!("  + reformatted {:?}", path);
                    migrated += 1;
                }
                Err(e) => {
                    eprintln!("  ! write failed {:?}: {}", path, e);
                    failed += 1;
                }
            }
        }
    }

    println!(
        "\nScanned {} file(s): {} reformatted, {} failed.{}{}",
        scanned,
        migrated,
        failed,
        if args.dry_run { " (dry run)" } else { "" },
        if args.reindex { " Reindexing..." } else { "" }
    );

    if args.reindex && !args.dry_run {
        let count = Store::reindex_all()?;
        println!("Reindexed {} fact(s) into centralized index.", count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(flatten)]
        args: MigrateArgs,
    }

    #[test]
    fn test_migrate_args_parsing() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from(["test", "--dry-run", "--reindex"])?;
        assert!(cli.args.dry_run);
        assert!(cli.args.reindex);

        let cli2 = TestCli::try_parse_from(["test"])?;
        assert!(!cli2.args.dry_run);
        assert!(!cli2.args.reindex);
        Ok(())
    }
}
