use std::io;

use thiserror::Error;
use time::format_description::well_known::Iso8601;
use time::{Date, Month, OffsetDateTime, UtcOffset};

// Error Types
#[derive(Error, Debug)]
pub enum EtError {
    #[error("invalid epoch timestamp: {0}")]
    InvalidEpoch(String),

    #[error("invalid duration: {0}")]
    InvalidDuration(String),

    #[error("unsupported unit: {0}")]
    UnsupportedUnit(String),

    #[error("invalid ISO-8601 timestamp: {0}")]
    InvalidIso(String),

    #[error("missing timezone in timestamp: {0}")]
    MissingTimezone(String),

    #[error("arithmetic overflow")]
    Overflow,

    #[error("no input provided")]
    NoInput,

    #[error("{0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, EtError>;

/// Duration offset that can be applied to an epoch timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    /// Fixed duration in seconds (s, m, h, d, w)
    Seconds(i64),
    /// Calendar months
    Months(i32),
    /// Calendar years
    Years(i32),
}

impl Duration {
    /// Parse a duration string.
    ///
    /// Fixed units: s (seconds), m (minutes), h (hours), d (days), w (weeks)
    /// Calendar units: M (months), Y (years)
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(EtError::InvalidDuration("empty".to_string()));
        }

        // Determine sign and strip it
        let (sign, rest) = if let Some(stripped) = s.strip_prefix('+') {
            (1i64, stripped)
        } else if let Some(stripped) = s.strip_prefix('-') {
            (-1i64, stripped)
        } else {
            (1i64, s)
        };

        if rest.is_empty() {
            return Err(EtError::InvalidDuration(s.to_string()));
        }

        // Find where digits end and unit begins
        let digit_end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());

        if digit_end == 0 {
            return Err(EtError::InvalidDuration(s.to_string()));
        }

        let value_str = &rest[..digit_end];
        let unit = &rest[digit_end..];

        let value: i64 = value_str
            .parse()
            .map_err(|_| EtError::InvalidDuration(s.to_string()))?;

        // Calendar units (case-sensitive: M for months, Y for years)
        match unit {
            "M" | "mo" | "month" | "months" => {
                let months = i32::try_from(sign * value)
                    .map_err(|_| EtError::Overflow)?;
                return Ok(Duration::Months(months));
            }
            "Y" | "y" | "yr" | "year" | "years" => {
                let years = i32::try_from(sign * value)
                    .map_err(|_| EtError::Overflow)?;
                return Ok(Duration::Years(years));
            }
            _ => {}
        }

        // Fixed-duration units
        let multiplier: i64 = match unit.to_lowercase().as_str() {
            "s" | "" => 1,
            "m" => 60,
            "h" => 3600,
            "d" => 86400,
            "w" => 604800,
            other => {
                return Err(EtError::UnsupportedUnit(other.to_string()));
            }
        };

        let seconds = sign
            .checked_mul(value)
            .and_then(|v| v.checked_mul(multiplier))
            .ok_or(EtError::Overflow)?;

        Ok(Duration::Seconds(seconds))
    }

    /// Return the seconds value if this is a fixed duration.
    pub fn as_seconds(&self) -> Option<i64> {
        match self {
            Duration::Seconds(s) => Some(*s),
            _ => None,
        }
    }
}

/// Add months to a date, clamping day to valid range for the resulting month.
///
/// Examples:
/// - Jan 31 + 1 month → Feb 28 (or Feb 29 in leap year)
/// - Mar 31 + 1 month → Apr 30
/// - Dec 15 + 1 month → Jan 15 (next year)
fn add_months_to_date(date: Date, months: i32) -> Result<Date> {
    let year = date.year();
    let month = date.month() as i32; // 1-12
    let day = date.day();

    // Calculate total months from epoch and add offset
    let total_months = (year as i64) * 12 + (month as i64 - 1) + (months as i64);

    // Convert back to year and month
    let new_year = (total_months.div_euclid(12)) as i32;
    let new_month_idx = total_months.rem_euclid(12) as u8 + 1; // 1-12

    let new_month = Month::try_from(new_month_idx)
        .map_err(|_| EtError::Overflow)?;

    // Clamp day to valid range for the new month
    let max_day = new_month.length(new_year);
    let new_day = day.min(max_day);

    Date::from_calendar_date(new_year, new_month, new_day)
        .map_err(|_| EtError::Overflow)
}

