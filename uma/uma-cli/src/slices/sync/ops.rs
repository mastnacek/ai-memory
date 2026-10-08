//! Sync operations: what push, pull and status actually do.
//!
//! Every operation takes the repository root explicitly rather than resolving
//! `Store::global_root_path()` itself, so the logic is testable against a
//! throwaway directory and can never touch live memory from a test.

use super::git;
use super::reports::{PullOutcome, PullReport, PushReport, StatusReport};
use anyhow::Result;
use std::path::Path;

// --------------------------------------------------------------------- push

/// Stages fact files, commits them, and pushes when a remote is configured.
///
/// A push with no remote still creates history locally, so adding a remote
/// later publishes everything in one go rather than losing it.
pub fn push_at(root: &Path, message: Option<String>) -> Result<PushReport> {
    let init = git::ensure_repo(root)?;
    let mut report = PushReport {
        repo_created: init.created,
        identity_set: init.identity,
        added: 0,
        updated: 0,
        removed: 0,
        commit: None,
        changed_paths: Vec::new(),
        pushed_to: None,
        note: None,
        housekeeping: Vec::new(),
    };

    // Defence in depth for the rebuildable cache: ignore it, and untrack it if
    // an older version of this slice ever committed it.
    if git::write_ignore(root)? {
        report
            .housekeeping
            .push("wrote .gitignore (index.db is a rebuildable cache)".to_string());
    }
    if git::is_tracked(root, "index.db") {
        git::untrack(root, "index.db")?;
        report
            .housekeeping
            .push("untracked index.db - the search cache must not be published".to_string());
    }

    git::run(root, &["add", "-A", "."])?;
    for change in git::changes(root) {
        // Housekeeping is maintenance, not memory: the .gitignore written
        // below, and an index.db being untracked, would otherwise make
        // "N fact(s) changed" a lie.
        if matches!(change.path.as_str(), ".gitignore" | "index.db") {
            continue;
        }
        match change.index {
            'A' => report.added += 1,
            'D' => report.removed += 1,
            _ => report.updated += 1,
        }
        report.changed_paths.push(change.path);
    }

    let total = report.added + report.updated + report.removed;
    if total > 0 {
        let summary = format!(
            "memory sync: {} fact(s) changed ({} added, {} updated, {} removed)",
            total, report.added, report.updated, report.removed
        );
        report.commit = git::commit(root, message.as_deref().unwrap_or(&summary))?;
    }

    if !git::has_commits(root) {
        report.note = Some("nothing to sync yet".to_string());
        return Ok(report);
    }

    match git::remote(root) {
        None => {
            report.note = Some(format!(
                "no remote configured — history is local only; add one with \
                 `git -C \"{}\" remote add origin <url>`",
                root.display()
            ));
        }
        Some(remote_name) => {
            let branch = git::branch(root).unwrap_or_else(|| "main".to_string());
            // The first push sets the upstream, which is what later pulls use.
            let args: Vec<&str> = if git::upstream(root).is_none() {
                vec!["push", "-u", remote_name.as_str(), "HEAD"]
            } else {
                vec!["push"]
            };
            git::run(root, &args)?;
            report.pushed_to = Some(format!("{remote_name}/{branch}"));
        }
    }

    Ok(report)
}

// --------------------------------------------------------------------- pull

fn empty_report() -> PullReport {
    PullReport {
        outcome: PullOutcome::NoRepo,
        conflicted_files: Vec::new(),
        conflict_markers: Vec::new(),
        note: None,
    }
}

/// Pulls remote changes into the memory repository.
///
/// Merge conflicts are never resolved automatically and the operation stops,
/// because a conflict inside YAML frontmatter makes a fact unparsable — and the
/// index would then skip it silently. Local files are left exactly as they are.
pub fn pull_at(root: &Path) -> Result<PullReport> {
    if !git::is_repo(root) {
        let mut report = empty_report();
        report.outcome = PullOutcome::NoRepo;
        report.note = Some("no memory repository yet — `uma sync push` creates it".to_string());
        return Ok(report);
    }

    let Some(remote_name) = git::remote(root) else {
        let mut report = empty_report();
        report.outcome = PullOutcome::NoRemote;
        report.note = Some("no remote configured; nothing to pull".to_string());
        return Ok(report);
    };

    git::run(root, &["fetch", remote_name.as_str()])?;
    let branch = git::branch(root).unwrap_or_else(|| "main".to_string());
    if !git::has_remote_branch(root, &remote_name, &branch) {
        let mut report = empty_report();
        report.outcome = PullOutcome::NoRemoteBranch;
        report.note = Some(format!("remote {remote_name} has no {branch} branch; nothing to pull"));
        return Ok(report);
    }
    let remote_ref = format!("{remote_name}/{branch}");

    // An unborn branch (fresh machine, empty memory) cannot merge: take the
    // remote branch wholesale instead.
    if !git::has_commits(root) {
        git::run(root, &["checkout", "-B", branch.as_str(), &remote_ref])?;
        return Ok(PullReport {
            outcome: PullOutcome::Restored,
            conflicted_files: Vec::new(),
            conflict_markers: git::scan_conflict_markers(root),
            note: Some(format!("memory restored from {remote_ref}")),
        });
    }

    // Skip the merge entirely when there is nothing to take.
    if let Some((ahead, behind)) = git::ahead_behind(root) {
        if behind == 0 {
            let mut report = empty_report();
            report.outcome = PullOutcome::UpToDate;
            report.note =
                Some(format!("in sync with {remote_ref} ({ahead} unpushed commit(s))"));
            return Ok(report);
        }
    }

    match git::run(root, &["merge", "--no-edit", &remote_ref]) {
        Err(err) => {
            let mut report = empty_report();
            report.outcome = PullOutcome::Conflict;
            report.conflicted_files = git::conflict_files(root);
            report.conflict_markers = git::scan_conflict_markers(root);
            report.note = Some(format!(
                "merge stopped: {err}. Resolve the files above, then run `uma sync push`."
            ));
            Ok(report)
        }
        Ok(_) => Ok(PullReport {
            outcome: PullOutcome::Merged,
            conflicted_files: Vec::new(),
            conflict_markers: git::scan_conflict_markers(root),
            note: Some(format!("merged {remote_ref}")),
        }),
    }
}

// ------------------------------------------------------------------- status

pub fn status_at(root: &Path) -> Result<StatusReport> {
    if !git::is_repo(root) {
        return Ok(StatusReport::default());
    }
    Ok(StatusReport {
        repo_exists: true,
        branch: git::branch(root),
        remote: git::remote(root),
        upstream: git::upstream(root),
        ahead_behind: git::ahead_behind(root),
        changed: git::changes(root).len(),
        last_commit: git::last_commit(root),
    })
}

