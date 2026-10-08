use chrono::Utc;
use uma_core::domain::{Fact, FactStatus};

/// Returns the lifecycle badge suffix for a fact status (empty string when stable).
pub fn status_suffix(status: &FactStatus) -> &'static str {
    match status {
        FactStatus::Stable => "",
        FactStatus::Deprecated => " [DEPRECATED]",
        FactStatus::Draft => " [DRAFT]",
    }
}

/// Lifecycle badge for a fact: ` [STALE]` when a stable fact is past its
/// stale_after date, else the status badge.
///
/// Deliberately distinct from deprecation: a stale claim might still be true
/// but has passed the date after which it should no longer be trusted without
/// re-verification. Confusing the two would hide the difference between
/// "replaced by a better revision" and "possibly expired".
pub fn activity_suffix(fact: &Fact) -> &'static str {
    let is_stale = fact.status == FactStatus::Stable
        && fact
            .validity
            .stale_after
            .map(|deadline| deadline <= Utc::now())
            .unwrap_or(false);
    if is_stale {
        " [STALE]"
    } else {
        status_suffix(&fact.status)
    }
}

/// Renders the complete representation of a Fact.
///
/// Split from `print_fact` so non-CLI consumers (the MCP server) can return the
/// same text without printing it or duplicating the format.
pub fn render_fact(fact: &Fact) -> String {
    let mut out = String::new();

    out.push_str(&format!("ID:       {}\n", fact.id));
    out.push_str(&format!("Scope:    {}\n", fact.scope));
    out.push_str(&format!("Type:     {}\n", fact.fact_type));
    out.push_str(&format!("Title:    {}\n", fact.title));
    out.push_str(&format!(
        "Status:   {}{}\n",
        fact.status,
        activity_suffix(fact)
    ));
    out.push_str(&format!(
        "Since:    {}\n",
        fact.validity.since.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    if let Some(until) = fact.validity.until {
        out.push_str(&format!(
            "Until:    {}\n",
            until.format("%Y-%m-%d %H:%M:%S UTC")
        ));
    }
    if let Some(ref template) = fact.template {
        out.push_str(&format!("Template: {}\n", template));
    }
    if !fact.tags.is_empty() {
        out.push_str(&format!("Tags:     {}\n", fact.tags.join(", ")));
    }
    if !fact.links.is_empty() {
        let links: Vec<String> = fact.links.iter().map(|l| l.to_string()).collect();
        out.push_str(&format!("Links:    {}\n", links.join(", ")));
    }
    out.push('\n');
    out.push_str(&fact.body);
    out
}

/// Prints the complete representation of a Fact to stdout.
pub fn print_fact(fact: &Fact) {
    println!("{}", render_fact(fact));
}

/// Renders a one-line summary of a Fact (for list views).
pub fn render_fact_summary(fact: &Fact) -> String {
    format!(
        "{} [{}] {}{}",
        fact.id,
        fact.fact_type,
        fact.title,
        activity_suffix(fact)
    )
}

/// Prints a one-line summary of a Fact (for list views).
pub fn print_fact_summary(fact: &Fact) {
    println!("{}", render_fact_summary(fact));
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uma_core::domain::{FactType, Scope};

    fn stable_fact() -> Fact {
        Fact::new(
            Scope::Global,
            FactType::Note,
            "title".to_string(),
            "body".to_string(),
        )
    }

    #[test]
    fn test_activity_suffix_distinguishes_stale_from_deprecated() {
        let mut fact = stable_fact();
        assert_eq!(activity_suffix(&fact), "", "no deadline, no badge");

        fact.validity.stale_after = Some(Utc::now() + Duration::days(30));
        assert_eq!(activity_suffix(&fact), "", "future deadline is not stale");

        fact.validity.stale_after = Some(Utc::now() - Duration::days(1));
        assert_eq!(activity_suffix(&fact), " [STALE]", "past deadline is stale");

        fact.status = FactStatus::Deprecated;
        assert_eq!(
            activity_suffix(&fact),
            " [DEPRECATED]",
            "replaced and expired are different states and must read differently"
        );
    }
}
