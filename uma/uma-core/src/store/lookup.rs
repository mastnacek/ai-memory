//! Cross-scope lookups backed by the central index.
//!
//! Project memory is deliberately isolated: `list`, `read` and `search` without
//! an explicit scope only see the current project plus global. That isolation is
//! right — but it created two blind spots this module closes, both read-only:
//!
//! - **Discovery**: nothing listed the scopes that exist, so an agent in one
//!   project could not even name another project's scope to query it.
//! - **Unambiguous IDs**: a ULID identifies exactly one fact, yet `find_by_id`
//!   missed it when the fact lived in a *different* project.

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::domain::{FactId, Scope};
use crate::indexer::parse_scope_str;
use crate::serialization::markdown_to_fact;
use crate::store::Store;

/// One scope and how many indexed facts it holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeSummary {
    pub scope: Scope,
    pub facts: usize,
}

/// Resolves a foreign project's store root from indexed `file_path` values.
///
/// Walks an indexed fact path's ancestors to the directory named `.uma` — that
/// directory *is* the project's store. Returns `None` when the index holds no
/// files for the project (deleted or never-indexed). Takes a connection so it
/// is testable against a throwaway index.
pub fn store_root_from_index(conn: &Connection, project_name: &str) -> Option<std::path::PathBuf> {
    let mut stmt = conn
        .prepare("SELECT file_path FROM facts_fts WHERE project_name = ?1 LIMIT 1")
        .ok()?;
    let file_path: String = stmt
        .query_row([project_name], |row| row.get(0))
        .optional()
        .ok()?
        .filter(|p: &String| !p.is_empty())?;
    let path = std::path::PathBuf::from(file_path);
    path.ancestors()
        .find(|dir| dir.file_name().is_some_and(|n| n == ".uma"))
        .map(|dir| dir.to_path_buf())
}

/// Summarises every scope present in the index, largest first.
///
/// Takes a connection rather than opening the database itself, so it is
/// testable against a throwaway index and never depends on live memory.
pub fn scope_summaries_from(conn: &Connection) -> Result<Vec<ScopeSummary>> {
    let mut stmt = conn.prepare(
        "SELECT scope, COUNT(*) FROM facts_fts GROUP BY scope ORDER BY COUNT(*) DESC, scope",
    )?;
    let rows = stmt.query_map([], |row| {
        let scope_str: String = row.get(0)?;
        let facts: i64 = row.get(1)?;
        Ok((scope_str, facts))
    })?;

    let mut summaries = Vec::new();
    for row in rows {
        let (scope_str, facts) = row?;
        summaries.push(ScopeSummary {
            scope: parse_scope_str(&scope_str),
            facts: facts.max(0) as usize,
        });
    }
    Ok(summaries)
}

