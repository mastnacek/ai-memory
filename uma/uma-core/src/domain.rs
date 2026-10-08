use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub Ulid);

impl Default for FactId {
    fn default() -> Self {
        Self::new()
    }
}

impl FactId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    pub fn from_ulid(ulid: Ulid) -> Self {
        Self(ulid)
    }

    pub fn as_ulid(&self) -> &Ulid {
        &self.0
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for FactId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_string(s)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    Project(String),
    Global,
}

impl Scope {
    pub fn dir_name(&self) -> &str {
        match self {
            Scope::Project(name) => name,
            Scope::Global => "global",
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Project(name) => write!(f, "project:{}", name),
            Scope::Global => write!(f, "global"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactType {
    Decision,
    Preference,
    Fact,
    Skill,
    Correction,
    Pattern,
    Reference,
    Note,
    Task,
    Custom(String),
}

impl FactType {
    pub fn dir_name(&self) -> &str {
        match self {
            FactType::Decision => "decision",
            FactType::Preference => "preference",
            FactType::Fact => "fact",
            FactType::Skill => "skill",
            FactType::Correction => "correction",
            FactType::Pattern => "pattern",
            FactType::Reference => "reference",
            FactType::Note => "note",
            FactType::Task => "task",
            FactType::Custom(name) => name,
        }
    }
}

impl fmt::Display for FactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactType::Custom(name) => write!(f, "{}", name),
            _ => write!(f, "{}", self.dir_name()),
        }
    }
}

impl std::str::FromStr for FactType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_lowercase().as_str() {
            "decision" => FactType::Decision,
            "preference" => FactType::Preference,
            "fact" => FactType::Fact,
            "skill" => FactType::Skill,
            "correction" => FactType::Correction,
            "pattern" => FactType::Pattern,
            "reference" => FactType::Reference,
            "note" => FactType::Note,
            "task" => FactType::Task,
            other => FactType::Custom(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FactStatus {
    #[default]
    Stable,
    Deprecated,
    Draft,
}

impl fmt::Display for FactStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactStatus::Stable => write!(f, "stable"),
            FactStatus::Deprecated => write!(f, "deprecated"),
            FactStatus::Draft => write!(f, "draft"),
        }
    }
}

impl std::str::FromStr for FactStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "stable" => Ok(FactStatus::Stable),
            "deprecated" | "stale" => Ok(FactStatus::Deprecated),
            "draft" => Ok(FactStatus::Draft),
            other => anyhow::bail!("Unknown fact status: {}", other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorEvent {
    pub by: String,
    pub at: DateTime<Utc>,
}

impl ActorEvent {
    pub fn new(by: impl Into<String>) -> Self {
        Self {
            by: by.into(),
            at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validity {
    pub since: DateTime<Utc>,
    pub until: Option<DateTime<Utc>>,
    pub stale_after: Option<DateTime<Utc>>,
}

impl Default for Validity {
    fn default() -> Self {
        Self {
            since: Utc::now(),
            until: None,
            stale_after: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub scope: Scope,
    pub fact_type: FactType,
    pub title: String,
    pub description: Option<String>,
    /// Invocation template for `skill` facts. Placeholders are written `{{name}}`.
    /// It is data only — UMA expands it and prints the result, but never executes it.
    pub template: Option<String>,
    pub body: String,
    pub status: FactStatus,
    pub supersedes: Option<FactId>,
    pub generated: Option<ActorEvent>,
    pub verified: Vec<ActorEvent>,
    pub validity: Validity,
    pub tags: Vec<String>,
    pub links: Vec<FactId>,
}

impl Fact {
    pub fn new(scope: Scope, fact_type: FactType, title: String, body: String) -> Self {
        Self {
            id: FactId::new(),
            scope,
            fact_type,
            title,
            description: None,
            template: None,
            body,
            status: FactStatus::Stable,
            supersedes: None,
            generated: Some(ActorEvent::new("pi-agent/1.1")),
            verified: vec![ActorEvent::new("human:operator")],
            validity: Validity::default(),
            tags: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Checks whether the fact is active and non-deprecated at the specified timestamp.
    pub fn is_active_at(&self, at: DateTime<Utc>) -> bool {
        if self.validity.since > at {
            return false;
        }
        if let Some(until) = self.validity.until {
            if at >= until {
                return false;
            }
        }
        if let Some(stale) = self.validity.stale_after {
            if at >= stale {
                return false;
            }
        }
        self.status == FactStatus::Stable
    }
}
