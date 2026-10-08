//! Thin wrapper over the git command line.
//!
//! Shelling out rather than linking a git library keeps the dependency surface
//! at zero and makes behaviour identical to what the operator would run by hand
//! — which matters, because the memory repository is meant to be inspectable and
//! fixable by the person who owns it.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Runs `git <args>` in `dir`, returning trimmed stdout.
pub fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .context("git executable not found — memory sync requires git")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(anyhow!("git {}: {}", args.join(" "), detail));
    }
    Ok(stdout)
}

/// True when `dir` is a git work tree.
pub fn is_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// What [`ensure_repo`] did.
pub struct RepoInit {
    pub created: bool,
    /// Set when a fallback identity had to be configured for this repo only.
    pub identity: Option<&'static str>,
}

/// Creates the repository if missing.
///
/// The branch is `main` explicitly (older git falls back to its own default),
/// and a *local* identity is configured only when none is reachable — without
/// one, the first commit fails with an error that has nothing to do with memory.
/// Scoped to this repository, never to the operator's global git config.
pub fn ensure_repo(dir: &Path) -> Result<RepoInit> {
    if is_repo(dir) {
        return Ok(RepoInit {
            created: false,
            identity: None,
        });
    }
    std::fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;

    if run(dir, &["init", "-b", "main"]).is_err() {
        run(dir, &["init"])?;
    }
    let identity = if run(dir, &["config", "user.name"]).is_err() {
        run(dir, &["config", "user.name", "UMA memory"])?;
        run(dir, &["config", "user.email", "uma@local"])?;
        Some("UMA memory <uma@local>")
    } else {
        None
    };

    Ok(RepoInit {
        created: true,
        identity,
    })
}

/// Writes the `.gitignore` that keeps `index.db` out of the repository.
///
/// The index is a rebuildable, machine-specific binary cache that changes on
/// every search; if it were ever committed, every future push would carry it.
/// Returns true when the file was written.
pub fn write_ignore(dir: &Path) -> Result<bool> {
    let ignore = dir.join(".gitignore");
    if ignore.exists() {
        return Ok(false);
    }
    std::fs::write(&ignore, "index.db\n")
        .with_context(|| format!("failed to write {:?}", ignore))?;
    Ok(true)
}

/// True when git already tracks `path` in this repository.
pub fn is_tracked(dir: &Path, path: &str) -> bool {
    run(dir, &["ls-files", "--error-unmatch", path]).is_ok()
}

/// Stops tracking `path` while leaving the file on disk.
pub fn untrack(dir: &Path, path: &str) -> Result<()> {
    run(dir, &["rm", "--cached", "--quiet", path])?;
    Ok(())
}

