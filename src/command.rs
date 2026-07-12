use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itertools::Itertools;

use crate::converter::Converter;
use crate::error::BotError;
use crate::time;

pub fn process_input(text: &str, now: DateTime<Utc>) -> String {
    let (command, rest) = text.split_once(' ').unwrap_or((text, ""));
    match command {
        "/start" => command_start(),
        "/now" => command_now(rest, now).unwrap_or_else(|e| e.to_string()),
        "/convert" => command_convert(rest, now).unwrap_or_else(|e| convert_error(&e)),
        _ => normal_message(text, now),
    }
}

fn normal_message(src_text: &str, now: DateTime<Utc>) -> String {
    command_convert(src_text, now)
        .or_else(|_| command_now(src_text, now))
        .unwrap_or_else(|_| invalid_command())
}

pub fn convert_from_input_or_default_timezones(
    src_text: &str,
    default_timezones: &[Tz],
) -> Result<Converter, BotError> {
    let converter = match time::parse_time(src_text) {
        Ok(time) => Converter::new(Some(time), default_timezones.to_vec()),
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

fn command_now(timezone: &str, now: DateTime<Utc>) -> Result<String, BotError> {
    let ret = Converter::try_from_only_timezones(timezone)?
        .now_in_timezones(now)
        .join(" - ");
    Ok(ret)
}

fn command_convert(input: &str, now: DateTime<Utc>) -> Result<String, BotError> {
    let ret = Converter::try_from(input)?
        .convert_time_between_timezones(now)?
        .into_iter()
        .next()
        .unwrap_or_else(|| "No time to convert".to_string());
    Ok(ret)
}

fn convert_error(error: &BotError) -> String {
    format!("{error}\n\nPlease follow the pattern below\n\n{CONVERT_COMMAND_INFO}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, TimeZone};

    fn winter_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn summer_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_convert_time_brt_cet() {
        let result = command_convert("12:00 BRT CET", winter_now());
        assert_eq!(result.unwrap(), "12:00 BRT - 16:00 CET");

        let result = command_convert("12:00 BRT CET", summer_now());
        assert_eq!(result.unwrap(), "12:00 BRT - 17:00 CET");
    }

    #[test]
    fn test_convert_time_utc_brl() {
        let result = command_convert("12:00 UTC BRT", winter_now());
        assert_eq!(result.unwrap(), "12:00 UTC - 09:00 BRT");
    }

    #[test]
    fn test_convert_time_one_digit() {
        let result = command_convert("1:00 BRT CET", winter_now());
        assert_eq!(result.unwrap(), "01:00 BRT - 05:00 CET");
    }

    #[test]
    fn test_convert_time_minimal() {
        let result = command_convert("2 BRT CET", winter_now());
        assert_eq!(result.unwrap(), "02:00 BRT - 06:00 CET");
    }

    #[test]
    fn test_convert_time_multiple_spaces() {
        let result = command_convert("12:00    BRT     RO    ", winter_now());
        assert_eq!(result.unwrap(), "12:00 BRT - 17:00 Europe/Bucharest");
    }

    #[test]
    fn test_convert_time_missing_target_tz() {
        let result = command_convert("12:00 UTC", winter_now());
        assert_eq!(result.unwrap(), "12:00 UTC");
    }

    #[test]
    fn test_convert_time_nonexistent() {
        // CET skips 02:00-03:00 on 2026-03-29 (spring forward).
        let now = Utc.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).unwrap();
        let result = command_convert("2:30 CET BRT", now);
        assert_eq!(
            result,
            Err(BotError::NonexistentTime {
                time: NaiveTime::from_hms_opt(2, 30, 0).unwrap(),
                tz: Tz::CET,
            })
        );
    }

    #[test]
    fn test_process_command_start() {
        let result = process_input("/start", winter_now());
        assert_eq!(result, command_start());
    }

    #[test]
    fn test_process_command_now() {
        let result = process_input("/now utc", winter_now());
        assert_eq!(result, command_now("utc", winter_now()).unwrap());
    }

    #[test]
    fn test_process_command_now_multiple_spaces() {
        let result = process_input("/now   utc    ", winter_now());
        assert_eq!(result, command_now("utc", winter_now()).unwrap());
    }

    #[test]
    fn test_process_command_convert() {
        let result = process_input("/convert 12:00 UTC BRT", winter_now());
        assert_eq!(
            result,
            command_convert("12:00 UTC BRT", winter_now()).unwrap()
        );
    }

    #[test]
    fn test_process_command_with_h_convert() {
        let result = process_input("/convert 12h UTC BRT", winter_now());
        assert_eq!(
            result,
            command_convert("12:00 UTC BRT", winter_now()).unwrap()
        );
    }

    #[test]
    fn test_process_command_invalid() {
        let result = process_input("invalid", winter_now());
        assert_eq!(result, invalid_command());
    }
}
