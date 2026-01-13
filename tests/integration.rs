use et::{apply_duration, format_iso, is_duration, parse_epoch, parse_iso, Duration, EtError};

// Duration Parsing - Fixed Units
#[test]
fn duration_seconds() {
    assert_eq!(Duration::parse("30s").unwrap(), Duration::Seconds(30));
    assert_eq!(Duration::parse("+30s").unwrap(), Duration::Seconds(30));
    assert_eq!(Duration::parse("-30s").unwrap(), Duration::Seconds(-30));
}

#[test]
fn duration_minutes() {
    assert_eq!(Duration::parse("5m").unwrap(), Duration::Seconds(300));
    assert_eq!(Duration::parse("-5m").unwrap(), Duration::Seconds(-300));
}

#[test]
fn duration_hours() {
    assert_eq!(Duration::parse("2h").unwrap(), Duration::Seconds(7200));
    assert_eq!(Duration::parse("+2h").unwrap(), Duration::Seconds(7200));
}

#[test]
fn duration_days() {
    assert_eq!(Duration::parse("7d").unwrap(), Duration::Seconds(604800));
    assert_eq!(Duration::parse("-1d").unwrap(), Duration::Seconds(-86400));
}

#[test]
fn duration_weeks() {
    assert_eq!(Duration::parse("2w").unwrap(), Duration::Seconds(1209600));
}

#[test]
fn duration_bare_number_as_seconds() {
    assert_eq!(Duration::parse("3600").unwrap(), Duration::Seconds(3600));
}

#[test]
fn duration_fixed_case_insensitive() {
    assert_eq!(Duration::parse("1H").unwrap(), Duration::parse("1h").unwrap());
    assert_eq!(Duration::parse("1D").unwrap(), Duration::parse("1d").unwrap());
    assert_eq!(Duration::parse("1W").unwrap(), Duration::parse("1w").unwrap());
}

// Duration Parsing - Calendar Units (Months and Years)
#[test]
fn duration_months_uppercase_m() {
    assert_eq!(Duration::parse("1M").unwrap(), Duration::Months(1));
    assert_eq!(Duration::parse("+3M").unwrap(), Duration::Months(3));
    assert_eq!(Duration::parse("-6M").unwrap(), Duration::Months(-6));
    assert_eq!(Duration::parse("12M").unwrap(), Duration::Months(12));
}

#[test]
fn duration_months_word_forms() {
    assert_eq!(Duration::parse("1mo").unwrap(), Duration::Months(1));
    assert_eq!(Duration::parse("2month").unwrap(), Duration::Months(2));
    assert_eq!(Duration::parse("3months").unwrap(), Duration::Months(3));
}

#[test]
fn duration_years() {
    assert_eq!(Duration::parse("1Y").unwrap(), Duration::Years(1));
    assert_eq!(Duration::parse("+2Y").unwrap(), Duration::Years(2));
    assert_eq!(Duration::parse("-1Y").unwrap(), Duration::Years(-1));
    assert_eq!(Duration::parse("1y").unwrap(), Duration::Years(1));
    assert_eq!(Duration::parse("1yr").unwrap(), Duration::Years(1));
    assert_eq!(Duration::parse("2year").unwrap(), Duration::Years(2));
    assert_eq!(Duration::parse("3years").unwrap(), Duration::Years(3));
}

#[test]
fn duration_large_calendar_values() {
    assert_eq!(Duration::parse("100M").unwrap(), Duration::Months(100));
    assert_eq!(Duration::parse("50Y").unwrap(), Duration::Years(50));
}

// Duration Parsing - Invalid Input
#[test]
fn duration_invalid_formats() {
    assert!(Duration::parse("").is_err());
    assert!(Duration::parse("abc").is_err());
    assert!(Duration::parse("++5s").is_err());
    assert!(Duration::parse("--5s").is_err());
    assert!(Duration::parse("+").is_err());
    assert!(Duration::parse("-").is_err());
}

#[test]
fn duration_unknown_unit() {
    assert!(matches!(Duration::parse("5x").unwrap_err(), EtError::UnsupportedUnit(_)));
    assert!(matches!(Duration::parse("10foo").unwrap_err(), EtError::UnsupportedUnit(_)));
}

// Epoch Parsing
#[test]
fn parse_epoch_valid() {
    assert_eq!(parse_epoch("1704912345").unwrap(), 1704912345);
    assert_eq!(parse_epoch("0").unwrap(), 0);
    assert_eq!(parse_epoch("-1000").unwrap(), -1000);
    assert_eq!(parse_epoch("  1704912345  ").unwrap(), 1704912345);
}

