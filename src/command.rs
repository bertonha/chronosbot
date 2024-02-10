use std::error::Error;

use chrono::{DateTime, Utc};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::Tz::CET;
use chrono_tz::{ParseError, Tz};

use crate::converter::Converter;
use crate::time;

pub fn process_command(text: &str) -> String {
    let (command, rest) = text.split_once(' ').unwrap_or((text, ""));
    match command {
        "/start" => command_start(),
        "/now" => command_now(rest),
        "/convert" => command_convert(rest),
        _ => invalid_command(),
    }
}

pub fn convert_time_between_timezones(src_text: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let converter = match time::parse_time(src_text) {
        Ok(time) => Converter::new(time, vec![CET, Sao_Paulo]),
        Err(_) => Converter::try_from(src_text)?,
    };
    Ok(converter
        .convert_time_between_timezones()
        .collect::<Vec<String>>())
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
        Ok(time) => time::format_time_with_timezone(time),
        Err(error) => error.to_string(),
    }
}

fn command_convert(input: &str) -> String {
    match Converter::try_from(input) {
        Ok(converter) => converter
            .convert_time_between_timezones()
            .next()
            .unwrap_or("No time to convert".to_string()),
        Err(_) => convert_error(),
    }
}

fn now(timezone: &str) -> Result<DateTime<Tz>, ParseError> {
    let tz = time::parse_tz(timezone.trim())?;
    Ok(Utc::now().with_timezone(&tz))
}

fn convert_error() -> String {
    format!("Invalid pattern. Please follow correct pattern as bellow\n\n{CONVERT_COMMAND_INFO}")
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use chrono_tz::OffsetComponents;

    use super::*;

    pub fn is_dst(tz: Tz) -> bool {
        let now = Utc::now().with_timezone(&tz);
        now.offset().dst_offset() == Duration::seconds(0)
    }

    #[test]
    fn test_convert_time_brt_cet() {
        let cet_hour = if is_dst(Tz::CET) { 16 } else { 17 };
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
        let cet_hour = if is_dst(Tz::CET) { "05" } else { "06" };
        let result = command_convert("1:00 BRT CET");
        assert_eq!(result, format!("01:00 BRT - {cet_hour}:00 CET"));
    }

    #[test]
    fn test_convert_time_minimal() {
        let cet_hour = if is_dst(Tz::CET) { "05" } else { "06" };
        let result = command_convert("1 BRT CET");
        assert_eq!(result, format!("01:00 BRT - {cet_hour}:00 CET"));
    }

    #[test]
    fn test_convert_time_multiple_spaces() {
        let eet_hour = if is_dst(Tz::EET) { 17 } else { 18 };
        let result = command_convert("12:00    BRT     RO    ");
        assert_eq!(result, format!("12:00 BRT - {eet_hour}:00 EET"));
    }

    #[test]
    fn test_convert_time_missing_target_tz() {
        let result = command_convert("12:00 UTC");
        assert_eq!(result, "12:00 UTC");
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
    fn test_process_command_with_h_convert() {
        let result = process_command("/convert 12h UTC BRT");
        assert_eq!(result, command_convert("12:00 UTC BRT"));
    }

    #[test]
    fn test_process_command_invalid() {
        let result = process_command("invalid");
        assert_eq!(result, invalid_command());
    }
}
