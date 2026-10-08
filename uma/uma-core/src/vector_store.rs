use crate::domain::FactId;
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::str::FromStr;

/// Initializes the vector storage table in SQLite.
pub fn init_vector_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS fact_embeddings (
            id TEXT PRIMARY KEY,
            model TEXT NOT NULL,
            dimensions INTEGER NOT NULL,
            vector BLOB NOT NULL,
            updated_at TEXT NOT NULL
        );",
    )
    .context("Failed to initialize fact_embeddings table")?;
    Ok(())
}

/// Serializes a float vector into little-endian byte array BLOB.
pub fn vector_to_bytes(v: &[f64]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 8);
    for &val in v {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

/// Deserializes a byte array BLOB into a float vector.
pub fn bytes_to_vector(bytes: &[u8]) -> Vec<f64> {
    let count = bytes.len() / 8;
    let mut v = Vec::with_capacity(count);
    for i in 0..count {
        if let Ok(slice) = bytes[i * 8..(i + 1) * 8].try_into() {
            v.push(f64::from_le_bytes(slice));
        }
    }
    v
}

/// Stores or updates an embedding for a specific FactId.
pub fn save_embedding(conn: &Connection, id: &FactId, model: &str, vector: &[f64]) -> Result<()> {
    let id_str = id.to_string();
    let dims = vector.len() as i64;
    let bytes = vector_to_bytes(vector);
    let now_str = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO fact_embeddings (id, model, dimensions, vector, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            model = excluded.model,
            dimensions = excluded.dimensions,
            vector = excluded.vector,
            updated_at = excluded.updated_at",
        params![id_str, model, dims, bytes, now_str],
    )
    .context("Failed to insert/update embedding in database")?;

    Ok(())
}

/// Loads an embedding for a specific FactId if it exists.
pub fn load_embedding(conn: &Connection, id: &FactId) -> Result<Option<Vec<f64>>> {
    let id_str = id.to_string();
    let mut stmt = conn
        .prepare("SELECT vector FROM fact_embeddings WHERE id = ?1")
        .context("Failed to prepare load_embedding query")?;

    let mut rows = stmt.query(params![id_str])?;
    if let Some(row) = rows.next()? {
        let bytes: Vec<u8> = row.get(0)?;
        Ok(Some(bytes_to_vector(&bytes)))
    } else {
        Ok(None)
    }
}

/// Loads all stored embeddings into a memory map keyed by FactId.
pub fn load_all_embeddings(conn: &Connection) -> Result<HashMap<FactId, Vec<f64>>> {
    let mut stmt = conn
        .prepare("SELECT id, vector FROM fact_embeddings")
        .context("Failed to prepare load_all_embeddings query")?;

    let mut rows = stmt.query([])?;
    let mut map = HashMap::new();

    while let Some(row) = rows.next()? {
        let id_str: String = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        if let Ok(id) = FactId::from_str(&id_str) {
            map.insert(id, bytes_to_vector(&bytes));
        }
    }

    Ok(map)
}

/// Removes an embedding for a specific FactId.
pub fn delete_embedding(conn: &Connection, id: &FactId) -> Result<()> {
    let id_str = id.to_string();
    conn.execute("DELETE FROM fact_embeddings WHERE id = ?1", params![id_str])
        .context("Failed to delete embedding from database")?;
    Ok(())
}

/// Returns the total count of stored embeddings.
pub fn count_embeddings(conn: &Connection) -> Result<usize> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM fact_embeddings", [], |r| r.get(0))
        .unwrap_or(0);
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_bytes_roundtrip() {
        let original = vec![0.12345, -0.98765, 0.0, 100.25];
        let bytes = vector_to_bytes(&original);
        let restored = bytes_to_vector(&bytes);
        assert_eq!(original.len(), restored.len());
        for (a, b) in original.iter().zip(&restored) {
            assert!((a - b).abs() < 1e-9);
        }
    }

    #[test]
    fn test_vector_store_crud() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        init_vector_schema(&conn)?;

        let id = FactId::new();
        let vec_data = vec![0.1, 0.2, 0.3];
        save_embedding(&conn, &id, "test-model", &vec_data)?;

        let loaded = load_embedding(&conn, &id)?;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), vec_data);

        let all = load_all_embeddings(&conn)?;
        assert_eq!(all.len(), 1);
        assert!(all.contains_key(&id));

        delete_embedding(&conn, &id)?;
        let after_delete = load_embedding(&conn, &id)?;
        assert!(after_delete.is_none());

        Ok(())
    }
}
