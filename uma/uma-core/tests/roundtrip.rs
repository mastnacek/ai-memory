//! Integration roundtrip: the paths a fact travels from creation to retrieval.
//!
//! Temp-directory stores are non-canonical by design, so they write Markdown
//! only and never touch the shared central index. Everything here therefore runs
//! against isolated, throwaway state and can never damage real memory.
//!
//! Note on supersession: `Store::supersede` resolves the real global/project
//! roots, so it must never be called from a test — it would write to live memory.
//! `Store::supersede_within` is the single-store primitive used here instead; it
//! cannot reach outside the store it is called on.

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
fn supersede_roundtrip_retires_the_predecessor_and_chains_the_revision() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let store = Store::new(dir.path().to_path_buf());

    let original = fact(
        "Use pnpm for dependency installs",
        "Install dependencies with pnpm.",
    );
    store.write(&original)?;

    let mut revision = fact(
        "Use pnpm for dependency installation",
        "Dependencies are installed with pnpm.",
    );
    // As the supersede slice does: a revision restates the claim, so it
    // inherits the predecessor's origin unless explicitly overridden.
    revision.validity.since = original.validity.since;
    let stored = store.supersede_within(&original.id, revision)?;

    // The revision is active and chains back to what it replaced.
    assert_eq!(stored.supersedes, Some(original.id));
    assert_eq!(stored.status, FactStatus::Stable);

    // The claim's origin survives the supersession: a revision restates the
    // claim, so it has been true since the predecessor said it was. Only an
    // explicit caller override (imports) may move `since`.
    assert_eq!(stored.validity.since, original.validity.since);

    // The predecessor is deprecated with a closed validity window, never deleted.
    let retired = store.read_by_id(&original.id)?;
    assert_eq!(retired.status, FactStatus::Deprecated);
    assert!(retired.validity.until.is_some());
    assert_eq!(retired.title, original.title);

    // Default reads therefore show exactly one live decision.
    let now = chrono::Utc::now();
    let all = store.list(&Scope::Global, Some(&FactType::Decision))?;
    assert_eq!(all.len(), 2, "both revisions must remain on disk");
    let active: Vec<_> = all.iter().filter(|f| f.is_active_at(now)).collect();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, stored.id);

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
