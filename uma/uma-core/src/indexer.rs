use crate::domain::{Fact, FactId, FactType, Scope};
use crate::embeddings::EmbeddingClient;
use crate::search::SearchHit;
use crate::serialization::markdown_to_fact;
use crate::vector_store::{self, init_vector_schema, load_all_embeddings, save_embedding};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

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
        let indexer = Self { conn, db_path };
        indexer.init_schema()?;
        Ok(indexer)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Initializes both FTS5 keyword and vector storage schemas.
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS facts_fts USING fts5(
                    id UNINDEXED, scope, project_name, fact_type, title, body, tags,
                    file_path UNINDEXED, since UNINDEXED, tokenize = 'porter unicode61'
                );",
            )
            .context("Failed to initialize centralized FTS5 schema")?;

        init_vector_schema(&self.conn)?;
        Ok(())
    }

    /// Indexes a single Fact in FTS5 index.
    pub fn index_fact(&self, fact: &Fact, file_path: Option<&Path>) -> Result<()> {
        let id_str = fact.id.to_string();
        let scope_str = fact.scope.to_string();
        let project_name = match &fact.scope {
            Scope::Project(p) => p.as_str(),
            Scope::Global => "",
        };
        let file_path_str = file_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        self.conn
            .execute("DELETE FROM facts_fts WHERE id = ?1", params![id_str])?;
        self.conn.execute(
            "INSERT INTO facts_fts (id, scope, project_name, fact_type, title, body, tags, file_path, since)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id_str,
                scope_str,
                project_name,
                fact.fact_type.to_string(),
                fact.title,
                fact.body,
                fact.tags.join(" "),
                file_path_str,
                fact.validity.since.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    /// Generates and stores an embedding vector for a fact.
    pub fn vectorize_fact(&self, fact: &Fact, client: &EmbeddingClient) -> Result<()> {
        let content_to_embed = format!("{}: {}", fact.title, fact.body);
        let vector = client.embed_one(&content_to_embed)?;
        save_embedding(&self.conn, &fact.id, &client.model, &vector)
    }

    /// Removes a Fact and its vector embedding from the database.
    pub fn remove_fact(&self, id: &FactId) -> Result<()> {
        let id_str = id.to_string();
        self.conn
            .execute("DELETE FROM facts_fts WHERE id = ?1", params![id_str])?;
        vector_store::delete_embedding(&self.conn, id)?;
        Ok(())
    }

    /// Rebuilds the FTS5 index from markdown directories.
    pub fn reindex_from_dirs<P: AsRef<Path>>(&self, dirs: &[P]) -> Result<usize> {
        self.conn.execute_batch("DELETE FROM facts_fts;")?;
        let mut count = 0;
        for dir in dirs {
            let dir_path = dir.as_ref();
            if !dir_path.exists() {
                continue;
            }
            for entry in WalkDir::new(dir_path).into_iter().flatten() {
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|e| e == "md")
                {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(fact) = markdown_to_fact(&content) {
                            self.index_fact(&fact, Some(entry.path()))?;
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    /// Scans all indexed facts and generates embeddings for any facts missing vectors.
    pub fn vectorize_missing(&self, client: &EmbeddingClient) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT id, title, body FROM facts_fts")?;
        let mut rows = stmt.query([])?;
        let existing = load_all_embeddings(&self.conn)?;
        let mut missing = Vec::new();

        while let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let body: String = row.get(2)?;
            if let Ok(id) = FactId::from_str(&id_str) {
                if !existing.contains_key(&id) {
                    missing.push((id, format!("{}: {}", title, body)));
                }
            }
        }

        if missing.is_empty() {
            return Ok(0);
        }

        let texts: Vec<&str> = missing.iter().map(|(_, t)| t.as_str()).collect();
        let vectors = client.embed_batch(&texts)?;

        for ((id, _), vec) in missing.iter().zip(vectors) {
            save_embedding(&self.conn, id, &client.model, &vec)?;
        }

        Ok(missing.len())
    }

    /// Performs BM25-ranked keyword search across all indexed facts.
    pub fn search_keyword(
        &self,
        query: &str,
        scope: Option<&Scope>,
        current_project: Option<&str>,
        fact_type: Option<&FactType>,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() {
            return Ok(Vec::new());
        }

        let fts_query = build_fts_query(trimmed_query);
        let mut sql = String::from(
            "SELECT id, scope, project_name, fact_type, title,
                    snippet(facts_fts, 5, '[match]', '[/match]', '...', 16) AS snippet,
                    tags,
                    file_path,
                    bm25(facts_fts, 0.0, 1.0, 1.0, 1.0, 10.0, 2.0, 5.0, 0.0, 0.0) AS rank
             FROM facts_fts WHERE facts_fts MATCH ?1",
        );

        match scope {
            Some(Scope::Global) => sql.push_str(" AND scope = 'global'"),
            Some(Scope::Project(_)) => sql.push_str(" AND project_name = ?"),
            None => {
                if current_project.is_some() {
                    sql.push_str(" AND (project_name = ? OR scope = 'global')");
                }
            }
        }

        if fact_type.is_some() {
            sql.push_str(" AND fact_type = ?");
        }

        sql.push_str(" ORDER BY rank ASC LIMIT ?");

        let execute_search = |fts_q: &str| -> Result<Vec<SearchHit>> {
            let mut params_vec: Vec<String> = vec![fts_q.to_string()];
            match scope {
                Some(Scope::Project(proj)) => params_vec.push(proj.clone()),
                None => {
                    if let Some(cp) = current_project {
                        params_vec.push(cp.to_string());
                    }
                }
                _ => {}
            }
            if let Some(ft) = fact_type {
                params_vec.push(ft.to_string());
            }

            let mut stmt = self.conn.prepare(&sql)?;
            let mut rows = stmt.query(rusqlite::params_from_iter(
                params_vec
                    .iter()
                    .map(|s| s as &dyn rusqlite::ToSql)
                    .chain(std::iter::once(&limit as &dyn rusqlite::ToSql)),
            ))?;

            let mut hits = Vec::new();
            while let Some(row) = rows.next()? {
                let id_str: String = row.get(0)?;
                let scope_str: String = row.get(1)?;
                let project_name: String = row.get(2)?;
                let type_str: String = row.get(3)?;
                let title: String = row.get(4)?;
                let snippet: String = row.get(5)?;
                let tags_str: String = row.get(6)?;
                let file_path_str: String = row.get(7)?;
                let rank: f64 = row.get(8)?;

                let file_path = if file_path_str.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(file_path_str))
                };

                hits.push(SearchHit {
                    id: FactId::from_str(&id_str)?,
                    scope: parse_scope_str(&scope_str),
                    project_name,
                    fact_type: FactType::from_str(&type_str).unwrap_or(FactType::Note),
                    title,
                    snippet,
                    tags: tags_str.split_whitespace().map(String::from).collect(),
                    file_path,
                    score: -rank,
                });
            }
            Ok(hits)
        };

        match execute_search(&fts_query) {
            Ok(res) => Ok(res),
            Err(_) => execute_search(&build_safe_fts_query(trimmed_query)),
        }
    }
}

fn build_fts_query(query: &str) -> String {
    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.is_empty() {
        return query.to_string();
    }
    tokens
        .iter()
        .map(|t| {
            let clean = t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if clean.is_empty() {
                format!("\"{}\"", t)
            } else {
                format!("{}*", clean)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_safe_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_scope_str(s: &str) -> Scope {
    if s == "global" {
        Scope::Global
    } else if let Some(name) = s.strip_prefix("project:") {
        Scope::Project(name.to_string())
    } else {
        Scope::Project(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_indexer_write_search() -> Result<()> {
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

        let hits = indexer.search_keyword("storage", None, Some("repo-a"), None, 10)?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].project_name, "repo-a");
        Ok(())
    }
}