/// Add years to a date, clamping day for leap year edge cases.
///
/// Examples:
/// - Feb 29 2024 + 1 year → Feb 28 2025
/// - Feb 28 2023 + 1 year → Feb 28 2024
fn add_years_to_date(date: Date, years: i32) -> Result<Date> {
    let new_year = date.year()
        .checked_add(years)
        .ok_or(EtError::Overflow)?;
    let month = date.month();
    let day = date.day();

    // Clamp day for Feb 29 in non-leap years
    let max_day = month.length(new_year);
    let new_day = day.min(max_day);

    Date::from_calendar_date(new_year, month, new_day)
        .map_err(|_| EtError::Overflow)
}

/// Get the current Unix epoch time in seconds.
pub fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// Apply a duration offset to an epoch timestamp.
pub fn apply_duration(epoch: i64, duration: Duration) -> Result<i64> {
    match duration {
        Duration::Seconds(secs) => {
            epoch.checked_add(secs).ok_or(EtError::Overflow)
        }
        Duration::Months(months) => {
            let dt = OffsetDateTime::from_unix_timestamp(epoch)
                .map_err(|_| EtError::InvalidEpoch(epoch.to_string()))?;

            let new_date = add_months_to_date(dt.date(), months)?;
            let new_dt = new_date
                .with_time(dt.time())
                .assume_offset(UtcOffset::UTC);

            Ok(new_dt.unix_timestamp())
        }
        Duration::Years(years) => {
            let dt = OffsetDateTime::from_unix_timestamp(epoch)
                .map_err(|_| EtError::InvalidEpoch(epoch.to_string()))?;

            let new_date = add_years_to_date(dt.date(), years)?;
            let new_dt = new_date
                .with_time(dt.time())
                .assume_offset(UtcOffset::UTC);

            Ok(new_dt.unix_timestamp())
        }
    }
}

/// Parse an epoch timestamp from a string.
pub fn parse_epoch(s: &str) -> Result<i64> {
    let s = s.trim();
    s.parse::<i64>()
        .map_err(|_| EtError::InvalidEpoch(s.to_string()))
}

/// Parse an ISO-8601 timestamp to Unix epoch seconds.
pub fn parse_iso(s: &str) -> Result<i64> {
    let s = s.trim();

    // Check for timezone indicator
    if !s.contains('Z')
        && !s.contains('+')
        && !s.chars().enumerate().any(|(i, c)| {
            c == '-' && i > 10
        })
    {
        let has_tz = if let Some(t_pos) = s.find('T') {
            let after_t = &s[t_pos..];
            after_t.contains('Z') || after_t.contains('+') || after_t[1..].contains('-')
        } else {
            s.contains('Z')
        };

        if !has_tz {
            return Err(EtError::MissingTimezone(s.to_string()));
        }
    }

    let dt = OffsetDateTime::parse(s, &Iso8601::PARSING)
        .map_err(|_| EtError::InvalidIso(s.to_string()))?;

    Ok(dt.unix_timestamp())
}

/// Format an epoch timestamp to ISO-8601 UTC.
pub fn format_iso(epoch: i64) -> Result<String> {
    let dt = OffsetDateTime::from_unix_timestamp(epoch)
        .map_err(|_| EtError::InvalidEpoch(epoch.to_string()))?;

    let format = time::format_description::parse(
        "[year]-[month padding:zero]-[day padding:zero]T[hour padding:zero]:[minute padding:zero]:[second padding:zero]Z",
    )
    .expect("valid format description");

    dt.format(&format)
        .map_err(|_| EtError::InvalidEpoch(epoch.to_string()))
}

/// Check if a string looks like a duration.
pub fn is_duration(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();
    if first == '+' || first == '-' {
        return true;
    }

    // Check if it's digits followed by a unit letter
    if first.is_ascii_digit() {
        let last = s.chars().last().unwrap();
        // Include M and Y for months/years
        matches!(
            last,
            's' | 'm' | 'h' | 'd' | 'w' | 'S' | 'H' | 'D' | 'W' | 'M' | 'Y' | 'y'
        )
    } else {
        false
    }
}
