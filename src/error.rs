use chrono::NaiveTime;
use chrono_tz::{ParseError, Tz};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum BotError {
    #[error("Invalid timezone: {0}")]
    InvalidTimezone(#[from] ParseError),
    #[error("Invalid time: {0}")]
    InvalidTime(String),
    #[error("{time} does not exist in {tz} on that date (daylight saving transition)")]
    NonexistentTime { time: NaiveTime, tz: Tz },
}
