pub mod consolidate;
pub mod domain;
pub mod embeddings;
pub mod health;
pub mod indexer;
pub mod search;
pub mod secrets;
pub mod serialization;
pub mod sources;
pub mod similarity;
pub mod skill;
pub mod store;
pub mod timeline;
pub mod vector_store;

#[cfg(test)]
mod tests {
    use crate::domain::{Fact, FactType, Scope};
    use crate::serialization::{fact_to_markdown, markdown_to_fact};
    use crate::store::Store;
    use tempfile::tempdir;

    #[test]
    fn test_fact_serialization_roundtrip() {
        let fact = Fact::new(
            Scope::Global,
            FactType::Decision,
            "Test Decision".to_string(),
            "This is the body of the decision.".to_string(),
        );

        let markdown = fact_to_markdown(&fact).unwrap();
        let parsed = markdown_to_fact(&markdown).unwrap();

        assert_eq!(fact.id, parsed.id);
        assert_eq!(fact.scope, parsed.scope);
        assert_eq!(fact.fact_type, parsed.fact_type);
        assert_eq!(fact.title, parsed.title);
        assert_eq!(fact.body, parsed.body);
    }

    #[test]
    fn test_store_write_read() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());

        let fact = Fact::new(
            Scope::Global,
            FactType::Note,
            "Test Note".to_string(),
            "Note body content.".to_string(),
        );

        store.write(&fact).unwrap();
        let read_fact = store
            .read(&Scope::Global, &FactType::Note, &fact.id)
            .unwrap();

        assert_eq!(fact.id, read_fact.id);
        assert_eq!(fact.title, read_fact.title);
        assert_eq!(fact.body, read_fact.body);
    }

    #[test]
    fn test_store_read_by_id() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());

        let fact = Fact::new(
            Scope::Project("test-project".to_string()),
            FactType::Pattern,
            "Test Pattern".to_string(),
            "Pattern description.".to_string(),
        );

        store.write(&fact).unwrap();
        let read_fact = store.read_by_id(&fact.id).unwrap();

        assert_eq!(fact.id, read_fact.id);
        assert_eq!(fact.title, read_fact.title);
    }

    #[test]
    fn test_store_list() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());

        let fact1 = Fact::new(
            Scope::Global,
            FactType::Note,
            "Note 1".to_string(),
            "Body 1".to_string(),
        );
        let fact2 = Fact::new(
            Scope::Global,
            FactType::Note,
            "Note 2".to_string(),
            "Body 2".to_string(),
        );

        store.write(&fact1).unwrap();
        store.write(&fact2).unwrap();

        let facts = store.list(&Scope::Global, Some(&FactType::Note)).unwrap();
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(Scope::Global.to_string(), "global");
        assert_eq!(
            Scope::Project("my-proj".to_string()).to_string(),
            "project:my-proj"
        );
    }

    #[test]
    fn test_fact_type_display() {
        assert_eq!(FactType::Decision.to_string(), "decision");
        assert_eq!(
            FactType::Custom("custom-type".to_string()).to_string(),
            "custom-type"
        );
    }

    #[test]
    fn test_store_search() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("index.db");
        let indexer = crate::indexer::Indexer::open(&db_path)?;

        let fact1 = Fact::new(
            Scope::Global,
            FactType::Decision,
            "Adopt SQLite for FTS5 Indexing".to_string(),
            "We use embedded SQLite FTS5 for fast BM25 keyword search.".to_string(),
        );
        let fact2 = Fact::new(
            Scope::Global,
            FactType::Preference,
            "Rust Tooling".to_string(),
            "Prefer standard cargo tools.".to_string(),
        );

        indexer.index_fact(&fact1, None)?;
        indexer.index_fact(&fact2, None)?;

        let hits = crate::search::search_keyword(
            indexer.connection(),
            &crate::search::SearchOptions {
                query: "FTS5",
                scope: None,
                current_project: None,
                fact_type: None,
                include_deprecated: false,
                as_of: None,
                limit: 10,
            },
        )?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, fact1.id);
        assert_eq!(hits[0].title, "Adopt SQLite for FTS5 Indexing");
        Ok(())
    }

    #[test]
    fn test_non_canonical_store_does_not_pollute_central_index() -> anyhow::Result<()> {
        // A store created from an arbitrary path (tests, temp dirs) must never
        // write into the shared central index, or it leaves orphan rows behind.
        let dir = tempdir()?;
        let store = Store::new(dir.path().to_path_buf());
        assert!(!store.is_canonical());

        let fact = Fact::new(
            Scope::Global,
            FactType::Note,
            "Temp Note".to_string(),
            "Must stay out of the central index.".to_string(),
        );
        store.write(&fact)?;

        // The markdown landed on disk...
        assert!(store.read_by_id(&fact.id).is_ok());

        // ...but no row was written to the central index.
        let indexer = Store::central_indexer()?;
        let count: i64 = indexer.connection().query_row(
            "SELECT COUNT(*) FROM facts_fts WHERE id = ?1",
            [fact.id.to_string()],
            |r| r.get(0),
        )?;
        assert_eq!(count, 0, "temp store polluted the central index");
        Ok(())
    }
}
