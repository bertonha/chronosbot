use std::error::Error;

use chrono::{DateTime, Utc};
use chrono_tz::{ParseError, Tz};

use crate::time::{format_time_with_timezone, parse_tz, time_with_timezone};

pub fn process_command(text: &str) -> String {
    let (command, rest) = text.split_once(' ').unwrap_or((text, ""));
    match command {
        "/start" => command_start(),
        "/now" => command_now(rest),
        "/convert" => command_convert(rest),
        _ => invalid_command(),
    }
}

const CONVERT_COMMAND_INFO: &str = "<time> <source_timezone> <target_timezone>";

fn command_list() -> String {
    format!(
        "Commands accepted:\n\
        /start\n\
        /now <timezone>\n\
        /convert {CONVERT_COMMAND_INFO}"
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
                time_with_timezone(split_values[0], split_values[1])?
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
    use chrono::Duration;
    use chrono_tz::OffsetComponents;

    fn now_in_timezone(tz: &Tz) -> DateTime<Tz> {
        Utc::now().with_timezone(&tz)
    }

    fn is_dst(tz: &Tz) -> bool {
        let now = now_in_timezone(&tz);
        now.offset().dst_offset() == Duration::seconds(0)
    }

    #[test]
    fn test_convert_time_brt_cet() {
        let cet_hour = if is_dst(&Tz::CET) { 16 } else { 17 };
        let result = command_convert("12:00 BRT CET");
        assert_eq!(result, format!("12:00 BRT - {cet_hour}:00 CET"));
    }
    #[test]
    fn test_convert_time_utc_brl() {
        let result = command_convert("12:00 UTC BRT");
        assert_eq!(result, "12:00 UTC - 09:00 BRT");
    }
    #[test]
    fn test_convert_time_one_digit() {
        let cet_hour = if is_dst(&Tz::CET) { "05" } else { "06" };
        let result = command_convert("1:00 BRT CET");
        assert_eq!(result, format!("01:00 BRT - {cet_hour}:00 CET"));
    }
    #[test]
    fn test_convert_time_minimal() {
        let cet_hour = if is_dst(&Tz::CET) { "05" } else { "06" };
        let result = command_convert("1 BRT CET");
        assert_eq!(result, format!("01:00 BRT - {cet_hour}:00 CET"));
    }
    #[test]
    fn test_convert_time_multiple_spaces() {
        let eet_hour = if is_dst(&Tz::EET) { 17 } else { 18 };
        let result = command_convert("12:00    BRT     RO    ");
        assert_eq!(result, format!("12:00 BRT - {eet_hour}:00 EET"));
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
        assert_eq!(result, command_convert("12:00 UTC BRT"));
    }

    #[test]
    fn test_process_command_invalid() {
        let result = process_command("invalid");
        assert_eq!(result, invalid_command());
    }
}
