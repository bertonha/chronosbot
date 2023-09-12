use chrono::{Timelike, Utc};
use regex::Regex;
use std::error::Error;

use crate::time::{format_time, parse_time, parse_tz};

pub fn start() -> String {
    "Welcome!".to_string()
}

pub fn now(timezone: &str) -> String {
    let tz = match parse_tz(timezone) {
        Ok(tz) => tz,
        _ => return format!("Invalid timezone: {timezone}").to_string(),
    };
    format_time(Utc::now().with_timezone(&tz))
}

pub fn convert_time(input: &str) -> Result<String, Box<dyn Error>> {
    let re = Regex::new(r"(\d{1,2}:?\d{0,2}) (\w*) (\w*)")?;

    // Check if the input string matches the pattern
    if let Some(captures) = re.captures(input) {
        let source_time = captures.get(1).unwrap().as_str();
        let source_timezone = parse_tz(captures.get(2).unwrap().as_str())?;
        let target_timezone = parse_tz(captures.get(3).unwrap().as_str())?;

        let time = parse_time(source_time)?;
        let source_time = Utc::now()
            .with_timezone(&source_timezone)
            .with_hour(time.hour())
            .unwrap()
            .with_minute(time.minute())
            .unwrap()
            .with_second(0)
            .unwrap();
        let target_time = source_time.with_timezone(&target_timezone);
        Ok(format_time(target_time))
    } else {
        Err(Box::try_from("Invalid format".to_string()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_time_brt_cet() {
        let result = convert_time("12:00 BRT CET");
        assert_eq!(result.ok(), Some("17:00:00".to_string()));
    }
    #[test]
    fn test_convert_time_utc_brl() {
        let result = convert_time("12:00 UTC BRT");
        assert_eq!(result.ok(), Some("09:00:00".to_string()));
    }
    #[test]
    fn test_convert_time_one_digit() {
        let result = convert_time("1:00 BRT CET");
        assert_eq!(result.ok(), Some("06:00:00".to_string()));
    }

    #[test]
    fn test_convert_time_minimal() {
        let result = convert_time("1 BRT CET");
        assert_eq!(result.ok(), Some("06:00:00".to_string()));
    }

    #[test]
    fn test_convert_time_missing_target_tz() {
        let result = convert_time("12:00 UTC");
        assert!(result.is_err());
    }
}
