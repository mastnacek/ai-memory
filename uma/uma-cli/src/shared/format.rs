use uma_core::domain::{Fact, FactStatus};

/// Returns the lifecycle badge suffix for a fact status (empty string when stable).
pub fn status_suffix(status: &FactStatus) -> &'static str {
    match status {
        FactStatus::Stable => "",
        FactStatus::Deprecated => " [DEPRECATED]",
        FactStatus::Draft => " [DRAFT]",
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
        status_suffix(&fact.status)
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
        status_suffix(&fact.status)
    )
}

/// Prints a one-line summary of a Fact (for list views).
pub fn print_fact_summary(fact: &Fact) {
    println!("{}", render_fact_summary(fact));
}
