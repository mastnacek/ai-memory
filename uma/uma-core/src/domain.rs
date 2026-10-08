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
        Ok(match s.to_lowercase().as_str() {
            "decision" => FactType::Decision,
            "pattern" => FactType::Pattern,
            "reference" => FactType::Reference,
            "note" => FactType::Note,
            "task" => FactType::Task,
            other => FactType::Custom(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validity {
    pub since: DateTime<Utc>,
    pub until: Option<DateTime<Utc>>,
}

impl Default for Validity {
    fn default() -> Self {
        Self {
            since: Utc::now(),
            until: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub scope: Scope,
    pub fact_type: FactType,
    pub title: String,
    pub body: String,
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
            body,
            validity: Validity::default(),
            tags: Vec::new(),
            links: Vec::new(),
        }
    }
}