#[test]
fn parse_epoch_invalid() {
    assert!(parse_epoch("abc").is_err());
    assert!(parse_epoch("12.34").is_err());
    assert!(parse_epoch("").is_err());
}

// ISO Parsing
#[test]
fn parse_iso_utc() {
    assert_eq!(parse_iso("2024-01-10T12:00:00Z").unwrap(), 1704888000);
}

#[test]
fn parse_iso_with_offset() {
    let z = parse_iso("2024-01-10T12:00:00Z").unwrap();
    let zero = parse_iso("2024-01-10T12:00:00+00:00").unwrap();
    assert_eq!(z, zero);

    // +02:00 means 2 hours ahead, so UTC is 2 hours earlier
    let plus2 = parse_iso("2024-01-10T12:00:00+02:00").unwrap();
    let expected = parse_iso("2024-01-10T10:00:00Z").unwrap();
    assert_eq!(plus2, expected);
}

#[test]
fn parse_iso_missing_timezone() {
    assert!(matches!(
        parse_iso("2024-01-10T12:00:00").unwrap_err(),
        EtError::MissingTimezone(_)
    ));
}

#[test]
fn parse_iso_invalid() {
    assert!(parse_iso("not-a-date").is_err());
    assert!(parse_iso("2024-13-01T00:00:00Z").is_err()); // invalid month
    assert!(parse_iso("2024-01-32T00:00:00Z").is_err()); // invalid day
}

// Format
#[test]
fn format_epoch() {
    assert_eq!(format_iso(0).unwrap(), "1970-01-01T00:00:00Z");
    assert_eq!(format_iso(1704888000).unwrap(), "2024-01-10T12:00:00Z");
    assert_eq!(format_iso(-86400).unwrap(), "1969-12-31T00:00:00Z");
}

// Fixed Duration Arithmetic
#[test]
fn apply_seconds() {
    let base = 1704912345; // 2024-01-10T18:45:45Z
    assert_eq!(apply_duration(base, Duration::Seconds(3600)).unwrap(), base + 3600);
    assert_eq!(apply_duration(base, Duration::Seconds(-86400)).unwrap(), base - 86400);
    assert_eq!(apply_duration(base, Duration::Seconds(0)).unwrap(), base);
}

#[test]
fn apply_seconds_overflow() {
    assert!(apply_duration(i64::MAX, Duration::Seconds(1)).is_err());
    assert!(apply_duration(i64::MIN, Duration::Seconds(-1)).is_err());
}

// Month Arithmetic - Basic
/// Helper: parse ISO, apply duration, format back to ISO for readable assertions
fn apply_and_format(iso: &str, duration: Duration) -> String {
    let epoch = parse_iso(iso).unwrap();
    let result = apply_duration(epoch, duration).unwrap();
    format_iso(result).unwrap()
}

#[test]
fn add_one_month_basic() {
    // Jan 15 + 1M = Feb 15
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(1)), "2024-02-15T12:00:00Z");
    // Feb 15 + 1M = Mar 15
    assert_eq!(apply_and_format("2024-02-15T12:00:00Z", Duration::Months(1)), "2024-03-15T12:00:00Z");
    // Nov 15 + 1M = Dec 15
    assert_eq!(apply_and_format("2024-11-15T12:00:00Z", Duration::Months(1)), "2024-12-15T12:00:00Z");
}

#[test]
fn add_month_year_rollover() {
    // Dec 15 + 1M = Jan 15 next year
    assert_eq!(apply_and_format("2024-12-15T12:00:00Z", Duration::Months(1)), "2025-01-15T12:00:00Z");
    // Dec 31 + 1M = Jan 31 next year
    assert_eq!(apply_and_format("2024-12-31T12:00:00Z", Duration::Months(1)), "2025-01-31T12:00:00Z");
}

#[test]
fn subtract_month_basic() {
    // Mar 15 - 1M = Feb 15
    assert_eq!(apply_and_format("2024-03-15T12:00:00Z", Duration::Months(-1)), "2024-02-15T12:00:00Z");
    // Feb 15 - 1M = Jan 15
    assert_eq!(apply_and_format("2024-02-15T12:00:00Z", Duration::Months(-1)), "2024-01-15T12:00:00Z");
}

#[test]
fn subtract_month_year_rollover() {
    // Jan 15 - 1M = Dec 15 previous year
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(-1)), "2023-12-15T12:00:00Z");
}

