use std::error::Error;

use chrono::{Timelike, Utc};

use crate::time::{format_time_with_timezone, parse_time, parse_tz};

pub fn process_command(text: &str) -> String {
    let (command, rest) = text.split_once(' ').unwrap_or((text, ""));
    match command {
        "/start" => start(),
        "/now" => now(rest),
        "/convert" => convert(rest).unwrap_or_else(|e| e.to_string()),
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

fn start() -> String {
    format!("Welcome!\n\n{}", command_list())
}

fn now(timezone: &str) -> String {
    let tz = match parse_tz(timezone.trim()) {
        Ok(tz) => tz,
        Err(err) => return err.to_string(),
    };
    let now = Utc::now().with_timezone(&tz);
    format_time_with_timezone(now)
}

pub fn convert(input: &str) -> Result<String, Box<dyn Error>> {
    let bla = input.split_whitespace().collect::<Vec<&str>>();

    if bla.len() != 3 {
        return Err(format!(
            "Invalid pattern. Please follow correct pattern as bellow\n\n{CONVERT_COMMAND_INFO}"
        )
        .into());
    }

    let time = parse_time(bla[0])?;
    let source_tz = parse_tz(bla[1])?;
    let target_tz = parse_tz(bla[2])?;

    let source_time = Utc::now()
        .with_timezone(&source_tz)
        .with_hour(time.hour())
        .unwrap()
        .with_minute(time.minute())
        .unwrap()
        .with_second(0)
        .unwrap();
    let target_time = source_time.with_timezone(&target_tz);
    Ok(format!(
        "{} - {}",
        format_time_with_timezone(source_time),
        format_time_with_timezone(target_time)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_time_brt_cet() {
        let result = convert("12:00 BRT CET");
        assert_eq!(result.unwrap(), "12:00 BRT - 17:00 CET");
    }
    #[test]
    fn test_convert_time_utc_brl() {
        let result = convert("12:00 UTC BRT");
        assert_eq!(result.unwrap(), "12:00 UTC - 09:00 BRT");
    }
    #[test]
    fn test_convert_time_one_digit() {
        let result = convert("1:00 BRT CET");
        assert_eq!(result.unwrap(), "01:00 BRT - 06:00 CET");
    }
    #[test]
    fn test_convert_time_minimal() {
        let result = convert("1 BRT CET");
        assert_eq!(result.unwrap(), "01:00 BRT - 06:00 CET");
    }
    #[test]
    fn test_convert_time_multiple_spaces() {
        let result = convert("12:00    BRT     RO    ");
        assert_eq!(result.unwrap(), "12:00 BRT - 18:00 EET");
    }
    #[test]
    fn test_convert_time_missing_target_tz() {
        let result = convert("12:00 UTC");
        assert!(result.is_err());
    }
    #[test]
    fn test_process_command_start() {
        let result = process_command("/start");
        assert_eq!(result, start());
    }
    #[test]
    fn test_process_command_now() {
        let result = process_command("/now utc");
        assert_eq!(result, now("utc"));
    }
    #[test]
    fn test_process_command_now_multiple_spaces() {
        let result = process_command("/now   utc    ");
        assert_eq!(result, now("utc"));
    }
    #[test]
    fn test_process_command_convert() {
        let result = process_command("/convert 12:00 UTC BRT");
        assert_eq!(result, convert("12:00 UTC BRT").unwrap());
    }
    #[test]
    fn test_process_command_invalid() {
        let result = process_command("invalid");
        assert_eq!(result, invalid_command());
    }
}
