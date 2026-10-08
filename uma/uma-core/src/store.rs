use crate::domain::{Fact, FactId, FactType, Scope};
use crate::serialization::{fact_to_markdown, markdown_to_fact};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn global() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "uma", "uma")
            .context("Could not determine project directories")?;
        let global_root = proj_dirs.data_dir().join("global");
        std::fs::create_dir_all(&global_root).context("Failed to create global store directory")?;
        Ok(Self::new(global_root))
    }

    pub fn project(project_name: String) -> Result<Self> {
        let root = Self::find_git_root()?.join(".uma").join(project_name);
        std::fs::create_dir_all(&root).context("Failed to create project store directory")?;
        Ok(Self::new(root))
    }

    pub fn find_git_root() -> Result<PathBuf> {
        let mut current = std::env::current_dir().context("Failed to get current directory")?;
        loop {
            if current.join(".git").exists() {
                return Ok(current);
            }
            if !current.pop() {
                anyhow::bail!("Not inside a git repository");
            }
        }
    }

    fn fact_path(&self, scope: &Scope, fact_type: &FactType, id: &FactId) -> PathBuf {
        let mut path = self.root.clone();
        if let Scope::Project(_) = scope {
            path = path.join(scope.dir_name());
        }
        path.join(fact_type.dir_name()).join(format!("{}.md", id))
    }

    pub fn write(&self, fact: &Fact) -> Result<()> {
        let path = self.fact_path(&fact.scope, &fact.fact_type, &fact.id);
        std::fs::create_dir_all(path.parent().unwrap())
            .context("Failed to create fact directory")?;
        let content = fact_to_markdown(fact).context("Failed to serialize fact")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write fact to {:?}", path))?;
        Ok(())
    }

    pub fn read(&self, scope: &Scope, fact_type: &FactType, id: &FactId) -> Result<Fact> {
        let path = self.fact_path(scope, fact_type, id);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read fact from {:?}", path))?;
        markdown_to_fact(&content).context("Failed to parse fact")
    }

    pub fn read_by_id(&self, id: &FactId) -> Result<Fact> {
        for entry in WalkDir::new(&self.root) {
            let entry = entry?;
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(fact) = markdown_to_fact(&content) {
                    if fact.id == *id {
                        return Ok(fact);
                    }
                }
            }
        }
        anyhow::bail!("Fact with id {} not found", id)
    }

    pub fn list(&self, scope: &Scope, fact_type: Option<&FactType>) -> Result<Vec<Fact>> {
        let mut facts = Vec::new();
        let mut search_root = self.root.clone();
        if let Scope::Project(_) = scope {
            search_root = search_root.join(scope.dir_name());
        }
        if let Some(ft) = fact_type {
            search_root = search_root.join(ft.dir_name());
        };

        if !search_root.exists() {
            return Ok(facts);
        }

        for entry in WalkDir::new(search_root) {
            let entry = entry?;
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(fact) = markdown_to_fact(&content) {
                    facts.push(fact);
                }
            }
        }

        facts.sort_by(|a, b| b.validity.since.cmp(&a.validity.since));
        Ok(facts)
    }

    pub fn delete(&self, scope: &Scope, fact_type: &FactType, id: &FactId) -> Result<()> {
        let path = self.fact_path(scope, fact_type, id);
        if path.exists() {
            std::fs::remove_file(path).context("Failed to delete fact file")?;
        }
        Ok(())
    }
}
