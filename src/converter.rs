use chrono::{DateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use itertools::Itertools;

use crate::error::BotError;
use crate::time::{format_time_with_timezone, parse_time, parse_tz, time_with_timezone};

pub struct Converter {
    pub base_time: Option<NaiveTime>,
    pub timezones: Vec<Tz>,
}

impl Converter {
    pub fn new(base_time: Option<NaiveTime>, timezones: Vec<Tz>) -> Self {
        Self {
            base_time,
            timezones,
        }
    }

    pub fn try_from_only_timezones(src_text: &str) -> Result<Self, BotError> {
        let timezones = timezone_parser(src_text.split_whitespace())?;
        Ok(Self::new(None, timezones))
    }

    pub fn convert_time_between_timezones(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<String>, BotError> {
        self.timezones
            .iter()
            .map(|tz| self.convert_from_timezone(tz, now))
            .collect()
    }

    pub fn now_in_timezones(&self, now: DateTime<Utc>) -> impl Iterator<Item = String> + '_ {
        self.timezones
            .iter()
            .map(move |tz| format_time_with_timezone(&now.with_timezone(tz)))
    }

    fn convert_from_timezone(&self, src_tz: &Tz, now: DateTime<Utc>) -> Result<String, BotError> {
        let src_time = match &self.base_time {
            Some(time) => time_with_timezone(time, src_tz, now)?,
            None => now.with_timezone(src_tz),
        };
        let mut times = vec![src_time];
        for dst_tz in &self.timezones {
            if src_tz == dst_tz {
                continue;
            }
            times.push(src_time.with_timezone(dst_tz));
        }
        Ok(times.iter().map(format_time_with_timezone).join(" - "))
    }
}

impl TryFrom<&str> for Converter {
    type Error = BotError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let split_values: Vec<&str> = input.split_whitespace().collect();

        let (base_time, timezone_start_index) = if split_values.is_empty() {
            (None, 0)
        } else if split_values[0] == "now" {
            (None, 1)
        } else {
            match parse_time(split_values[0]) {
                Ok(time) => (Some(time), 1),
                Err(_) => (None, 0),
            }
        };

        let timezones = timezone_parser(split_values.into_iter().skip(timezone_start_index))?;
        Ok(Self::new(base_time, timezones))
    }
}

fn timezone_parser<'a>(input: impl Iterator<Item = &'a str>) -> Result<Vec<Tz>, BotError> {
    input.unique().map(parse_tz).collect()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono_tz::America::Sao_Paulo;
    use chrono_tz::{CET, EET};

    use super::*;

    fn winter_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn summer_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_try_from() {
        let converter = Converter::try_from("12:00 CET BRT EET").unwrap();
        assert_eq!(converter.base_time, NaiveTime::from_hms_opt(12, 0, 0));
        assert_eq!(converter.timezones, vec![CET, Sao_Paulo, EET]);
    }

    #[test]
    fn test_convert_time_winter() {
        let converter =
            Converter::new(NaiveTime::from_hms_opt(12, 0, 0), vec![CET, Sao_Paulo, EET]);
        let result = converter
            .convert_time_between_timezones(winter_now())
            .unwrap();

        assert_eq!(
            result,
            vec![
                "12:00 CET - 08:00 BRT - 13:00 EET",
                "12:00 BRT - 16:00 CET - 17:00 EET",
                "12:00 EET - 11:00 CET - 07:00 BRT",
            ]
        );
    }

    #[test]
    fn test_convert_time_summer() {
        let converter =
            Converter::new(NaiveTime::from_hms_opt(12, 0, 0), vec![CET, Sao_Paulo, EET]);
        let result = converter
            .convert_time_between_timezones(summer_now())
            .unwrap();

        assert_eq!(
            result,
            vec![
                "12:00 CET - 07:00 BRT - 13:00 EET",
                "12:00 BRT - 17:00 CET - 18:00 EET",
                "12:00 EET - 11:00 CET - 06:00 BRT",
            ]
        );
    }

    #[test]
    fn test_now_in_timezones() {
        let converter = Converter::new(None, vec![Tz::UTC, Sao_Paulo]);
        let result: Vec<String> = converter.now_in_timezones(winter_now()).collect();
        assert_eq!(result, vec!["12:00 UTC", "09:00 BRT"]);
    }
}
