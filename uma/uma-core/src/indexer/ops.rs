//! Index maintenance: indexing, reindexing, embeddings.

use crate::domain::{Fact, FactId, Scope};
use crate::embeddings::EmbeddingClient;
use crate::serialization::markdown_to_fact;
use crate::vector_store::{self, load_all_embeddings, save_embedding};
use anyhow::Result;
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

use super::Indexer;

impl Indexer {
    /// Indexes a single Fact in FTS5 index.
    pub fn index_fact(&self, fact: &Fact, file_path: Option<&Path>) -> Result<()> {
        let id_str = fact.id.to_string();
        let scope_str = fact.scope.to_string();
        let project_name = match &fact.scope {
            Scope::Project(p) => p.as_str(),
            Scope::Global => "",
        };
        let file_path_str = file_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let desc_str = fact.description.clone().unwrap_or_default();
        let status_str = fact.status.to_string();
        let supersedes_str = fact.supersedes.map(|s| s.to_string()).unwrap_or_default();
        let until_str = fact
            .validity
            .until
            .map(|u| u.to_rfc3339())
            .unwrap_or_default();
        let stale_str = fact
            .validity
            .stale_after
            .map(|s| s.to_rfc3339())
            .unwrap_or_default();

        self.conn
            .execute("DELETE FROM facts_fts WHERE id = ?1", params![id_str])?;
        self.conn
            .execute(
                "INSERT INTO facts_fts (id, scope, project_name, fact_type, title, description, body, tags, status, supersedes, file_path, since, until, stale_after)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    id_str,
                    scope_str,
                    project_name,
                    fact.fact_type.to_string(),
                    fact.title,
                    desc_str,
                    fact.body,
                    fact.tags.join(" "),
                    status_str,
                    supersedes_str,
                    file_path_str,
                    fact.validity.since.to_rfc3339(),
                    until_str,
                    stale_str
                ],
            )?;
        Ok(())
    }

    /// Generates and stores an embedding vector for a fact.
    pub fn vectorize_fact(&self, fact: &Fact, client: &EmbeddingClient) -> Result<()> {
        let content_to_embed = format!("{}: {}", fact.title, fact.body);
        let vector = client.embed_one(&content_to_embed)?;
        save_embedding(&self.conn, &fact.id, &client.model, &vector)
    }

    /// Removes a Fact and its vector embedding from the database.
    pub fn remove_fact(&self, id: &FactId) -> Result<()> {
        let id_str = id.to_string();
        self.conn
            .execute("DELETE FROM facts_fts WHERE id = ?1", params![id_str])?;
        vector_store::delete_embedding(&self.conn, id)?;
        Ok(())
    }

    /// Rebuilds the FTS5 index from markdown directories.
    /// Rebuilds the FTS5 index from markdown directories.
    ///
    /// Roots already present in the index are folded in, so reindexing from one
    /// project never silently drops other projects from the shared index. Roots
    /// that no longer exist on disk are skipped and therefore pruned.
    pub fn reindex_from_dirs<P: AsRef<Path>>(&self, dirs: &[P]) -> Result<usize> {
        let mut roots: Vec<PathBuf> = Vec::new();
        for dir in dirs {
            let p = dir.as_ref().to_path_buf();
            if !roots.contains(&p) {
                roots.push(p);
            }
        }

        // Recover store roots from existing rows: file_path is <root>/<type>/<id>.md.
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT DISTINCT file_path FROM facts_fts WHERE file_path != ''")
        {
            let recorded: Vec<String> = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .filter_map(|p| p.ok())
                .collect();
            for file in recorded {
                if let Some(root) = PathBuf::from(&file).parent().and_then(|t| t.parent()) {
                    let root = root.to_path_buf();
                    if !roots.contains(&root) {
                        roots.push(root);
                    }
                }
            }
        }

        self.conn.execute_batch("DELETE FROM facts_fts;")?;
        let mut count = 0;
        for dir_path in &roots {
            if !dir_path.exists() {
                continue;
            }
            for entry in WalkDir::new(dir_path).into_iter().flatten() {
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|e| e == "md")
                {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(fact) = markdown_to_fact(&content) {
                            self.index_fact(&fact, Some(entry.path()))?;
                            count += 1;
                        }
                    }
                }
            }
        }

        // Drop embeddings whose fact is no longer indexed (deleted files,
        // removed temp stores) so the vector table cannot accumulate orphans.
        let _ = self.prune_orphan_embeddings()?;
        Ok(count)
    }

    /// Deletes embeddings that no longer correspond to any indexed fact.
    pub fn prune_orphan_embeddings(&self) -> Result<usize> {
        let removed = self.conn.execute(
            "DELETE FROM fact_embeddings WHERE id NOT IN (SELECT id FROM facts_fts)",
            [],
        )?;
        Ok(removed)
    }

    /// Generates embeddings for every indexed fact that lacks a vector.
    pub fn vectorize_missing(&self, client: &EmbeddingClient) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT id, title, body FROM facts_fts")?;
        let mut rows = stmt.query([])?;
        let existing = load_all_embeddings(&self.conn)?;
        let mut missing = Vec::new();

        while let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let body: String = row.get(2)?;
            if let Ok(id) = FactId::from_str(&id_str) {
                if !existing.contains_key(&id) {
                    missing.push((id, format!("{}: {}", title, body)));
                }
            }
        }

        if missing.is_empty() {
            return Ok(0);
        }

        let texts: Vec<&str> = missing.iter().map(|(_, t)| t.as_str()).collect();
        let vectors = client.embed_batch(&texts)?;

        for ((id, _), vec) in missing.iter().zip(vectors) {
            save_embedding(&self.conn, id, &client.model, &vec)?;
        }

        Ok(missing.len())
    }

}
