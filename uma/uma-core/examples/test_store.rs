use directories::ProjectDirs;
use uma_core::{
    domain::{Fact, FactType, Scope},
    serialization::markdown_to_fact,
    store::Store,
};

fn main() -> anyhow::Result<()> {
    let store = Store::global()?;
    println!("Store created");

    let fact = Fact::new(
        Scope::Global,
        FactType::Note,
        "test".to_string(),
        "body".to_string(),
    );
    store.write(&fact)?;
    println!("Written fact: {}", fact.id);

    // Test parsing the new file
    let proj_dirs = ProjectDirs::from("com", "uma", "uma").unwrap();
    let store_root = proj_dirs.data_dir().join("global");
    let note_dir = store_root.join("note");

    for entry in std::fs::read_dir(&note_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = std::fs::read_to_string(&path)?;
        match markdown_to_fact(&content) {
            Ok(parsed) => println!("✓ Parsed: id={}, title={}", parsed.id, parsed.title),
            Err(e) => println!("✗ Parse error for {:?}: {:?}", path, e),
        }
    }

    let read_fact = store.read_by_id(&fact.id)?;
    println!("Read fact: {}", read_fact.title);
    Ok(())
}
