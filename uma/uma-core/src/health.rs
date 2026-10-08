//! Read-only index health inspection.
//!
//! Kept in the kernel rather than in a CLI slice so the SQL stays with the
//! schema it queries, and so consumers never need a database dependency of their
//! own. Everything here opens the index with `SQLITE_OPEN_READ_ONLY`: a health
//! check must never drop or rebuild the cache it is inspecting, which the normal
//! `Indexer::open` path deliberately does on a schema mismatch.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::indexer::FTS_SCHEMA_VERSION;

/// A point-in-time snapshot of the index cache, compared against what is on disk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct IndexHealth {
    /// Version recorded in `uma_meta` by whichever build last wrote the index.
    pub schema_version: i64,
    /// Version this build expects.
    pub expected_schema_version: i64,
    /// Rows in the full-text index.
    pub indexed_facts: usize,
    /// Stored embedding vectors.
    pub embeddings: usize,
    /// Vectors whose fact is no longer indexed.
    pub orphan_embeddings: usize,
    /// Indexed rows whose Markdown file no longer exists.
    pub stale_rows: usize,
}

impl IndexHealth {
    pub fn schema_is_current(&self) -> bool {
        self.schema_version == self.expected_schema_version
    }

    /// Facts that are indexed but have no vector yet.
    pub fn unvectorized(&self) -> usize {
        self.indexed_facts.saturating_sub(self.embeddings)
    }
}

/// Inspects an existing index file without modifying it.
pub fn inspect(db_path: &Path) -> Result<IndexHealth> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("Failed to open index read-only: {}", db_path.display()))?;

    Ok(IndexHealth {
        schema_version: schema_version(&conn),
        expected_schema_version: FTS_SCHEMA_VERSION,
        indexed_facts: scalar(&conn, "SELECT COUNT(*) FROM facts_fts"),
        embeddings: scalar(&conn, "SELECT COUNT(*) FROM fact_embeddings"),
        orphan_embeddings: scalar(
            &conn,
            "SELECT COUNT(*) FROM fact_embeddings WHERE id NOT IN (SELECT id FROM facts_fts)",
        ),
        stale_rows: count_stale_rows(&conn),
    })
}

fn schema_version(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT value FROM uma_meta WHERE key = 'fts_schema_version'",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

/// Counts index rows pointing at files that no longer exist.
///
/// A row with no recorded path is skipped: an older index may not have stored
/// one, and "unknown" is not the same as "stale".
pub fn count_stale_rows(conn: &Connection) -> usize {
    let Ok(mut stmt) = conn.prepare("SELECT file_path FROM facts_fts WHERE file_path != ''") else {
        return 0;
    };
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) else {
        return 0;
    };
    rows.filter_map(|row| row.ok())
        .filter(|path| !PathBuf::from(path).exists())
        .count()
}

/// Runs a single-value count query, degrading to 0 so a missing table in an older
/// index produces a readable report instead of aborting it.
fn scalar(conn: &Connection, sql: &str) -> usize {
    conn.query_row(sql, [], |row| row.get::<_, i64>(0))
        .map(|n| n.max(0) as usize)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Fact, FactType, Scope};
    use crate::indexer::Indexer;
    use tempfile::tempdir;

    fn fact(title: &str) -> Fact {
        Fact::new(
            Scope::Global,
            FactType::Decision,
            title.to_string(),
            "body".to_string(),
        )
    }

    #[test]
    fn test_inspect_reports_a_fresh_index() -> Result<()> {
        let dir = tempdir()?;
        let db = dir.path().join("index.db");
        let indexer = Indexer::open(&db)?;
        indexer.index_fact(&fact("one"), None)?;
        indexer.index_fact(&fact("two"), None)?;

        let health = inspect(&db)?;
        assert_eq!(health.indexed_facts, 2);
        assert_eq!(health.embeddings, 0);
        assert_eq!(health.unvectorized(), 2);
        assert_eq!(health.orphan_embeddings, 0);
        assert!(health.schema_is_current());
        Ok(())
    }

    #[test]
    fn test_rows_without_a_recorded_path_are_not_treated_as_stale() -> Result<()> {
        let dir = tempdir()?;
        let db = dir.path().join("index.db");
        let indexer = Indexer::open(&db)?;
        // No file path recorded at all.
        indexer.index_fact(&fact("no path"), None)?;

        let health = inspect(&db)?;
        assert_eq!(health.stale_rows, 0, "unknown path is not the same as stale");
        Ok(())
    }

    #[test]
    fn test_rows_pointing_at_deleted_files_are_reported_stale() -> Result<()> {
        let dir = tempdir()?;
        let db = dir.path().join("index.db");
        let indexer = Indexer::open(&db)?;
        let missing = dir.path().join("gone").join("x.md");
        indexer.index_fact(&fact("stale"), Some(&missing))?;

        let health = inspect(&db)?;
        assert_eq!(health.stale_rows, 1);
        Ok(())
    }
}
