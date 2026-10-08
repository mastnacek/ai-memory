//! Sync operation tests: run against throwaway directories only.

use super::git;
use super::ops::{pull_at, push_at, status_at};
use super::reports::PullOutcome;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// A throwaway "memory store" with one fact and a git repo.
fn memory_root(dir: &Path, name: &str) -> PathBuf {
    let root = dir.join(name);
    let fact = root.join("decision").join("one.md");
    std::fs::create_dir_all(fact.parent().unwrap()).unwrap();
    std::fs::write(&fact, "---\nid: one\n---\nbody\n").unwrap();
    root
}

/// A bare origin, a machine A that publishes, and a machine B that clones.
fn two_machines(dir: &Path) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let origin = dir.join("origin.git");
    let machine_a = memory_root(dir, "machine-a");
    let machine_b = dir.join("machine-b");

    git::run(&machine_a, &["init", "-b", "main"])?;
    git::run(&machine_a, &["add", "-A", "."])?;
    git::commit(&machine_a, "seed")?;
    // -b main matters: a bare repo's HEAD would otherwise point at git's
    // compile-time default, and a clone of it would land on a different
    // branch than the one machine A published.
    git::run(
        origin.parent().unwrap(),
        &["init", "--bare", "-b", "main", &origin.to_string_lossy()],
    )?;
    git::run(&machine_a, &["remote", "add", "origin", &origin.to_string_lossy()])?;
    push_at(&machine_a, None)?;
    git::clone(&origin, &machine_b)?;
    Ok((origin, machine_a, machine_b))
}

fn rewrite(path: &Path, body: &str) -> Result<()> {
    std::fs::write(path, format!("---\nid: one\n---\n{body}\n"))?;
    Ok(())
}

#[test]
fn test_push_commits_locally_and_reports_no_remote() -> Result<()> {
    let dir = tempdir()?;
    let root = memory_root(dir.path(), "global");

    let report = push_at(&root, None)?;

    assert!(report.commit.is_some(), "a commit must be created");
    assert_eq!(report.added, 1);
    assert!(report.pushed_to.is_none());
    assert!(report.note.as_deref().unwrap().contains("no remote"));
    assert!(git::changes(&root).is_empty(), "tree should be clean");
    Ok(())
}

#[test]
fn test_push_without_changes_still_pushes_local_history() -> Result<()> {
    let dir = tempdir()?;
    let root = memory_root(dir.path(), "global");
    push_at(&root, None)?;

    let report = push_at(&root, None)?;
    assert_eq!(report.commit, None, "nothing new to commit");
    assert_eq!(report.added + report.updated + report.removed, 0);
    // The note is still "no remote configured", which is true - but the
    // operation must NOT claim there is nothing to sync: the history it just
    // created would be published the moment a remote is added.
    let note = report.note.as_deref().unwrap_or("");
    assert!(note.contains("no remote"), "{note}");
    assert!(!note.contains("nothing to sync"), "{note}");
    Ok(())
}

#[test]
fn test_pull_without_repo_or_remote_is_a_clean_noop() -> Result<()> {
    let dir = tempdir()?;
    let root = dir.path().join("global");

    assert_eq!(pull_at(&root)?.outcome, PullOutcome::NoRepo);
    git::ensure_repo(&root)?;
    assert_eq!(pull_at(&root)?.outcome, PullOutcome::NoRemote);
    Ok(())
}

#[test]
fn test_push_and_pull_roundtrip_between_two_machines() -> Result<()> {
    let dir = tempdir()?;
    let (_origin, machine_a, machine_b) = two_machines(dir.path())?;
    let root_a = machine_a;

    // Machine B adds a fact and publishes it.
    let fact = machine_b.join("preference").join("two.md");
    std::fs::create_dir_all(fact.parent().unwrap())?;
    std::fs::write(&fact, "---\nid: two\n---\nbody\n")?;
    git::run(&machine_b, &["add", "-A", "."])?;
    git::commit(&machine_b, "machine b adds a fact")?;
    push_at(&machine_b, None)?;

    // Machine A pulls: the new fact arrives, no conflicts.
    let report = pull_at(&root_a)?;
    assert_eq!(report.outcome, PullOutcome::Merged, "{report:?}");
    assert!(report.conflicted_files.is_empty());
    assert!(report.conflict_markers.is_empty());
    assert!(root_a.join("preference").join("two.md").exists());

    // A further pull is a clean no-op.
    assert_eq!(pull_at(&root_a)?.outcome, PullOutcome::UpToDate);
    Ok(())
}

