//! Finding types for the health report.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Warn => "warn",
            Level::Fail => "FAIL",
        }
    }
}

/// One check result. A remedy is present whenever the level is not `Ok`, so a
/// warning never leaves the operator without a next step.
pub struct Finding {
    pub level: Level,
    pub label: String,
    pub detail: String,
    pub remedy: Option<&'static str>,
}

pub fn ok(label: impl Into<String>, detail: impl Into<String>) -> Finding {
    Finding {
        level: Level::Ok,
        label: label.into(),
        detail: detail.into(),
        remedy: None,
    }
}

pub fn warn(label: impl Into<String>, detail: impl Into<String>, remedy: &'static str) -> Finding {
    Finding {
        level: Level::Warn,
        label: label.into(),
        detail: detail.into(),
        remedy: Some(remedy),
    }
}

pub fn fail(label: impl Into<String>, detail: impl Into<String>, remedy: &'static str) -> Finding {
    Finding {
        level: Level::Fail,
        label: label.into(),
        detail: detail.into(),
        remedy: Some(remedy),
    }
}
