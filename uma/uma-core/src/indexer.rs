use crate::domain::{Fact, FactId, Scope};
use crate::embeddings::EmbeddingClient;
use crate::serialization::markdown_to_fact;
use crate::vector_store::{self, init_vector_schema, load_all_embeddings, save_embedding};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

/// Current layout of the `facts_fts` FTS5 table. Bump when columns change.
///
/// Public so health checks can report a mismatch between what a given
/// `index.db` stores and what this build expects.
pub const FTS_SCHEMA_VERSION: i64 = 2;

pub struct Indexer {
    conn: Connection,
    #[allow(dead_code)]
    db_path: PathBuf,
}

impl Indexer {
    /// Opens or creates the centralized SQLite index database.
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open index database {:?}", db_path))?;
        let indexer = Self { conn, db_path };
        indexer.init_schema()?;
        Ok(indexer)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Initializes the FTS5 keyword and vector storage schemas.
    ///
    /// The FTS index is a rebuildable cache, so when the column layout changes
    /// the old table is dropped rather than failing on a stale database.
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS uma_meta (key TEXT PRIMARY KEY, value INTEGER NOT NULL)",
                [],
            )
            .context("Failed to initialize uma_meta table")?;

        let current: i64 = self
            .conn
            .query_row(
                "SELECT value FROM uma_meta WHERE key = 'fts_schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // Verify the *actual* column layout rather than trusting the recorded
        // version: an older build may have stamped a version without recreating
        // the table. `facts_fts` is a rebuildable cache, so a mismatch means drop
        // it and let the caller reindex.
        let has_expected_columns = {
            let mut stmt = self.conn.prepare("PRAGMA table_info(facts_fts)")?;
            let cols: Vec<String> = stmt
                .query_map([], |r| r.get::<_, String>(1))?
                .filter_map(|c| c.ok())
                .collect();
            !cols.is_empty() && cols.iter().any(|c| c == "description")
        };

        if !has_expected_columns || current != FTS_SCHEMA_VERSION {
            let _ = self.conn.execute_batch("DROP TABLE IF EXISTS facts_fts;");
        }

        self.conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS facts_fts USING fts5(
                    id UNINDEXED, scope, project_name, fact_type, title, description,
                    body, tags, status, supersedes UNINDEXED, file_path UNINDEXED,
                    since UNINDEXED, until UNINDEXED, stale_after UNINDEXED,
                    tokenize = 'porter unicode61'
                );",
            )
            .context("Failed to initialize centralized FTS5 schema")?;

        self.conn
            .execute(
                "INSERT INTO uma_meta (key, value) VALUES ('fts_schema_version', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![FTS_SCHEMA_VERSION],
            )
            .context("Failed to record FTS schema version")?;

        init_vector_schema(&self.conn)?;
        Ok(())
    }

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

/// Builds a prefix-match FTS5 query from free-form user input.
pub fn build_fts_query(query: &str) -> String {
    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.is_empty() {
        return query.to_string();
    }
    tokens
        .iter()
        .map(|t| {
            let clean = t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if clean.is_empty() {
                format!("\"{}\"", t)
            } else {
                format!("{}*", clean)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Builds a quoted FTS5 query safe against syntax errors from special characters.
pub fn build_safe_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_scope_str(s: &str) -> Scope {
    if s == "global" {
        Scope::Global
    } else if let Some(name) = s.strip_prefix("project:") {
        Scope::Project(name.to_string())
    } else {
        Scope::Project(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::FactType;
    use tempfile::tempdir;

    #[test]
    fn test_indexer_write_and_schema_version() -> Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("index.db");
        let indexer = Indexer::open(&db_path)?;

        let fact1 = Fact::new(
            Scope::Project("repo-a".to_string()),
            FactType::Decision,
            "Use PostgreSQL in Repo A".to_string(),
            "Repo A uses PostgreSQL for relational storage.".to_string(),
        );
        indexer.index_fact(&fact1, None)?;

        let count: i64 =
            indexer
                .connection()
                .query_row("SELECT COUNT(*) FROM facts_fts", [], |r| r.get(0))?;
        assert_eq!(count, 1);
        Ok(())
    }
}
