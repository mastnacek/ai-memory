use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

pub const DEFAULT_EMBEDDING_MODEL: &str = "qwen/qwen3-embedding-8b";

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: Vec<&'a str>,
}

#[derive(Deserialize)]
struct EmbeddingItem {
    embedding: Vec<f64>,
    #[serde(default)]
    index: usize,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    #[serde(default)]
    data: Vec<EmbeddingItem>,
    #[serde(default)]
    error: Option<OpenRouterError>,
}

#[derive(Deserialize)]
struct OpenRouterError {
    message: String,
}

pub struct EmbeddingClient {
    client: Client,
    api_key: String,
    pub model: String,
}

impl EmbeddingClient {
    /// Creates a new embedding client by resolving the OpenRouter API key automatically.
    pub fn new(model_override: Option<&str>) -> Result<Self> {
        let api_key = resolve_api_key().context(
            "OpenRouter API key not found in OPENROUTER_API_KEY or ~/.pi/agent/auth.json",
        )?;
        let model = model_override
            .unwrap_or(DEFAULT_EMBEDDING_MODEL)
            .to_string();

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(5)
            .build()
            .context("Failed to initialize HTTP client")?;

        Ok(Self {
            client,
            api_key,
            model,
        })
    }

    /// Embeds a batch of texts using OpenRouter's /v1/embeddings endpoint.
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f64>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let payload = EmbeddingRequest {
            model: &self.model,
            input: texts.to_vec(),
        };

        let mut retries = 0;
        let max_retries = 3;

        loop {
            let response = self
                .client
                .post("https://openrouter.ai/api/v1/embeddings")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://github.com/mastnacek/ai-memory")
                .header("X-Title", "UMA (Universal Memory Architecture)")
                .json(&payload)
                .send();

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let parsed: EmbeddingResponse = resp
                            .json()
                            .context("Failed to parse OpenRouter embeddings JSON response")?;

                        if let Some(err) = parsed.error {
                            anyhow::bail!("OpenRouter embeddings error: {}", err.message);
                        }

                        let mut result: Vec<Option<Vec<f64>>> = vec![None; texts.len()];
                        for item in parsed.data {
                            if item.index < result.len() {
                                result[item.index] = Some(item.embedding);
                            }
                        }

                        let mut vectors = Vec::with_capacity(texts.len());
                        for (idx, opt) in result.into_iter().enumerate() {
                            let vec = opt.with_context(|| {
                                format!("Missing embedding for input text at index {}", idx)
                            })?;
                            vectors.push(vec);
                        }

                        return Ok(vectors);
                    } else if (status.as_u16() == 429 || status.is_server_error())
                        && retries < max_retries
                    {
                        retries += 1;
                        let sleep_ms = 500 * (1 << retries);
                        std::thread::sleep(Duration::from_millis(sleep_ms));
                        continue;
                    } else {
                        let body = resp.text().unwrap_or_default();
                        anyhow::bail!("OpenRouter API error (HTTP {}): {}", status.as_u16(), body);
                    }
                }
                Err(e) if retries < max_retries => {
                    retries += 1;
                    let sleep_ms = 500 * (1 << retries);
                    std::thread::sleep(Duration::from_millis(sleep_ms));
                }
                Err(e) => {
                    return Err(e).context("Failed to send request to OpenRouter embeddings API");
                }
            }
        }
    }

    /// Embeds a single text.
    pub fn embed_one(&self, text: &str) -> Result<Vec<f64>> {
        let mut vecs = self.embed_batch(&[text])?;
        vecs.pop().context("No embedding returned")
    }
}

/// Computes cosine similarity between two float vectors. Returns a score in [0.0, 1.0].
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(0.0, 1.0)
}

/// Resolves the OpenRouter API key from environment or Pi configuration files.
pub fn resolve_api_key() -> Option<String> {
    // 1. Check environment variable
    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        let trimmed = key.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    let home = dirs_home()?;

    // 2. Check ~/.pi/agent/auth.json
    let auth_path = home.join(".pi").join("agent").join("auth.json");
    if auth_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&auth_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(key) = val["openrouter"]["key"].as_str() {
                    let trimmed = key.trim().to_string();
                    if !trimmed.is_empty() {
                        return Some(trimmed);
                    }
                }
                if let Some(key) = val["openrouter"]["access_token"].as_str() {
                    let trimmed = key.trim().to_string();
                    if !trimmed.is_empty() {
                        return Some(trimmed);
                    }
                }
            }
        }
    }

    // 3. Check ~/.pi/agent/openrouter-accounts.json
    let accounts_path = home
        .join(".pi")
        .join("agent")
        .join("openrouter-accounts.json");
    if accounts_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&accounts_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(accounts) = val["accounts"].as_array() {
                    for acc in accounts {
                        if let Some(key) = acc["apiKey"].as_str().or_else(|| acc["key"].as_str()) {
                            let trimmed = key.trim().to_string();
                            if !trimmed.is_empty() {
                                return Some(trimmed);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-6);

        let v3 = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&v1, &v3) - 0.0).abs() < 1e-6);

        let v4 = vec![0.7071, 0.7071, 0.0];
        assert!((cosine_similarity(&v1, &v4) - 0.7071).abs() < 1e-3);
    }
}
