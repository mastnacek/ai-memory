use uma_core::domain::{Fact, FactStatus};

/// Returns the lifecycle badge suffix for a fact status (empty string when stable).
pub fn status_suffix(status: &FactStatus) -> &'static str {
    match status {
        FactStatus::Stable => "",
        FactStatus::Deprecated => " [DEPRECATED]",
        FactStatus::Draft => " [DRAFT]",
    }
}

/// Prints the complete representation of a Fact to stdout.
pub fn print_fact(fact: &Fact) {
    println!("ID:       {}", fact.id);
    println!("Scope:    {}", fact.scope);
    println!("Type:     {}", fact.fact_type);
    println!("Title:    {}", fact.title);
    println!("Status:   {}{}", fact.status, status_suffix(&fact.status));
    println!(
        "Since:    {}",
        fact.validity.since.format("%Y-%m-%d %H:%M:%S UTC")
    );
    if let Some(until) = fact.validity.until {
        println!("Until:    {}", until.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if !fact.tags.is_empty() {
        println!("Tags:     {}", fact.tags.join(", "));
    }
    if !fact.links.is_empty() {
        println!(
            "Links:    {}",
            fact.links
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();
    println!("{}", fact.body);
}

/// Prints a one-line summary of a Fact (for list views).
pub fn print_fact_summary(fact: &Fact) {
    println!(
        "{} [{}] {}{}",
        fact.id,
        fact.fact_type,
        fact.title,
        status_suffix(&fact.status)
    );
}