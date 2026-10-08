use crate::domain::{Fact, FactId, FactType, Scope};
use crate::embeddings::EmbeddingClient;
use crate::indexer::Indexer;
use crate::search::{search_hybrid, search_semantic, SearchHit, SearchMode};
use crate::serialization::{fact_to_markdown, markdown_to_fact};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Store {
    pub root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Returns the global store instance located in the user profile.
    pub fn global() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "uma", "uma")
            .context("Could not determine project directories")?;
        let global_root = proj_dirs.data_dir().join("global");
        std::fs::create_dir_all(&global_root).context("Failed to create global store directory")?;
        Ok(Self::new(global_root))
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

    /// Writes a fact to its Markdown file and immediately updates the centralized SQLite index.
    pub fn write(&self, fact: &Fact) -> Result<()> {
        let path = self.fact_path(&fact.scope, &fact.fact_type, &fact.id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create fact directory")?;
        }
        let content = fact_to_markdown(fact).context("Failed to serialize fact")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write fact to {:?}", path))?;

        if let Ok(indexer) = Self::central_indexer() {
            let _ = indexer.index_fact(fact, Some(&path));
            if let Ok(client) = EmbeddingClient::new(None) {
                let _ = indexer.vectorize_fact(fact, &client);
            }
        }

        Ok(())
    }

    /// Reads a fact from its Markdown file.
    pub fn read(&self, scope: &Scope, fact_type: &FactType, id: &FactId) -> Result<Fact> {
        let path = self.fact_path(scope, fact_type, id);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read fact from {:?}", path))?;
        markdown_to_fact(&content).context("Failed to parse fact")
    }

    /// Searches for a fact by ID across markdown files in the current store root.
    pub fn read_by_id(&self, id: &FactId) -> Result<Fact> {
        for entry in WalkDir::new(&self.root).into_iter().flatten() {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(fact) = markdown_to_fact(&content) {
                    if fact.id == *id {
                        return Ok(fact);
                    }
                }
            }
        }
        anyhow::bail!("Fact with id {} not found", id)
    }

    /// Finds a fact by ID across project store and global store.
    pub fn find_by_id(id: &FactId) -> Result<Fact> {
        if let Some(project_name) = Self::current_project_name() {
            if let Ok(project_store) = Self::project(project_name) {
                if let Ok(fact) = project_store.read_by_id(id) {
                    return Ok(fact);
                }
            }
        }
        let global_store = Self::global()?;
        global_store.read_by_id(id)
    }

    /// Performs search using BM25 keyword matching, Semantic vector search, or Hybrid RRF fusion.
    pub fn search_all(
        query: &str,
        scope: Option<&Scope>,
        fact_type: Option<&FactType>,
        mode: SearchMode,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        let indexer = Self::central_indexer()?;
        let current_project = match scope {
            Some(_) => None,
            None => Self::current_project_name(),
        };

        match mode {
            SearchMode::Keyword => {
                let hits = indexer.search_keyword(
                    query,
                    scope,
                    current_project.as_deref(),
                    fact_type,
                    limit,
                )?;
                if hits.is_empty() {
                    let indexed = Self::reindex_all()?;
                    if indexed > 0 {
                        return indexer.search_keyword(
                            query,
                            scope,
                            current_project.as_deref(),
                            fact_type,
                            limit,
                        );
                    }
                }
                Ok(hits)
            }
            SearchMode::Semantic => {
                let client = EmbeddingClient::new(None)?;
                search_semantic(
                    indexer.connection(),
                    query,
                    scope,
                    current_project.as_deref(),
                    fact_type,
                    limit,
                    &client,
                )
            }
            SearchMode::Hybrid => {
                if let Ok(client) = EmbeddingClient::new(None) {
                    search_hybrid(
                        &indexer,
                        query,
                        scope,
                        current_project.as_deref(),
                        fact_type,
                        limit,
                        &client,
                    )
                } else {
                    indexer.search_keyword(
                        query,
                        scope,
                        current_project.as_deref(),
                        fact_type,
                        limit,
                    )
                }
            }
        }
    }

    /// Rebuilds the centralized SQLite index by scanning global and project stores.
    pub fn reindex_all() -> Result<usize> {
        let indexer = Self::central_indexer()?;
        let mut dirs_to_scan = Vec::new();

        if let Ok(global_store) = Self::global() {
            dirs_to_scan.push(global_store.root);
        }

        if let Ok(git_root) = Self::find_git_root() {
            let project_uma = git_root.join(".uma");
            if project_uma.exists() {
                dirs_to_scan.push(project_uma);
            }
        }

        indexer.reindex_from_dirs(&dirs_to_scan)
    }

    /// Generates vector embeddings for all facts that are currently missing them.
    pub fn vectorize_all() -> Result<usize> {
        let indexer = Self::central_indexer()?;
        let client = EmbeddingClient::new(None)?;
        indexer.vectorize_missing(&client)
    }

    /// Lists facts from this store filtered by scope and fact type.
    pub fn list(&self, _scope: &Scope, fact_type: Option<&FactType>) -> Result<Vec<Fact>> {
        let mut facts = Vec::new();
        let search_root = match fact_type {
            Some(ft) => self.root.join(ft.dir_name()),
            None => self.root.clone(),
        };

        if !search_root.exists() {
            return Ok(facts);
        }

        for entry in WalkDir::new(search_root).into_iter().flatten() {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(fact) = markdown_to_fact(&content) {
                    facts.push(fact);
                }
            }
        }

        facts.sort_by(|a, b| b.validity.since.cmp(&a.validity.since));
        Ok(facts)
    }

    /// Deletes a fact and updates the centralized SQLite index.
    pub fn delete(&self, scope: &Scope, fact_type: &FactType, id: &FactId) -> Result<()> {
        let path = self.fact_path(scope, fact_type, id);
        if path.exists() {
            std::fs::remove_file(path).context("Failed to delete fact file")?;
        }
        if let Ok(indexer) = Self::central_indexer() {
            let _ = indexer.remove_fact(id);
        }
        Ok(())
    }
}
