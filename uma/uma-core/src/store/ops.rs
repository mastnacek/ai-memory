//! Fact operations: persist, revise, read, list, delete.

use crate::domain::{Fact, FactId, FactStatus, FactType, Scope};
use crate::embeddings::EmbeddingClient;
use crate::serialization::{fact_to_markdown, markdown_to_fact};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use walkdir::WalkDir;

use super::Store;

impl Store {
    /// Writes a fact to its Markdown file and, for canonical stores, updates
    /// the shared central index.
    pub fn write(&self, fact: &Fact) -> Result<()> {
        let path = self.fact_path(&fact.scope, &fact.fact_type, &fact.id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create fact directory")?;
        }
        let content = fact_to_markdown(fact).context("Failed to serialize fact")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write fact to {:?}", path))?;

        if self.is_canonical() {
            if let Ok(indexer) = Self::central_indexer() {
                let _ = indexer.index_fact(fact, Some(&path));
                if let Ok(client) = EmbeddingClient::new(None) {
                    let _ = indexer.vectorize_fact(fact, &client);
                }
            }
        }

        Ok(())
    }

    /// Retires a fact **within this store**: deprecate it and close its validity.
    fn retire(&self, old_id: &FactId, at: DateTime<Utc>) -> Result<()> {
        let mut old_fact = self.read_by_id(old_id)?;
        old_fact.status = FactStatus::Deprecated;
        old_fact.validity.until = Some(at);
        self.write(&old_fact)
    }

    /// Writes a revision chained to `old_id` **within this store**.
    ///
    /// Revision history is dated by each fact's `generated.at` (see the
    /// timeline); the claim's `since` passes through untouched.
    fn store_revision(&self, old_id: &FactId, mut new_fact: Fact) -> Result<Fact> {
        new_fact.supersedes = Some(*old_id);
        // The claim's origin (`since`) is the caller's decision: a revision
        // restates an existing claim, so it inherits the predecessor's origin
        // unless the caller explicitly overrides it (e.g. an import restoring
        // a session's date). Resetting it here silently moved every claim's
        // origin to the moment of its latest revision.
        new_fact.status = FactStatus::Stable;
        self.write(&new_fact)?;
        Ok(new_fact)
    }

    /// Supersedes a fact confined to a **single** store: both the retirement and
    /// the revision happen here, and no other root is ever resolved.
    ///
    /// Use this when the caller already knows which store holds the fact — an
    /// isolated store in a test, or an importer walking another system's data.
    /// Unlike [`Store::supersede`] it cannot reach outside `self`, so it is safe
    /// to call against a temporary store.
    pub fn supersede_within(&self, old_id: &FactId, new_fact: Fact) -> Result<Fact> {
        let now = Utc::now();
        self.retire(old_id, now)?;
        self.store_revision(old_id, new_fact)
    }

    /// Supersedes a fact whose location is unknown: finds the predecessor, retires
    /// it in its own store, then writes the revision into `self`.
    ///
    /// Because the two stores are resolved independently, a supersession may move
    /// a fact between scopes (project → global, or the reverse).
    pub fn supersede(&self, old_id: &FactId, new_fact: Fact) -> Result<Fact> {
        let old_fact = Self::find_by_id(old_id)?;
        let old_store = match &old_fact.scope {
            Scope::Global => Self::global()?,
            Scope::Project(p) => Self::project(p.clone())?,
        };

        let now = Utc::now();
        old_store.retire(old_id, now)?;
        self.store_revision(old_id, new_fact)
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
        if let Ok(fact) = global_store.read_by_id(id) {
            return Ok(fact);
        }

        // A ULID is unambiguous, so a miss in the current project and global
        // scope is not evidence of absence: the fact may live in *another*
        // project. The index knows where every indexed fact's file is, so use
        // it — read-only, and a miss stays a clean miss.
        let indexer = Self::central_indexer()?;
        if let Some(fact) = super::lookup::indexed_fact_from(indexer.connection(), id)? {
            return Ok(fact);
        }

        anyhow::bail!("Fact with id {} not found", id)
    }

    /// Lists facts from this store filtered by scope and fact type.
    pub fn list(&self, scope: &Scope, fact_type: Option<&FactType>) -> Result<Vec<Fact>> {
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
                    // Scope is a property of the fact, not of the directory it
                    // happens to sit in: imports and cross-store supersession
                    // can place a foreign-scope fact here, and listing this
                    // store must answer with the scope that was asked for.
                    if fact.scope == *scope {
                        facts.push(fact);
                    }
                }
            }
        }

        facts.sort_by(|a, b| b.validity.since.cmp(&a.validity.since));
        Ok(facts)
    }

    /// Deletes a fact from disk and, for canonical stores, from the shared index.
    pub fn delete(&self, scope: &Scope, fact_type: &FactType, id: &FactId) -> Result<()> {
        let path = self.fact_path(scope, fact_type, id);
        if path.exists() {
            std::fs::remove_file(path).context("Failed to delete fact file")?;
        }
        if self.is_canonical() {
            if let Ok(indexer) = Self::central_indexer() {
                let _ = indexer.remove_fact(id);
            }
        }
        Ok(())
    }
}
