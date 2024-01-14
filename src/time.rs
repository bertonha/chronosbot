#![allow(non_upper_case_globals)]

use chrono::{DateTime, NaiveTime, Timelike, Utc};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::Europe::{Amsterdam, Bucharest, Madrid};
use chrono_tz::{ParseError, Tz, CET, EET, EST, UTC};
use std::error::Error;

pub fn parse_tz(text: &str) -> Result<Tz, ParseError> {
    match text.to_lowercase().as_str() {
        "utc" => Ok(UTC),
        "cet" | "europe" => Ok(CET),
        "eet" => Ok(EET),
        "est" => Ok(EST),
        "madrid" | "barcelona" | "spain" | "es" => Ok(Madrid),
        "brazil" | "brasil" | "brt" | "br" => Ok(Sao_Paulo),
        "netherlands" | "amsterdam" | "nl" => Ok(Amsterdam),
        "romania" | "romenia" | "ro" => Ok(Bucharest),
        _ => text.parse(),
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
        Madrid | Amsterdam => "CET".to_string(),
        Sao_Paulo => "BRT".to_string(),
        Bucharest => "EET".to_string(),
        _ => tz.to_string(),
    }
}

fn clean_time(time: &str) -> String {
    time.replace(['H', 'h'], "")
}

pub fn parse_time(text: &str) -> Result<NaiveTime, Box<dyn Error>> {
    let clean_text = clean_time(text);
    match NaiveTime::parse_from_str(&clean_text, "%H:%M:%S") {
        Ok(time) => Ok(time),
        Err(_) => match NaiveTime::parse_from_str(&clean_text, "%H:%M") {
            Ok(time) => Ok(time),
            Err(error) => {
                let hour = clean_text.parse::<u32>()?;
                match NaiveTime::from_hms_opt(hour, 0, 0) {
                    Some(time) => Ok(time),
                    None => Err(Box::new(error)),
                }
            }
        },
    }
}

pub fn time_with_timezone(time: &str, timezone: &str) -> Result<DateTime<Tz>, Box<dyn Error>> {
    let time = parse_time(time)?;
    let source_tz = parse_tz(timezone)?;
    Ok(Utc::now()
        .with_timezone(&source_tz)
        .with_hour(time.hour())
        .unwrap()
        .with_minute(time.minute())
        .unwrap()
        .with_second(0)
        .unwrap())
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
    fn test_parse_time_invalid() {
        let result = parse_time("HALO");
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_tz() {
        let result = parse_tz("UTC");
        assert_eq!(result, Ok(UTC));
        let result = parse_tz("BRT");
        assert_eq!(result, Ok(Sao_Paulo));
        let result = parse_tz("CET");
        assert_eq!(result, Ok(CET));
    }
}
