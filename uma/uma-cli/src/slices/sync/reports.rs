//! Report types for sync operations.
//!
//! Operations return *values*, not printed text, so the logic is testable
//! against a throwaway directory and the CLI layer decides how to render it.

use std::path::PathBuf;

/// What a push did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushReport {
    pub repo_created: bool,
    /// Set when a fallback identity had to be configured for the memory repo.
    pub identity_set: Option<&'static str>,
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    /// Short hash of the commit created by this push, if any.
    pub commit: Option<String>,
    /// The fact files that changed, so the operator can see what was committed.
    pub changed_paths: Vec<String>,
    /// `origin/main` when the push reached a remote.
    pub pushed_to: Option<String>,
    /// Guidance for the operator (no remote, nothing to sync, ...).
    pub note: Option<String>,
    /// Maintenance the push performed (wrote .gitignore, untracked the cache).
    pub housekeeping: Vec<String>,
}

/// How a pull resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullOutcome {
    /// No repository yet — `uma sync push` creates it.
    NoRepo,
    NoRemote,
    NoRemoteBranch,
    /// Local history was unborn; the branch was taken from the remote.
    Restored,
    /// Remote changes were merged in.
    Merged,
    /// Already in sync with the upstream branch.
    UpToDate,
    /// Merge stopped with unmerged paths; resolution is the operator's job.
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullReport {
    pub outcome: PullOutcome,
    /// Files Git reports as unmerged.
    pub conflicted_files: Vec<String>,
    /// Files that still contain conflict markers after the merge.
    pub conflict_markers: Vec<PathBuf>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusReport {
    pub repo_exists: bool,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub upstream: Option<String>,
    /// `(ahead, behind)` against the upstream branch.
    pub ahead_behind: Option<(usize, usize)>,
    pub changed: usize,
    pub last_commit: Option<String>,
}