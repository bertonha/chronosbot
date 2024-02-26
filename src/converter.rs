use chrono::NaiveTime;
use chrono_tz::Tz;
use itertools::Itertools;

use crate::time::{format_times, now_on_timezone, parse_time, parse_tz, time_with_timezone};

pub struct Converter {
    pub base_time: NaiveTime,
    pub timezones: Vec<Tz>,
}

impl Converter {
    pub fn new(base_time: NaiveTime, timezones: Vec<Tz>) -> Self {
        Self {
            base_time,
            timezones,
        }
    }

    pub fn convert_time_between_timezones(&self) -> impl Iterator<Item = String> + '_ {
        self.timezones
            .iter()
            .map(|tz| convert_datetime_to_timezones(&self.base_time, tz, &self.timezones))
    }
}

impl TryFrom<&str> for Converter {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let split_values: Vec<&str> = input.split_whitespace().collect();

        let (parsed_time, timezone_start_index) = if split_values.is_empty() {
            (None, 0)
        } else if split_values[0] == "now" {
            (None, 1)
        } else {
            match parse_time(split_values[0]) {
                Ok(time) => (Some(time), 1),
                Err(_) => (None, 0),
            }
        };

        let timezones: Vec<Tz> = split_values
            .into_iter()
            .skip(timezone_start_index)
            .unique()
            .map(parse_tz)
            .collect::<Result<Vec<_>, _>>()?;

        if timezones.is_empty() {
            return Err("No timezones provided".into());
        }

        let base_time = parsed_time.unwrap_or_else(|| now_on_timezone(&timezones[0]));
        Ok(Self::new(base_time, timezones))
    }
}

fn convert_datetime_to_timezones(src_time: &NaiveTime, src_tz: &Tz, timezones: &[Tz]) -> String {
    let src_time = time_with_timezone(src_time, src_tz);
    let mut times = vec![src_time];
    for dst_tz in timezones {
        if src_tz == dst_tz {
            continue;
        }
        times.push(src_time.with_timezone(dst_tz));
    }
    format_times(times)
}

#[cfg(test)]
mod tests {
    use chrono_tz::America::Sao_Paulo;
    use chrono_tz::{CET, EET};

    use super::*;

    #[test]
    fn test_try_from() {
        let converter = Converter::try_from("12:00 CET BRT EET").unwrap();
        assert_eq!(
            converter.base_time,
            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
        );
        assert_eq!(converter.timezones, vec![CET, Sao_Paulo, EET]);
    }

    #[test]
    fn test_convert_time() {
        let converter = Converter::new(
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            vec![CET, Sao_Paulo, EET],
        );
        let result = converter.convert_time_between_timezones();

        assert!(result.eq([
            "12:00 CET - 08:00 BRT - 13:00 EET",
            "12:00 BRT - 16:00 CET - 17:00 EET",
            "12:00 EET - 11:00 CET - 07:00 BRT",
        ]));
    }
}
