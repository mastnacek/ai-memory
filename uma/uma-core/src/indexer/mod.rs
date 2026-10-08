use crate::vector_store::init_vector_schema;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

mod ops;
mod query;

#[cfg(test)]
mod tests;

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
        // Multiple clients (CLI, Pi agent, MCP server) share one index file:
        // WAL lets readers proceed during writes, and busy_timeout turns a
        // momentary lock into a short wait instead of an immediate
        // SQLITE_BUSY failure.
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")
            .with_context(|| format!("Failed to set concurrency pragmas on {:?}", db_path))?;
        let indexer = Self { conn, db_path };
        indexer.init_schema()?;
        Ok(indexer)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// How many facts the index currently holds.
    ///
    /// Distinguishes "never built" from "built but found nothing" — only the
    /// first is a reason to rebuild the cache.
    pub fn indexed_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM facts_fts", [], |row| row.get(0))
            .context("Failed to count indexed facts")?;
        Ok(count)
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
}

pub use query::{build_fts_query, build_safe_fts_query, parse_scope_str};
