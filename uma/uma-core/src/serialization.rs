use crate::domain::{Fact, FactId, FactType, Scope, Validity};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use hashlink::LinkedHashMap;
use std::str::FromStr;
use yaml_rust2::{Yaml, YamlEmitter};

const FRONTMATTER_DELIMITER: &str = "---\n";

pub fn fact_to_markdown(fact: &Fact) -> Result<String> {
    let mut yaml = Yaml::Hash(LinkedHashMap::new());

    if let Yaml::Hash(ref mut map) = yaml {
        map.insert(
            Yaml::String("id".to_string()),
            Yaml::String(fact.id.to_string()),
        );
        map.insert(
            Yaml::String("scope".to_string()),
            Yaml::String(fact.scope.to_string()),
        );
        map.insert(
            Yaml::String("type".to_string()),
            Yaml::String(fact.fact_type.to_string()),
        );
        map.insert(
            Yaml::String("title".to_string()),
            Yaml::String(fact.title.clone()),
        );
        map.insert(
            Yaml::String("since".to_string()),
            Yaml::String(fact.validity.since.to_rfc3339()),
        );
        if let Some(until) = fact.validity.until {
            map.insert(
                Yaml::String("until".to_string()),
                Yaml::String(until.to_rfc3339()),
            );
        }
        if !fact.tags.is_empty() {
            map.insert(
                Yaml::String("tags".to_string()),
                Yaml::Array(fact.tags.iter().map(|t| Yaml::String(t.clone())).collect()),
            );
        }
        if !fact.links.is_empty() {
            map.insert(
                Yaml::String("links".to_string()),
                Yaml::Array(
                    fact.links
                        .iter()
                        .map(|l| Yaml::String(l.to_string()))
                        .collect(),
                ),
            );
        }
    }

    let mut frontmatter_str = String::new();
    let mut emitter = YamlEmitter::new(&mut frontmatter_str);
    emitter
        .dump(&yaml)
        .context("Failed to emit YAML frontmatter")?;

    // YamlEmitter may emit a leading '---' document start marker; strip it
    let frontmatter_str = frontmatter_str.trim_start_matches("---\n").trim();

    Ok(format!(
        "{}{}\n{}\n{}",
        FRONTMATTER_DELIMITER, frontmatter_str, FRONTMATTER_DELIMITER, fact.body
    ))
}

pub fn markdown_to_fact(content: &str) -> Result<Fact> {
    let (frontmatter_str, body) =
        split_frontmatter(content).context("Invalid frontmatter format")?;

    let yaml = yaml_rust2::YamlLoader::load_from_str(frontmatter_str)
        .context("Failed to parse YAML frontmatter")?
        .into_iter()
        .next()
        .context("Empty frontmatter")?;

    let id_str = yaml["id"].as_str().context("Missing or invalid id")?;
    let scope_str = yaml["scope"].as_str().context("Missing or invalid scope")?;
    let type_str = yaml["type"].as_str().context("Missing or invalid type")?;
    let title = yaml["title"]
        .as_str()
        .context("Missing or invalid title")?
        .to_string();
    let since_str = yaml["since"].as_str().context("Missing or invalid since")?;
    let until_str = yaml["until"].as_str();
    let tags = yaml["tags"]
        .as_vec()
        .map(|v| {
            v.iter()
                .filter_map(|y| y.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let links = yaml["links"]
        .as_vec()
        .map(|v| {
            v.iter()
                .filter_map(|y| y.as_str().and_then(|s| FactId::from_str(s).ok()))
                .collect()
        })
        .unwrap_or_default();

    let id = FactId::from_str(id_str).context("Invalid FactId")?;
    let scope = parse_scope(scope_str).context("Invalid scope")?;
    let fact_type = FactType::from_str(type_str).context("Invalid fact type")?;
    let since = DateTime::parse_from_rfc3339(since_str)
        .context("Invalid since timestamp")?
        .with_timezone(&Utc);
    let until = until_str
        .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)))
        .transpose()
        .context("Invalid until timestamp")?;

    Ok(Fact {
        id,
        scope,
        fact_type,
        title,
        body: body.to_string(),
        validity: Validity { since, until },
        tags,
        links,
    })
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    if !content.starts_with(FRONTMATTER_DELIMITER) {
        return None;
    }
    let after_first = &content[FRONTMATTER_DELIMITER.len()..];
    let end_pos = after_first.find(FRONTMATTER_DELIMITER)?;
    let frontmatter = &after_first[..end_pos];
    let body = &after_first[end_pos + FRONTMATTER_DELIMITER.len()..];
    Some((frontmatter, body))
}

fn parse_scope(s: &str) -> Result<Scope> {
    if s == "global" {
        Ok(Scope::Global)
    } else if let Some(name) = s.strip_prefix("project:") {
        Ok(Scope::Project(name.to_string()))
    } else {
        anyhow::bail!("Invalid scope format: {}", s)
    }
}
