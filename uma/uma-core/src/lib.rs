pub mod domain;
pub mod serialization;
pub mod store;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Fact, FactId, FactType, Scope, Validity};
    use crate::serialization::{fact_to_markdown, markdown_to_fact};
    use crate::store::Store;
    use chrono::Utc;
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
}
