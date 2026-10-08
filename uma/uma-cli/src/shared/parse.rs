//! User-facing timestamp parsing for the CLI.
//!
//! A user typing a deadline should not have to spell out RFC 3339; a bare date
//! is the common case and is understood as end of day UTC.

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

/// Parses an RFC 3339 timestamp (`2026-12-31T23:59:59Z`), or a bare date
/// (`2026-12-31`) understood as 23:59:59 UTC on that day.
///
/// An explicit failure is better than a silent default: writing a fact with the
/// wrong expiry because the format was silently guessed is worse than refusing
/// to write it.
pub fn parse_datetime_or_date(input: &str) -> Result<DateTime<Utc>> {
    let trimmed = input.trim();

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let Some(end_of_day) = date.and_hms_opt(23, 59, 59) else {
            bail!("invalid date '{trimmed}'");
        };
        return Ok(Utc.from_utc_datetime(&end_of_day));
    }

    Err(anyhow::anyhow!(
        "Invalid timestamp '{trimmed}'. Use RFC 3339 (2026-12-31T23:59:59Z) or a date (2026-12-31)."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc3339_is_parsed_exactly() {
        let dt = parse_datetime_or_date("2026-12-31T12:00:00Z").unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-12-31T12:00:00+00:00");
    }

    #[test]
    fn test_bare_date_means_end_of_day_utc() {
        let dt = parse_datetime_or_date("2026-12-31").unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-12-31T23:59:59+00:00");
    }

    #[test]
    fn test_offsets_are_normalised_to_utc() {
        let dt = parse_datetime_or_date("2026-12-31T14:00:00+02:00").unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-12-31T12:00:00+00:00");
    }

    #[test]
    fn test_garbage_is_rejected_loudly() {
        let err = parse_datetime_or_date("next tuesday").unwrap_err();
        assert!(err.to_string().contains("Invalid timestamp"));
    }
}
