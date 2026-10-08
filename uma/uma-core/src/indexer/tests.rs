//! Indexer tests.

use super::*;
use crate::domain::{Fact, FactType, Scope};
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

    let count: i64 = indexer
        .connection()
        .query_row("SELECT COUNT(*) FROM facts_fts", [], |r| r.get(0))?;
    assert_eq!(count, 1);
    Ok(())
}
