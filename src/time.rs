use chrono::{DateTime, NaiveTime};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::Europe::Bucharest;
use chrono_tz::{ParseError, Tz, CET, UTC};
use std::error::Error;

pub fn parse_tz(text: &str) -> Result<Tz, ParseError> {
    match text.parse() {
        Ok(tz) => Ok(tz),
        Err(error) => match text.to_lowercase().as_str() {
            "utc" => Ok(UTC),
            "cet" | "europe" => Ok(CET),
            "brazil" | "brasil" | "brt" => Ok(Sao_Paulo),
            "romania" => Ok(Bucharest),
            _ => Err(error),
        },
    }
}

pub fn format_time(time: DateTime<Tz>) -> String {
    time.format("%H:%M:%S").to_string()
}

pub fn parse_time(text: &str) -> Result<NaiveTime, Box<dyn Error>> {
    match NaiveTime::parse_from_str(text, "%H:%M:%S") {
        Ok(time) => Ok(time),
        Err(_) => match NaiveTime::parse_from_str(text, "%H:%M") {
            Ok(time) => Ok(time),
            Err(error) => {
                let hour = text.parse::<u32>()?;
                match NaiveTime::from_hms_opt(hour, 0, 0) {
                    Some(time) => Ok(time),
                    None => Err(Box::try_from(error).unwrap()),
                }
            }
        },
    }
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
    fn test_parse_time_invalid() {
        let result = parse_time("HALO");
        assert!(result.is_err());
    }
}