#[test]
fn test_conflicting_pull_stops_and_never_discards_local_memory() -> Result<()> {
    let dir = tempdir()?;
    let (_origin, machine_a, machine_b) = two_machines(dir.path())?;
    let root_a = machine_a.clone();
    let fact_a = root_a.join("decision").join("one.md");

    // Both machines rewrite the same fact independently. A keeps its version
    // local; B publishes straight away (origin is only at the seed, so B's
    // push is a fast-forward).
    rewrite(&fact_a, "version A")?;
    git::run(&root_a, &["add", "-A", "."])?;
    git::commit(&root_a, "a edits one.md")?;

    let fact_b = machine_b.join("decision").join("one.md");
    rewrite(&fact_b, "version B")?;
    git::run(&machine_b, &["add", "-A", "."])?;
    git::commit(&machine_b, "b edits one.md")?;
    push_at(&machine_b, None)?;

    // A's pull now diverges: local "version A" against remote "version B".
    let report = pull_at(&root_a)?;
    assert_eq!(report.outcome, PullOutcome::Conflict, "{report:?}");
    assert!(report.conflicted_files.iter().any(|f| f.contains("one.md")));
    // Git preserves the local version inside the conflicted file, which the
    // report must surface - the operator resolves it, never the tool.
    let local = std::fs::read_to_string(&fact_a)?;
    assert!(local.contains("version A"), "{local:?}");
    assert!(report.conflict_markers.iter().any(|p| p == &fact_a));
    Ok(())
}

#[test]
fn test_status_reports_a_clean_published_repo() -> Result<()> {
    let dir = tempdir()?;
    let root = memory_root(dir.path(), "global");
    push_at(&root, None)?;

    let report = status_at(&root)?;
    assert!(report.repo_exists);
    assert_eq!(report.branch.as_deref(), Some("main"));
    assert_eq!(report.changed, 0);
    assert!(report.last_commit.is_some());
    Ok(())
}

#[test]
fn test_status_before_any_sync_reports_no_repo() -> Result<()> {
    let dir = tempdir()?;
    let report = status_at(&dir.path().join("global"))?;
    assert!(!report.repo_exists);
    assert!(report.branch.is_none());
    Ok(())
}

#[test]
fn test_push_never_tracks_the_search_cache() -> Result<()> {
    // Regression: index.db lives INSIDE the global root, so a bare
    // `git add -A .` used to commit a binary cache that changes on every
    // search - every future push would be dirty, and the repository would
    // carry megabytes of machine-specific noise.
    let dir = tempdir()?;
    let root = memory_root(dir.path(), "global");
    std::fs::write(root.join("index.db"), "not really sqlite")?;

    let report = push_at(&root, None)?;

    assert!(!git::is_tracked(&root, "index.db"), "cache must stay untracked");
    assert!(root.join(".gitignore").exists());
    assert!(report
        .housekeeping
        .iter()
        .any(|item| item.contains(".gitignore")));
    // Everything that *is* published must be a fact document or the ignore
    // file itself; the cache must never appear.
    let tracked = git::run(&root, &["ls-files"])?;
    assert!(tracked
        .lines()
        .all(|p| p.ends_with(".md") || p == ".gitignore"), "{tracked}");
    Ok(())
}

#[test]
fn test_push_untracks_a_cache_an_older_version_committed() -> Result<()> {
    // Heal repositories created before the ignore existed.
    let dir = tempdir()?;
    let root = memory_root(dir.path(), "global");
    std::fs::write(root.join("index.db"), "stale cache")?;
    git::ensure_repo(&root)?;
    git::run(&root, &["add", "-A", "."])?;
    git::commit(&root, "old build committed the cache")?;
    assert!(git::is_tracked(&root, "index.db"), "setup: cache is tracked");

    let report = push_at(&root, None)?;

    assert!(report
        .housekeeping
        .iter()
        .any(|item| item.contains("untracked")));
    assert!(!git::is_tracked(&root, "index.db"));
    assert!(root.join("index.db").exists(), "the file itself stays on disk");
    Ok(())
}