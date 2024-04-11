use std::error::Error;
use std::str::FromStr;

use chrono::{DateTime, NaiveTime, Timelike, Utc};
use chrono_tz::America::Sao_Paulo as SAO_PAULO;
use chrono_tz::{ParseError, Tz, CET, EET};

pub fn parse_tz(text: &str) -> Result<Tz, ParseError> {
    match text.to_lowercase().as_str() {
        "europe" => Ok(CET),
        "madrid" | "barcelona" | "spain" | "es" => Ok(CET),
        "brazil" | "brasil" | "brt" | "br" => Ok(SAO_PAULO),
        "netherlands" | "amsterdam" | "nl" => Ok(CET),
        "romania" | "romenia" | "ro" => Ok(EET),
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
        SAO_PAULO => "BRT".to_string(),
        _ => tz.to_string(),
    }
}

fn clean_time(time: &str) -> String {
    time.replace(['H', 'h'], "")
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
    use chrono_tz::UTC;

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
        assert_eq!(result, Ok(SAO_PAULO));
        let result = parse_tz("CET");
        assert_eq!(result, Ok(CET));
    }
}
