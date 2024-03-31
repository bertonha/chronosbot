use chrono::NaiveTime;
use chrono_tz::{ParseError, Tz};
use itertools::Itertools;

use crate::time::{
    format_time_with_timezone, now_on_timezone, parse_time, parse_tz, time_with_timezone,
};

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

    pub fn try_from_only_timezones(src_text: &str) -> Result<Self, ParseError> {
        let timezones = timezone_parser(src_text.split_whitespace())?;
        Ok(Self::new(None, timezones))
    }

    pub fn convert_time_between_timezones(&self) -> impl Iterator<Item = String> + '_ {
        self.timezones
            .iter()
            .map(|tz| convert_datetime_to_timezones(&self.base_time, tz, &self.timezones))
    }

    pub fn now_in_timezones(&self) -> impl Iterator<Item = String> + '_ {
        self.timezones
            .iter()
            .map(|tz| format_time_with_timezone(now_on_timezone(tz)))
    }
}

impl TryFrom<&str> for Converter {
    type Error = Box<dyn std::error::Error>;

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

fn convert_datetime_to_timezones(
    src_time: &Option<NaiveTime>,
    src_tz: &Tz,
    timezones: &[Tz],
) -> String {
    let src_time = match src_time {
        Some(time) => time_with_timezone(time, src_tz),
        None => now_on_timezone(src_tz),
    };
    let mut times = vec![src_time];
    for dst_tz in timezones {
        if src_tz == dst_tz {
            continue;
        }
        times.push(src_time.with_timezone(dst_tz));
    }
    times.into_iter().map(format_time_with_timezone).join(" - ")
}

fn timezone_parser<'a>(input: impl Iterator<Item = &'a str>) -> Result<Vec<Tz>, ParseError> {
    input.unique().map(parse_tz).collect()
}

#[cfg(test)]
mod tests {
    use crate::utils::is_dst;
    use chrono_tz::America::Sao_Paulo;
    use chrono_tz::{CET, EET};

    use super::*;

    #[test]
    fn test_try_from() {
        let converter = Converter::try_from("12:00 CET BRT EET").unwrap();
        assert_eq!(converter.base_time, NaiveTime::from_hms_opt(12, 0, 0));
        assert_eq!(converter.timezones, vec![CET, Sao_Paulo, EET]);
    }

    #[test]
    fn test_convert_time() {
        let converter =
            Converter::new(NaiveTime::from_hms_opt(12, 0, 0), vec![CET, Sao_Paulo, EET]);
        let result = converter.convert_time_between_timezones();

        let expected_result = if is_dst(CET) {
            [
                "12:00 CET - 07:00 BRT - 13:00 EET",
                "12:00 BRT - 17:00 CET - 18:00 EET",
                "12:00 EET - 11:00 CET - 06:00 BRT",
            ]
        } else {
            [
                "12:00 CET - 08:00 BRT - 13:00 EET",
                "12:00 BRT - 16:00 CET - 17:00 EET",
                "12:00 EET - 11:00 CET - 07:00 BRT",
            ]
        };
        assert!(result.eq(expected_result));
    }
}
