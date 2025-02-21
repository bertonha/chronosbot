use std::error::Error;

use chrono_tz::{ParseError, Tz};
use itertools::Itertools;

use crate::converter::Converter;
use crate::time;

pub fn process_input(text: &str) -> String {
    let (command, rest) = text.split_once(' ').unwrap_or((text, ""));
    match command {
        "/start" => command_start(),
        "/now" => command_now(rest).unwrap_or_else(|e| e.to_string()),
        "/convert" => command_convert(rest).unwrap_or(convert_error()),
        _ => normal_message(text),
    }
}

fn normal_message(src_text: &str) -> String {
    command_convert(src_text).unwrap_or(command_now(src_text).unwrap_or(invalid_command()))
}

pub fn convert_from_input_or_default_timezones(
    src_text: &str,
    default_timezones: Vec<Tz>,
) -> Result<Converter, Box<dyn Error>> {
    let converter = match time::parse_time(src_text) {
        Ok(time) => Converter::new(Some(time), default_timezones),
        Err(_) => Converter::try_from(src_text)?,
    };
    Ok(converter)
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

fn command_now(timezone: &str) -> Result<String, ParseError> {
    let ret = Converter::try_from_only_timezones(timezone)?
        .now_in_timezones()
        .join(" - ");
    Ok(ret)
}

fn command_convert(input: &str) -> Result<String, Box<dyn Error>> {
    let ret = Converter::try_from(input)?
        .convert_time_between_timezones()
        .next()
        .unwrap_or_else(|| "No time to convert".to_string());
    Ok(ret)
}

fn convert_error() -> String {
    format!("Invalid pattern. Please follow correct pattern as bellow\n\n{CONVERT_COMMAND_INFO}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::is_dst;

    #[test]
    fn test_convert_time_brt_cet() {
        let cet_hour = if is_dst(Tz::CET) { "17:00" } else { "16:00" };
        let result = command_convert("12:00 BRT CET");
        assert_eq!(result.unwrap(), format!("12:00 BRT - {cet_hour} CET"));
    }

    #[test]
    fn test_convert_time_utc_brl() {
        let result = command_convert("12:00 UTC BRT");
        assert_eq!(result.unwrap(), "12:00 UTC - 09:00 BRT");
    }

    #[test]
    fn test_convert_time_one_digit() {
        let cet_hour = if is_dst(Tz::CET) { "06:00" } else { "05:00" };
        let result = command_convert("1:00 BRT CET");
        assert_eq!(result.unwrap(), format!("01:00 BRT - {cet_hour} CET"));
    }

    #[test]
    fn test_convert_time_minimal() {
        let cet_hour = if is_dst(Tz::CET) { "07:00" } else { "06:00" };
        let result = command_convert("2 BRT CET");
        assert_eq!(result.unwrap(), format!("02:00 BRT - {cet_hour} CET"));
    }

    #[test]
    fn test_convert_time_multiple_spaces() {
        let bucharest_hour = if is_dst(Tz::Europe__Bucharest) {
            "18:00"
        } else {
            "17:00"
        };
        let result = command_convert("12:00    BRT     RO    ");
        assert_eq!(
            result.unwrap(),
            format!("12:00 BRT - {bucharest_hour} Europe/Bucharest")
        );
    }

    #[test]
    fn test_convert_time_missing_target_tz() {
        let result = command_convert("12:00 UTC");
        assert_eq!(result.unwrap(), "12:00 UTC");
    }

    #[test]
    fn test_process_command_start() {
        let result = process_input("/start");
        assert_eq!(result, command_start());
    }

    #[test]
    fn test_process_command_now() {
        let result = process_input("/now utc");
        assert_eq!(result, command_now("utc").unwrap());
    }

    #[test]
    fn test_process_command_now_multiple_spaces() {
        let result = process_input("/now   utc    ");
        assert_eq!(result, command_now("utc").unwrap());
    }

    #[test]
    fn test_process_command_convert() {
        let result = process_input("/convert 12:00 UTC BRT");
        assert_eq!(result, command_convert("12:00 UTC BRT").unwrap());
    }

    #[test]
    fn test_process_command_with_h_convert() {
        let result = process_input("/convert 12h UTC BRT");
        assert_eq!(result, command_convert("12:00 UTC BRT").unwrap());
    }

    #[test]
    fn test_process_command_invalid() {
        let result = process_input("invalid");
        assert_eq!(result, invalid_command());
    }
}
