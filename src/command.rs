use std::error::Error;

use chrono::{DateTime, Timelike, Utc};
use chrono_tz::{ParseError, Tz};

use crate::time::{format_time_with_timezone, parse_time, parse_tz};

pub fn process_command(text: &str) -> String {
    let (command, rest) = text.split_once(' ').unwrap_or((text, ""));
    match command {
        "/start" => command_start(),
        "/now" => command_now(rest),
        "/convert" => command_convert(rest),
        _ => invalid_command(),
    }
}

const NOW_COMMAND_INFO: &str = "/now <timezone>";
const CONVERT_COMMAND_INFO: &str = "/convert <time> <source_timezone> <target_timezone>";

fn command_list() -> String {
    format!(
        "Commands accepted:\n\
        /start\n\
        {NOW_COMMAND_INFO}\n\
        {CONVERT_COMMAND_INFO}"
    )
}

fn invalid_command() -> String {
    format!("Invalid command.\n\n{}", command_list())
}

fn command_start() -> String {
    format!("Welcome!\n\n{}", command_list())
}

fn command_now(timezone: &str) -> String {
    match now(timezone) {
        Ok(time) => format_time_with_timezone(time),
        Err(error) => error.to_string(),
    }
}

fn command_convert(input: &str) -> String {
    convert(input).unwrap_or_else(|e| e.to_string())
}

fn now(timezone: &str) -> Result<DateTime<Tz>, ParseError> {
    let tz = parse_tz(timezone.trim())?;
    Ok(Utc::now().with_timezone(&tz))
}

pub fn convert(input: &str) -> Result<String, Box<dyn Error>> {
    let split_values = input.split_whitespace().collect::<Vec<&str>>();
    let time_index;

    let source_time = match split_values.len() {
        2 => {
            time_index = 1;
            match now(split_values[0]) {
                Ok(time) => time,
                Err(_) => {
                    return Err(convert_error().into());
                }
            }
        }
        3 => {
            time_index = 2;
            if split_values[0] == "now" {
                now(split_values[1])?
            } else {
                let time = parse_time(split_values[0])?;
                let source_tz = parse_tz(split_values[1])?;
                Utc::now()
                    .with_timezone(&source_tz)
                    .with_hour(time.hour())
                    .unwrap()
                    .with_minute(time.minute())
                    .unwrap()
                    .with_second(0)
                    .unwrap()
            }
        }
        _ => {
            return Err(convert_error().into());
        }
    };

    let target_tz = parse_tz(split_values[time_index])?;

    let target_time = source_time.with_timezone(&target_tz);
    Ok(format!(
        "{} - {}",
        format_time_with_timezone(source_time),
        format_time_with_timezone(target_time)
    ))
}

fn convert_error() -> String {
    format!("Invalid pattern. Please follow correct pattern as bellow\n\n{CONVERT_COMMAND_INFO}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_time_brt_cet() {
        let result = command_convert("12:00 BRT CET");
        assert_eq!(result, "12:00 BRT - 17:00 CET");
    }
    #[test]
    fn test_convert_time_utc_brl() {
        let result = command_convert("12:00 UTC BRT");
        assert_eq!(result, "12:00 UTC - 09:00 BRT");
    }
    #[test]
    fn test_convert_time_one_digit() {
        let result = command_convert("1:00 BRT CET");
        assert_eq!(result, "01:00 BRT - 06:00 CET");
    }
    #[test]
    fn test_convert_time_minimal() {
        let result = command_convert("1 BRT CET");
        assert_eq!(result, "01:00 BRT - 06:00 CET");
    }
    #[test]
    fn test_convert_time_multiple_spaces() {
        let result = command_convert("12:00    BRT     RO    ");
        assert_eq!(result, "12:00 BRT - 18:00 EET");
    }
    #[test]
    fn test_convert_time_missing_target_tz() {
        let result = command_convert("12:00 UTC");
        assert_eq!(result, convert_error());
    }
    #[test]
    fn test_process_command_start() {
        let result = process_command("/start");
        assert_eq!(result, command_start());
    }
    #[test]
    fn test_process_command_now() {
        let result = process_command("/now utc");
        assert_eq!(result, command_now("utc"));
    }
    #[test]
    fn test_process_command_now_multiple_spaces() {
        let result = process_command("/now   utc    ");
        assert_eq!(result, command_now("utc"));
    }
    #[test]
    fn test_process_command_convert() {
        let result = process_command("/convert 12:00 UTC BRT");
        assert_eq!(result, command_now("12:00 UTC BRT").unwrap());
    }

    #[test]
    fn test_process_command_invalid() {
        let result = process_command("invalid");
        assert_eq!(result, invalid_command());
    }
}
