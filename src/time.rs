use std::str::FromStr;

use chrono::{DateTime, LocalResult, NaiveTime, TimeDelta, Utc};
use chrono_tz::{OffsetComponents, Tz};

use crate::error::BotError;

pub fn parse_tz(text: &str) -> Result<Tz, BotError> {
    let tz = match text.to_lowercase().as_str() {
        "edt" | "est" => Tz::EST5EDT,
        "cdt" | "cst" => Tz::CST6CDT,
        "mdt" | "mst" => Tz::MST7MDT,
        "pdt" | "pst" => Tz::PST8PDT,
        "ist" | "irlanda" | "ireland" => Tz::Europe__Dublin,
        "europe" | "eu" => Tz::CET,
        "madrid" | "barcelona" | "spain" | "es" => Tz::Europe__Madrid,
        "brazil" | "brasil" | "brt" | "br" => Tz::America__Sao_Paulo,
        "netherlands" | "amsterdam" | "nl" => Tz::Europe__Amsterdam,
        "romania" | "romenia" | "ro" => Tz::Europe__Bucharest,
        _ => Tz::from_str_insensitive(text)?,
    };
    Ok(tz)
}

pub fn format_time(time: &DateTime<Tz>) -> String {
    time.format("%H:%M").to_string()
}

pub fn format_time_with_timezone(time: &DateTime<Tz>) -> String {
    format!("{} {}", format_time(time), format_timezone(time))
}

pub fn format_timezone(time: &DateTime<Tz>) -> String {
    match time.timezone() {
        Tz::America__Sao_Paulo => "BRT".to_string(),
        Tz::EST5EDT => dst_aware_abbreviation(time, "EST", "EDT"),
        Tz::CST6CDT => dst_aware_abbreviation(time, "CST", "CDT"),
        Tz::MST7MDT => dst_aware_abbreviation(time, "MST", "MDT"),
        Tz::PST8PDT => dst_aware_abbreviation(time, "PST", "PDT"),
        Tz::Europe__Dublin => "IST".to_string(),
        tz => tz.to_string(),
    }
}

fn is_dst(time: &DateTime<Tz>) -> bool {
    time.offset().dst_offset() != TimeDelta::zero()
}

fn dst_aware_abbreviation(time: &DateTime<Tz>, standard: &str, daylight: &str) -> String {
    if is_dst(time) { daylight } else { standard }.to_string()
}

fn clean_time(time: &str) -> String {
    let cleaned_time = time.replace(['H', 'h'], ":");
    if cleaned_time.ends_with(':') {
        cleaned_time + "00"
    } else {
        cleaned_time
    }
}

pub fn parse_time(text: &str) -> Result<NaiveTime, BotError> {
    let clean_text = clean_time(text);
    NaiveTime::from_str(&clean_text)
        .ok()
        .or_else(|| {
            let hour = clean_text.parse::<u32>().ok()?;
            NaiveTime::from_hms_opt(hour, 0, 0)
        })
        .ok_or_else(|| BotError::InvalidTime(text.to_string()))
}

pub fn time_with_timezone(
    time: &NaiveTime,
    tz: &Tz,
    now: DateTime<Utc>,
) -> Result<DateTime<Tz>, BotError> {
    let date = now.with_timezone(tz).date_naive();
    match date.and_time(*time).and_local_timezone(*tz) {
        LocalResult::Single(datetime) => Ok(datetime),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest),
        LocalResult::None => Err(BotError::NonexistentTime {
            time: *time,
            tz: *tz,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn winter_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn summer_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_parse_time_complete() {
        let result = parse_time("12:13:45");
        assert_eq!(result.ok(), NaiveTime::from_hms_opt(12, 13, 45));
    }

    #[test]
    fn test_parse_time_hour_minute() {
        let result = parse_time("12:45");
        assert_eq!(result.ok(), NaiveTime::from_hms_opt(12, 45, 0));
    }

    #[test]
    fn test_parse_time_hour() {
        let result = parse_time("12");
        assert_eq!(result.ok(), NaiveTime::from_hms_opt(12, 0, 0));
    }

    #[test]
    fn test_parse_time_hour_with_h() {
        let result = parse_time("12H");
        assert_eq!(result.ok(), NaiveTime::from_hms_opt(12, 0, 0));
    }

    #[test]
    fn test_parse_time_hour_with_h_and_minutes() {
        let result = parse_time("12h30");
        assert_eq!(result.ok(), NaiveTime::from_hms_opt(12, 30, 0));
    }

    #[test]
    fn test_parse_time_invalid() {
        let result = parse_time("HALO");
        assert_eq!(result, Err(BotError::InvalidTime("HALO".into())));
    }

    #[test]
    fn test_parse_tz() {
        assert_eq!(parse_tz("UTC"), Ok(Tz::UTC));
        assert_eq!(parse_tz("BRT"), Ok(Tz::America__Sao_Paulo));
        assert_eq!(parse_tz("CET"), Ok(Tz::CET));
        assert_eq!(parse_tz("PST"), Ok(Tz::PST8PDT));
    }

    #[test]
    fn test_format_timezone() {
        let now = winter_now();
        assert_eq!(format_timezone(&now.with_timezone(&Tz::UTC)), "UTC");
        assert_eq!(
            format_timezone(&now.with_timezone(&Tz::America__Sao_Paulo)),
            "BRT"
        );
        assert_eq!(format_timezone(&now.with_timezone(&Tz::CET)), "CET");
    }

    #[test]
    fn test_format_timezone_dst_aware() {
        assert_eq!(
            format_timezone(&winter_now().with_timezone(&Tz::PST8PDT)),
            "PST"
        );
        assert_eq!(
            format_timezone(&summer_now().with_timezone(&Tz::PST8PDT)),
            "PDT"
        );
        assert_eq!(
            format_timezone(&winter_now().with_timezone(&Tz::EST5EDT)),
            "EST"
        );
        assert_eq!(
            format_timezone(&summer_now().with_timezone(&Tz::EST5EDT)),
            "EDT"
        );
    }

    #[test]
    fn test_time_with_timezone_nonexistent() {
        // CET skips 02:00-03:00 on 2026-03-29 (spring forward).
        let now = Utc.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).unwrap();
        let time = NaiveTime::from_hms_opt(2, 30, 0).unwrap();
        assert_eq!(
            time_with_timezone(&time, &Tz::CET, now),
            Err(BotError::NonexistentTime { time, tz: Tz::CET })
        );
    }

    #[test]
    fn test_time_with_timezone_ambiguous_picks_earliest() {
        // CET repeats 02:00-03:00 on 2026-10-25 (fall back).
        let now = Utc.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).unwrap();
        let time = NaiveTime::from_hms_opt(2, 30, 0).unwrap();
        let result = time_with_timezone(&time, &Tz::CET, now).unwrap();
        assert_eq!(
            result.to_utc(),
            Utc.with_ymd_and_hms(2026, 10, 25, 0, 30, 0).unwrap()
        );
    }
}
