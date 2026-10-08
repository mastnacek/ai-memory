use crate::domain::{FactId, FactType, Scope};
use crate::indexer::Indexer;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

mod lookup;

pub use lookup::{scope_summaries_from, ScopeSummary};
mod ops;
mod search;

pub struct Store {
    pub root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Returns the canonical global store root (without creating it).
    ///
    /// Public so health checks can inspect the location without opening it.
    pub fn global_root_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "uma", "uma")
            .context("Could not determine project directories")?;
        Ok(proj_dirs.data_dir().join("global"))
    }

    /// Returns the global store instance located in the user profile.
    pub fn global() -> Result<Self> {
        let global_root = Self::global_root_path()?;
        std::fs::create_dir_all(&global_root).context("Failed to create global store directory")?;
        Ok(Self::new(global_root))
    }

    /// Whether this store is a canonical root (global or the current project's `.uma`).
    ///
    /// Only canonical stores feed the shared central index. Ad-hoc
    /// `Store::new(path)` instances (tests, temp dirs) write Markdown only, so
    /// they can never pollute the production search index with orphan rows.
    pub fn is_canonical(&self) -> bool {
        if let Ok(global_root) = Self::global_root_path() {
            if self.root == global_root {
                return true;
            }
        }
        if let Ok(git_root) = Self::find_git_root() {
            if self.root == git_root.join(".uma") {
                return true;
            }
        }
        false
    }

    /// Returns the project store instance for the current git repository.
    pub fn project(_project_name: String) -> Result<Self> {
        let root = Self::find_git_root()?.join(".uma");
        std::fs::create_dir_all(&root).context("Failed to create project store directory")?;
        Ok(Self::new(root))
    }

    /// Resolves the current git repository root.
    pub fn find_git_root() -> Result<PathBuf> {
        let mut current = std::env::current_dir().context("Failed to get current directory")?;
        loop {
            if current.join(".git").exists() {
                return Ok(current);
            }
            if !current.pop() {
                anyhow::bail!("Not inside a git repository");
            }
        }
    }

    /// Returns the name of the current project if inside a git repository.
    pub fn current_project_name() -> Option<String> {
        Self::find_git_root().ok().and_then(|git_root| {
            git_root
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
        })
    }

    /// Returns the path to the single centralized SQLite index database in the user profile.
    pub fn central_db_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "uma", "uma")
            .context("Could not determine project directories")?;
        Ok(proj_dirs.data_dir().join("index.db"))
    }

    /// Opens the single centralized SQLite indexer.
    pub fn central_indexer() -> Result<Indexer> {
        Indexer::open(Self::central_db_path()?)
    }

    fn fact_path(&self, _scope: &Scope, fact_type: &FactType, id: &FactId) -> PathBuf {
        self.root
            .join(fact_type.dir_name())
            .join(format!("{}.md", id))
    }
}