// Month Arithmetic - Day Clamping (Edge Cases)
#[test]
fn add_month_clamp_jan31_to_feb28_nonleap() {
    // Jan 31 2023 + 1M = Feb 28 2023 (non-leap year)
    assert_eq!(apply_and_format("2023-01-31T12:00:00Z", Duration::Months(1)), "2023-02-28T12:00:00Z");
}

#[test]
fn add_month_clamp_jan31_to_feb29_leap() {
    // Jan 31 2024 + 1M = Feb 29 2024 (leap year)
    assert_eq!(apply_and_format("2024-01-31T12:00:00Z", Duration::Months(1)), "2024-02-29T12:00:00Z");
}

#[test]
fn add_month_clamp_jan30_to_feb28_nonleap() {
    // Jan 30 2023 + 1M = Feb 28 2023 (clamped)
    assert_eq!(apply_and_format("2023-01-30T12:00:00Z", Duration::Months(1)), "2023-02-28T12:00:00Z");
}

#[test]
fn add_month_clamp_jan29_to_feb28_nonleap() {
    // Jan 29 2023 + 1M = Feb 28 2023 (clamped)
    assert_eq!(apply_and_format("2023-01-29T12:00:00Z", Duration::Months(1)), "2023-02-28T12:00:00Z");
}

#[test]
fn add_month_clamp_mar31_to_apr30() {
    // Mar 31 + 1M = Apr 30 (April has 30 days)
    assert_eq!(apply_and_format("2024-03-31T12:00:00Z", Duration::Months(1)), "2024-04-30T12:00:00Z");
}

#[test]
fn add_month_clamp_may31_to_jun30() {
    // May 31 + 1M = Jun 30
    assert_eq!(apply_and_format("2024-05-31T12:00:00Z", Duration::Months(1)), "2024-06-30T12:00:00Z");
}

#[test]
fn add_month_clamp_aug31_to_sep30() {
    // Aug 31 + 1M = Sep 30
    assert_eq!(apply_and_format("2024-08-31T12:00:00Z", Duration::Months(1)), "2024-09-30T12:00:00Z");
}

#[test]
fn subtract_month_clamp_mar31_to_feb29_leap() {
    // Mar 31 2024 - 1M = Feb 29 2024 (leap year)
    assert_eq!(apply_and_format("2024-03-31T12:00:00Z", Duration::Months(-1)), "2024-02-29T12:00:00Z");
}

#[test]
fn subtract_month_clamp_mar31_to_feb28_nonleap() {
    // Mar 31 2023 - 1M = Feb 28 2023 (non-leap year)
    assert_eq!(apply_and_format("2023-03-31T12:00:00Z", Duration::Months(-1)), "2023-02-28T12:00:00Z");
}

#[test]
fn subtract_month_clamp_mar30_to_feb28_nonleap() {
    // Mar 30 2023 - 1M = Feb 28 2023 (clamped)
    assert_eq!(apply_and_format("2023-03-30T12:00:00Z", Duration::Months(-1)), "2023-02-28T12:00:00Z");
}

// Month Arithmetic - Multiple Months
#[test]
fn add_multiple_months() {
    // Jan 15 + 6M = Jul 15
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(6)), "2024-07-15T12:00:00Z");
    // Jan 15 + 12M = Jan 15 next year
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(12)), "2025-01-15T12:00:00Z");
    // Jan 15 + 24M = Jan 15 two years later
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(24)), "2026-01-15T12:00:00Z");
}

#[test]
fn subtract_multiple_months() {
    // Jul 15 - 6M = Jan 15
    assert_eq!(apply_and_format("2024-07-15T12:00:00Z", Duration::Months(-6)), "2024-01-15T12:00:00Z");
    // Jan 15 - 12M = Jan 15 previous year
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(-12)), "2023-01-15T12:00:00Z");
}

#[test]
fn add_13_months() {
    // Jan 15 + 13M = Feb 15 next year
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Months(13)), "2025-02-15T12:00:00Z");
}

// Year Arithmetic - Basic
#[test]
fn add_one_year() {
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Years(1)), "2025-01-15T12:00:00Z");
    assert_eq!(apply_and_format("2024-06-15T12:00:00Z", Duration::Years(1)), "2025-06-15T12:00:00Z");
    assert_eq!(apply_and_format("2024-12-31T23:59:59Z", Duration::Years(1)), "2025-12-31T23:59:59Z");
}

#[test]
fn subtract_one_year() {
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Years(-1)), "2023-01-15T12:00:00Z");
    assert_eq!(apply_and_format("2024-12-31T12:00:00Z", Duration::Years(-1)), "2023-12-31T12:00:00Z");
}

