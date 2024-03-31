use std::error::Error;

use chrono_tz::Tz;
use itertools::Itertools;

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

fn command_now(timezone: &str) -> String {
    match Converter::try_from_only_timezones(timezone) {
        Ok(converter) => converter.now_in_timezones().join(" - "),
        Err(error) => error.to_string(),
    }
}

fn command_convert(input: &str) -> String {
    match Converter::try_from(input) {
        Ok(converter) => converter
            .convert_time_between_timezones()
            .next()
            .unwrap_or_else(|| "No time to convert".to_string()),
        Err(_) => convert_error(),
    }
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
        assert_eq!(result, format!("12:00 BRT - {cet_hour} CET"));
    }

    #[test]
    fn test_convert_time_utc_brl() {
        let result = command_convert("12:00 UTC BRT");
        assert_eq!(result, "12:00 UTC - 09:00 BRT");
    }

    #[test]
    fn test_convert_time_one_digit() {
        let cet_hour = if is_dst(Tz::CET) { "06:00" } else { "05:00" };
        let result = command_convert("1:00 BRT CET");
        assert_eq!(result, format!("01:00 BRT - {cet_hour} CET"));
    }

    #[test]
    fn test_convert_time_minimal() {
        let cet_hour = if is_dst(Tz::CET) { "07:00" } else { "06:00" };
        let result = command_convert("2 BRT CET");
        assert_eq!(result, format!("02:00 BRT - {cet_hour} CET"));
    }

    #[test]
    fn test_convert_time_multiple_spaces() {
        let eet_hour = if is_dst(Tz::EET) { "18:00" } else { "17:00" };
        let result = command_convert("12:00    BRT     RO    ");
        assert_eq!(result, format!("12:00 BRT - {eet_hour} EET"));
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
