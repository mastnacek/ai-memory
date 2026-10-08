//! Integration roundtrip: the paths a fact travels from creation to retrieval.
//!
//! Temp-directory stores are non-canonical by design, so they write Markdown
//! only and never touch the shared central index. Everything here therefore runs
//! against isolated, throwaway state and can never damage real memory.
//!
//! Note on `Store::supersede`: it resolves the real global/project roots itself
//! rather than using the store it is called on, so it is deliberately *not*
//! exercised here — a test would write to live memory. The deprecation contract
//! it depends on is covered by `deprecated_facts_drop_out_of_the_active_set`.

use tempfile::tempdir;
use uma_core::domain::{Fact, FactStatus, FactType, Scope};
use uma_core::indexer::Indexer;
use uma_core::search::{search_keyword, SearchOptions};
use uma_core::store::Store;

fn fact(title: &str, body: &str) -> Fact {
    Fact::new(
        Scope::Global,
        FactType::Decision,
        title.to_string(),
        body.to_string(),
    )
}

fn search_options(query: &str) -> SearchOptions<'_> {
    SearchOptions {
        query,
        scope: None,
        current_project: None,
        fact_type: None,
        include_deprecated: false,
        as_of: None,
        limit: 10,
    }
}

#[test]
fn write_read_list_roundtrip() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let store = Store::new(dir.path().to_path_buf());

    let first = fact("Use pnpm for dependency installs", "Install with pnpm.");
    let second = fact("Postgres handles connection pooling", "Pooled connections.");
    store.write(&first)?;
    store.write(&second)?;

    let read = store.read_by_id(&first.id)?;
    assert_eq!(read.title, first.title);
    assert_eq!(read.body, first.body);
    assert_eq!(read.status, FactStatus::Stable);

    let listed = store.list(&Scope::Global, Some(&FactType::Decision))?;
    assert_eq!(listed.len(), 2);

    Ok(())
}

#[test]
fn index_search_roundtrip_returns_only_the_matching_fact() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let indexer = Indexer::open(&dir.path().join("index.db"))?;

    let indexed = fact(
        "Adopt SQLite for FTS5 Indexing",
        "Embedded SQLite FTS5 gives fast BM25 keyword search.",
    );
    let other = fact("Rust Tooling", "Prefer standard cargo tools.");
    indexer.index_fact(&indexed, None)?;
    indexer.index_fact(&other, None)?;

    let hits = search_keyword(indexer.connection(), &search_options("FTS5"))?;

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, indexed.id);
    assert_eq!(hits[0].title, indexed.title);

    Ok(())
}

#[test]
fn deprecated_facts_drop_out_of_the_active_set() -> anyhow::Result<()> {
    // This is the contract `Store::supersede` relies on: once a fact is retired
    // it is no longer active, so every default read path hides it.
    let dir = tempdir()?;
    let store = Store::new(dir.path().to_path_buf());

    let mut original = fact("Use pnpm for dependency installs", "Install with pnpm.");
    store.write(&original)?;
    assert!(original.is_active_at(chrono::Utc::now()));

    original.status = FactStatus::Deprecated;
    original.validity.until = Some(chrono::Utc::now());
    store.write(&original)?;

    assert!(!original.is_active_at(chrono::Utc::now()));

    // Still on disk (deprecated, never deleted) — and still readable by id.
    let reread = store.read_by_id(&original.id)?;
    assert_eq!(reread.status, FactStatus::Deprecated);
    assert!(reread.validity.until.is_some());

    Ok(())
}