#[test]
fn add_multiple_years() {
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Years(5)), "2029-01-15T12:00:00Z");
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Years(10)), "2034-01-15T12:00:00Z");
    assert_eq!(apply_and_format("2024-01-15T12:00:00Z", Duration::Years(100)), "2124-01-15T12:00:00Z");
}

// Year Arithmetic - Leap Year Edge Cases
#[test]
fn add_year_from_feb29_to_feb28() {
    // Feb 29 2024 (leap) + 1Y = Feb 28 2025 (non-leap, clamped)
    assert_eq!(apply_and_format("2024-02-29T12:00:00Z", Duration::Years(1)), "2025-02-28T12:00:00Z");
}

#[test]
fn add_year_from_feb29_to_feb29() {
    // Feb 29 2024 + 4Y = Feb 29 2028 (both leap years)
    assert_eq!(apply_and_format("2024-02-29T12:00:00Z", Duration::Years(4)), "2028-02-29T12:00:00Z");
}

#[test]
fn subtract_year_from_feb29_to_feb28() {
    // Feb 29 2024 - 1Y = Feb 28 2023 (clamped)
    assert_eq!(apply_and_format("2024-02-29T12:00:00Z", Duration::Years(-1)), "2023-02-28T12:00:00Z");
}

#[test]
fn add_year_preserves_feb28() {
    // Feb 28 2023 + 1Y = Feb 28 2024 (no clamping needed)
    assert_eq!(apply_and_format("2023-02-28T12:00:00Z", Duration::Years(1)), "2024-02-28T12:00:00Z");
}

#[test]
fn century_leap_year_rules() {
    // 2000 was a leap year (divisible by 400)
    // 2100 will NOT be a leap year (divisible by 100 but not 400)
    // Feb 29 2096 + 4Y = Feb 28 2100 (clamped, 2100 not a leap year)
    assert_eq!(apply_and_format("2096-02-29T12:00:00Z", Duration::Years(4)), "2100-02-28T12:00:00Z");
}

// Time Preservation
#[test]
fn month_addition_preserves_time() {
    // Time component should be preserved
    assert_eq!(apply_and_format("2024-01-15T08:30:45Z", Duration::Months(1)), "2024-02-15T08:30:45Z");
    assert_eq!(apply_and_format("2024-01-15T23:59:59Z", Duration::Months(1)), "2024-02-15T23:59:59Z");
}

#[test]
fn year_addition_preserves_time() {
    assert_eq!(apply_and_format("2024-01-15T08:30:45Z", Duration::Years(1)), "2025-01-15T08:30:45Z");
}

// is_duration Tests
#[test]
fn is_duration_fixed_units() {
    assert!(is_duration("+3h"));
    assert!(is_duration("-7d"));
    assert!(is_duration("30s"));
    assert!(is_duration("2w"));
}

#[test]
fn is_duration_calendar_units() {
    assert!(is_duration("1M"));
    assert!(is_duration("+3M"));
    assert!(is_duration("-6M"));
    assert!(is_duration("1Y"));
    assert!(is_duration("+2Y"));
    assert!(is_duration("-1Y"));
    assert!(is_duration("1y"));
}

#[test]
fn is_duration_false_for_epoch() {
    assert!(!is_duration("1704912345"));
    assert!(!is_duration("0"));
}

#[test]
fn is_duration_false_for_keywords() {
    assert!(!is_duration("now"));
    assert!(!is_duration(""));
}

// Roundtrip Tests
#[test]
fn roundtrip_epoch_iso() {
    for epoch in [0i64, 1000000000, 1704912345, 2000000000, -86400] {
        let iso = format_iso(epoch).unwrap();
        let back = parse_iso(&iso).unwrap();
        assert_eq!(epoch, back);
    }
}

#[test]
fn roundtrip_month_arithmetic() {
    // Adding and subtracting same months should return to original (when no clamping)
    let original = parse_iso("2024-01-15T12:00:00Z").unwrap();
    let after = apply_duration(original, Duration::Months(6)).unwrap();
    let back = apply_duration(after, Duration::Months(-6)).unwrap();
    assert_eq!(original, back);
}

#[test]
fn roundtrip_year_arithmetic() {
    // Adding and subtracting same years should return to original (when no clamping)
    let original = parse_iso("2024-01-15T12:00:00Z").unwrap();
    let after = apply_duration(original, Duration::Years(5)).unwrap();
    let back = apply_duration(after, Duration::Years(-5)).unwrap();
    assert_eq!(original, back);
}

// Note: Roundtrip with clamping dates (like Feb 29) is NOT guaranteed to work
// because information is lost during clamping. This is expected behaviour.
