//! The Jev transport: typed questions over OpenRouter.
//!
//! TypeSafe's native System One API (`api.typesafe.ai/v1/systemone`) takes
//! `{state, questions}` and returns typed answers with calibrated confidence.
//! That API needs a TypeSafe key, so this transport initially speaks the same
//! *contract* through OpenRouter's `jev-router` chat model with a JSON-schema
//! response — one enum answer per question, confidence reported conservatively
//! because the router does not expose calibrated probabilities. Swapping in
//! the native transport later means changing this file only.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;

use super::{Backend, Judgment, RecallNeed, Relationship};

/// The router model OpenRouter currently serves.
pub const DEFAULT_MODEL: &str = "typesafe/jev-router";

/// Chat-completion timeout: a triage call that outlives this is worse than no
/// triage at all — the caller degrades to offline.
const TIMEOUT_SECS: u64 = 10;

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

fn api_key() -> Result<String> {
    crate::embeddings::resolve_api_key()
        .filter(|key| !key.is_empty())
        .context("No OpenRouter API key found (env OPENROUTER_API_KEY or ~/.pi/agent/auth.json)")
}

/// One chat completion with a JSON-schema response format.
fn complete(schema_name: &str, schema: serde_json::Value, user_payload: String) -> Result<String> {
    let key = api_key()?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .context("Failed to build HTTP client")?;

    let body = json!({
        "model": DEFAULT_MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are a fast classifier. Answer ONLY with a JSON object \
                            matching the given schema. No prose, no markdown."
            },
            { "role": "user", "content": user_payload },
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": schema_name,
                "strict": true,
                "schema": schema,
            },
        },
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(&key)
        .json(&body)
        .send()
        .context("OpenRouter request failed")?;

    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().unwrap_or_default();
        anyhow::bail!("OpenRouter returned {status}: {}", truncate(&detail, 200));
    }

    let parsed: ChatResponse = response
        .json()
        .context("OpenRouter returned invalid JSON")?;
    parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .context("OpenRouter returned no choices")
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        text.to_string()
    } else {
        let head: String = text.chars().take(width).collect();
        format!("{head}…")
    }
}

/// Parses the model's JSON answer into a relationship.
fn parse_relationship(content: &str) -> Result<Relationship> {
    #[derive(Deserialize)]
    struct Answer {
        relationship: Relationship,
    }
    let answer: Answer = serde_json::from_str(content.trim())
        .with_context(|| format!("Unparsable classifier answer: {}", truncate(content, 120)))?;
    Ok(answer.relationship)
}

/// Parses the model's JSON answer into a recall verdict.
fn parse_recall(content: &str) -> Result<RecallNeed> {
    #[derive(Deserialize)]
    struct Answer {
        search: bool,
        #[serde(default)]
        fact_types: Vec<String>,
    }
    let answer: Answer = serde_json::from_str(content.trim())
        .with_context(|| format!("Unparsable classifier answer: {}", truncate(content, 120)))?;
    Ok(RecallNeed {
        search: answer.search,
        fact_types: answer.fact_types,
        notes: None,
    })
}

/// Classifies the relationship between two fact texts semantically.
pub fn relationship(text_a: &str, text_b: &str) -> Result<Judgment<Relationship>> {
    let content = complete(
        "relationship",
        json!({
            "type": "object",
            "properties": {
                "relationship": {
                    "type": "string",
                    "enum": ["duplicate", "contradiction", "unrelated"],
                }
            },
            "required": ["relationship"],
            "additionalProperties": false,
        }),
        format!(
            "Two memory facts that look similar. Classify their relationship.\n\
             - duplicate: the same rule stated twice\n\
             - contradiction: opposite rules about the same subject\n\
             - unrelated: different rules that merely share vocabulary\n\
             Texts may be in English or Czech.\n\nFact A: {text_a}\n\nFact B: {text_b}"
        ),
    )?;

    Ok(Judgment {
        answer: parse_relationship(&content)?,
        // The router exposes no calibrated confidence; a semantic answer at
        // 0.8 must never be confused with the offline verdict's numbers.
        confidence: 0.8,
        judged_by: Backend::Jev,
        notes: Some(DEFAULT_MODEL.to_string()),
    })
}

/// Decides whether a user message needs memory recall, semantically.
pub fn recall_need(message: &str) -> Result<Judgment<RecallNeed>> {
    let content = complete(
        "recall_need",
        json!({
            "type": "object",
            "properties": {
                "search": { "type": "boolean" },
                "fact_types": {
                    "type": "array",
                    "items": { "type": "string", "enum": [
                        "decision", "pattern", "preference", "skill", "fact", "note"
                    ] },
                },
            },
            "required": ["search", "fact_types"],
            "additionalProperties": false,
        }),
        format!(
            "A user message to a coding agent. Decide whether answering it may depend on \
             remembered project decisions, patterns, preferences, or skills — and if so, \
             which fact types are worth searching.\n\
             DEFAULT TO search=false. Only set true when the message clearly references \
             implementation specifics, past decisions, project conventions, or asks \
             'why/how' about the project itself. Casual chat, thanks, approvals \
             ('looks good', 'commit it'), simple commands, and questions about the current \
             file contents do NOT need memory.\n\nMessage: {message}"
        ),
    )?;

    Ok(Judgment {
        answer: parse_recall(&content)?,
        confidence: 0.8,
        judged_by: Backend::Jev,
        notes: Some(DEFAULT_MODEL.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relationship_answer() {
        assert_eq!(
            parse_relationship(r#"{"relationship": "contradiction"}"#).unwrap(),
            Relationship::Contradiction
        );
        assert!(parse_relationship("not json").is_err());
    }

    #[test]
    fn test_parse_recall_answer() {
        let need = parse_recall(r#"{"search": true, "fact_types": ["decision"]}"#).unwrap();
        assert!(need.search);
        assert_eq!(need.fact_types, vec!["decision"]);

        let need = parse_recall(r#"{"search": false, "fact_types": []}"#).unwrap();
        assert!(!need.search);
    }
}