pub fn branch(dir: &Path) -> Option<String> {
    run(dir, &["symbolic-ref", "--short", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
}

/// True when the branch has at least one commit (an unborn `HEAD` does not).
pub fn has_commits(dir: &Path) -> bool {
    run(dir, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok()
}

/// The first configured remote, if any.
pub fn remote(dir: &Path) -> Option<String> {
    run(dir, &["remote"])
        .ok()?
        .lines()
        .next()
        .map(str::to_string)
}

pub fn upstream(dir: &Path) -> Option<String> {
    run(
        dir,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )
    .ok()
    .map(|s| s.trim().to_string())
}

/// True when the remote already carries `branch`.
pub fn has_remote_branch(dir: &Path, remote: &str, branch: &str) -> bool {
    let reference = format!("refs/remotes/{remote}/{branch}");
    run(dir, &["rev-parse", "--verify", "--quiet", &reference]).is_ok()
}

/// One working-tree change, as reported by `git status --porcelain`.
pub struct Change {
    /// Index status column: `A` added, `M` updated, `D` removed, `?` untracked.
    pub index: char,
    pub path: String,
}

pub fn changes(dir: &Path) -> Vec<Change> {
    let Ok(out) = run(dir, &["status", "--porcelain"]) else {
        return Vec::new();
    };
    out.lines()
        .filter(|line| line.len() >= 4)
        .map(|line| Change {
            index: line.as_bytes()[0] as char,
            path: line[3..].to_string(),
        })
        .collect()
}

/// Unmerged paths, empty when the tree is clean.
pub fn conflict_files(dir: &Path) -> Vec<String> {
    run(dir, &["diff", "--name-only", "--diff-filter=U"])
        .map(|out| out.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// Commits staged changes; `Ok(None)` when there was nothing to commit.
pub fn commit(dir: &Path, message: &str) -> Result<Option<String>> {
    match run(dir, &["commit", "-m", message]) {
        Ok(_) => Ok(Some(run(dir, &["rev-parse", "--short", "HEAD"])?)),
        Err(err) if err.to_string().contains("nothing to commit") => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn last_commit(dir: &Path) -> Option<String> {
    run(dir, &["log", "-1", "--oneline"]).ok()
}

/// `(ahead, behind)` against the upstream branch.
pub fn ahead_behind(dir: &Path) -> Option<(usize, usize)> {
    let out = run(
        dir,
        &["rev-list", "--left-right", "--count", "@{upstream}...HEAD"],
    )
    .ok()?;
    let mut parts = out.split_whitespace();
    let behind = parts.next()?.parse().ok()?;
    let ahead = parts.next()?.parse().ok()?;
    Some((ahead, behind))
}

/// Clones `url` into `path`. Test-only: it simulates a second machine, which
/// is the only place sync needs to create a work tree from scratch.
#[cfg(test)]
pub fn clone(url: &Path, path: &Path) -> Result<String> {
    let parent = path
        .parent()
        .context("clone destination has no parent directory")?;
    run(
        parent,
        &["clone", &url.to_string_lossy(), &path.to_string_lossy()],
    )
}

/// Markdown files that still contain unresolved conflict markers.
///
/// Checked after every pull, because a merge conflict inside YAML frontmatter
/// makes a fact unparsable — and the index would then silently skip it, so the
/// damage would be invisible exactly when the memory is least trustworthy.
pub fn scan_conflict_markers(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file() && entry.path().extension().is_some_and(|x| x == "md")
        })
        .filter(|entry| {
            std::fs::read_to_string(entry.path()).is_ok_and(|content| has_marker(&content))
        })
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn has_marker(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.starts_with("<<<<<<<") || line.starts_with(">>>>>>>"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_repo_creates_once_and_is_idempotent() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path().join("global");

        let first = ensure_repo(&root)?;
        assert!(first.created);
        assert!(is_repo(&root));

        let second = ensure_repo(&root)?;
        assert!(!second.created, "second call must not re-initialise");
        Ok(())
    }

    #[test]
    fn test_commit_roundtrip_tracks_add_and_delete() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path().join("global");
        ensure_repo(&root)?;

        let fact = root.join("decision").join("a.md");
        std::fs::create_dir_all(fact.parent().unwrap())?;
        std::fs::write(&fact, "---\nid: a\n---\nbody\n")?;

        // Before staging, the file is untracked.
        let untracked = changes(&root);
        assert!(untracked.iter().any(|c| c.index == '?'));

        run(&root, &["add", "-A", "."])?;
        let staged = changes(&root);
        assert!(staged.iter().any(|c| c.index == 'A'));

        let hash = commit(&root, "test commit")?;
        assert!(hash.is_some());
        assert!(has_commits(&root));
        assert!(
            changes(&root).is_empty(),
            "tree should be clean after commit"
        );

        std::fs::remove_file(&fact)?;
        run(&root, &["add", "-A", "."])?;
        assert!(changes(&root).iter().any(|c| c.index == 'D'));
        Ok(())
    }

    #[test]
    fn test_conflict_marker_scan() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path().join("global");
        std::fs::create_dir_all(&root)?;

        let clean = root.join("clean.md");
        std::fs::write(&clean, "---\nid: a\n---\n# Heading\n\ndivider ---\n")?;
        let conflicted = root.join("conflicted.md");
        std::fs::write(
            &conflicted,
            "---\n<<<<<<< HEAD\ntitle: ours\n=======\ntitle: theirs\n>>>>>>> other\n---\n",
        )?;

        let found = scan_conflict_markers(&root);
        assert_eq!(found, vec![conflicted]);
        Ok(())
    }
}
