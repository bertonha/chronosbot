use std::error::Error;
use std::str::FromStr;

use crate::utils::is_dst;
use chrono::{DateTime, NaiveTime, Timelike, Utc};
use chrono_tz::{ParseError, Tz};

pub fn parse_tz(text: &str) -> Result<Tz, ParseError> {
    match text.to_lowercase().as_str() {
        "edt" | "est" => Ok(Tz::EST5EDT),
        "cdt" | "cst" => Ok(Tz::CST6CDT),
        "mdt" | "mst" => Ok(Tz::MST7MDT),
        "pdt" | "pst" => Ok(Tz::PST8PDT),
        "europe" | "eu" => Ok(Tz::CET),
        "madrid" | "barcelona" | "spain" | "es" => Ok(Tz::Europe__Madrid),
        "brazil" | "brasil" | "brt" | "br" => Ok(Tz::America__Sao_Paulo),
        "netherlands" | "amsterdam" | "nl" => Ok(Tz::Europe__Amsterdam),
        "romania" | "romenia" | "ro" => Ok(Tz::Europe__Bucharest),
        _ => Tz::from_str_insensitive(text),
    }
}

pub fn format_time(time: DateTime<Tz>) -> String {
    time.format("%H:%M").to_string()
}

pub fn format_time_with_timezone(time: DateTime<Tz>) -> String {
    format!("{} {}", format_time(time), format_timezone(time.timezone()))
}

pub fn format_timezone(tz: Tz) -> String {
    match tz {
        Tz::America__Sao_Paulo => "BRT".to_string(),
        Tz::EST5EDT => demultiplexer_timezone(tz, "EST", "EDT"),
        Tz::CST6CDT => demultiplexer_timezone(tz, "CST", "CDT"),
        Tz::MST7MDT => demultiplexer_timezone(tz, "MST", "MDT"),
        Tz::PST8PDT => demultiplexer_timezone(tz, "PST", "PDT"),
        _ => tz.to_string(),
    }
}

fn demultiplexer_timezone(tz: Tz, summer_time: &str, standard_time: &str) -> String {
    if is_dst(tz) {
        summer_time.to_string()
    } else {
        standard_time.to_string()
    }
}

fn clean_time(time: &str) -> String {
    let cleaned_time = time.replace(['H', 'h'], ":");
    if cleaned_time.ends_with(':') {
        cleaned_time + "00"
    } else {
        cleaned_time
    }
}

pub fn parse_time(text: &str) -> Result<NaiveTime, Box<dyn Error>> {
    let clean_text = clean_time(text);
    match NaiveTime::from_str(&clean_text) {
        Ok(time) => Ok(time),
        Err(error) => {
            let hour = clean_text.parse::<u32>()?;
            NaiveTime::from_hms_opt(hour, 0, 0).ok_or(error.into())
        }
    }
}

pub fn now_on_timezone(tz: &Tz) -> DateTime<Tz> {
    Utc::now().with_timezone(tz)
}

pub fn time_with_timezone(time: &NaiveTime, tz: &Tz) -> DateTime<Tz> {
    Utc::now()
        .with_timezone(tz)
        .with_hour(time.hour())
        .unwrap()
        .with_minute(time.minute())
        .unwrap()
        .with_second(0)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(result.is_err());
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
        assert_eq!(format_timezone(Tz::UTC), "UTC");
        assert_eq!(format_timezone(Tz::America__Sao_Paulo), "BRT");
        assert_eq!(format_timezone(Tz::CET), "CET");
        let pacific_time = if is_dst(Tz::PST8PDT) { "PST" } else { "PDT" };
        assert_eq!(format_timezone(Tz::PST8PDT), pacific_time);
    }
}
