use chrono::DateTime;
use chrono_tz::America::Sao_Paulo;
use chrono_tz::Europe::Bucharest;
use chrono_tz::{Tz, CET, UTC};

pub fn parse_tz(text: &str) -> Option<Tz> {
    match text.parse() {
        Ok(tz) => Some(tz),
        Err(_) => match text.to_lowercase().as_str() {
            "utc" => Some(UTC),
            "cet" | "europe" => Some(CET),
            "brazil" | "brasil" | "brt" => Some(Sao_Paulo),
            "romania" => Some(Bucharest),
            _ => None,
        },
    }
}

pub fn format_time(time: DateTime<Tz>) -> String {
    time.format("%H:%M:%S").to_string()
}