/// Resolves an ID to its fact file through the index, when the index knows it.
///
/// Returns `Ok(None)` when the ID is not indexed — the caller then reports a
/// plain miss. A row whose recorded file no longer exists is treated as a miss
/// too: the index is a cache, and a missing file means the fact is gone.
pub fn indexed_fact_from(conn: &Connection, id: &FactId) -> Result<Option<crate::domain::Fact>> {
    let path: Option<String> = conn
        .query_row(
            "SELECT file_path FROM facts_fts WHERE id = ?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| anyhow::anyhow!("index lookup failed for {id}: {err}"))?;

    // Not indexed at all: a plain miss, not an error.
    let Some(path) = path else {
        return Ok(None);
    };
    if path.is_empty() {
        return Ok(None);
    }
    let path = std::path::PathBuf::from(&path);
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    Ok(markdown_to_fact(&content).ok())
}

impl Store {
    /// Every scope the index knows about, largest first.
    ///
    /// This is the discovery primitive for cross-project memory: without it, an
    /// agent cannot even name another project's scope to query with `--scope`.
    pub fn known_scopes() -> Result<Vec<ScopeSummary>> {
        let indexer = Self::central_indexer()?;
        scope_summaries_from(indexer.connection())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Fact, FactType};
    use crate::indexer::Indexer;
    use tempfile::tempdir;

    fn fact(project: &str, title: &str) -> Fact {
        Fact::new(
            Scope::Project(project.to_string()),
            FactType::Decision,
            title.to_string(),
            "body".to_string(),
        )
    }

    #[test]
    fn test_store_root_resolves_through_indexed_file_paths() -> Result<()> {
        let dir = tempdir()?;
        let indexer = Indexer::open(dir.path().join("index.db"))?;

        // No record yet -> unresolvable.
        assert_eq!(
            store_root_from_index(indexer.connection(), "ghost"),
            None,
            "a project the index never saw has no store root"
        );

        // An indexed fact under a foreign project's .uma resolves to that .uma.
        let fact = fact("ProjektB", "one");
        indexer.index_fact(
            &fact,
            Some(std::path::Path::new(r"D:/proj/ProjektB/.uma/decision/x.md")),
        )?;
        let root = store_root_from_index(indexer.connection(), "ProjektB")
            .expect("indexed path must resolve");
        assert_eq!(root, std::path::PathBuf::from(r"D:/proj/ProjektB/.uma"));
        Ok(())
    }

    #[test]
    fn test_list_filters_by_fact_scope_not_directory() -> Result<()> {
        let dir = tempdir()?;
        let store = Store::new(dir.path().to_path_buf());

        let mut foreign = fact("mozek_rust", "imported decision");
        foreign.scope = Scope::Project("mozek_rust".to_string());
        store.write(&foreign)?;
        let local = fact("ai-memory", "local decision");
        store.write(&local)?;

        let listed = store.list(&Scope::Project("mozek_rust".to_string()), None)?;
        assert_eq!(listed.len(), 1, "only the foreign-scope fact matches");
        assert_eq!(listed[0].scope, Scope::Project("mozek_rust".to_string()));

        let local_listed = store.list(&Scope::Project("ai-memory".to_string()), None)?;
        assert_eq!(local_listed.len(), 1);
        Ok(())
    }

    #[test]
    fn test_scope_summaries_group_by_scope() -> Result<()> {
        let dir = tempdir()?;
        let indexer = Indexer::open(dir.path().join("index.db"))?;
        indexer.index_fact(&fact("alpha", "one"), None)?;
        indexer.index_fact(&fact("alpha", "two"), None)?;
        indexer.index_fact(
            &Fact::new(
                Scope::Global,
                FactType::Note,
                "g".to_string(),
                "body".to_string(),
            ),
            None,
        )?;

        let summaries = scope_summaries_from(indexer.connection())?;
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].facts, 2, "largest scope first");
        assert!(matches!(&summaries[0].scope, Scope::Project(p) if p == "alpha"));
        assert_eq!(summaries[1].facts, 1);
        assert_eq!(summaries[1].scope, Scope::Global);
        Ok(())
    }

    #[test]
    fn test_indexed_fact_roundtrip_by_id() -> Result<()> {
        let dir = tempdir()?;
        let indexer = Indexer::open(dir.path().join("index.db"))?;
        let fact = fact("alpha", "reachable by id");
        // The file must actually exist: the index is a cache pointing at the
        // Markdown source of truth, which is exactly what the lookup trusts.
        let fact_path = dir.path().join("x.md");
        std::fs::write(&fact_path, crate::serialization::fact_to_markdown(&fact)?)?;
        indexer.index_fact(&fact, Some(&fact_path))?;

        let found = indexed_fact_from(indexer.connection(), &fact.id)?;
        assert_eq!(found.expect("fact should resolve").id, fact.id);

        // An unknown ID is a clean miss, not an error.
        assert!(indexed_fact_from(indexer.connection(), &FactId::new())?.is_none());
        Ok(())
    }

    #[test]
    fn test_indexed_fact_is_a_miss_when_the_file_is_gone() -> Result<()> {
        let dir = tempdir()?;
        let indexer = Indexer::open(dir.path().join("index.db"))?;
        let fact = fact("alpha", "file later deleted");
        indexer.index_fact(&fact, Some(dir.path().join("gone.md").as_path()))?;

        // The index row points at a file that does not exist: a miss, because
        // the index is a cache and the Markdown file is the source of truth.
        assert!(indexed_fact_from(indexer.connection(), &fact.id)?.is_none());
        Ok(())
    }
}
