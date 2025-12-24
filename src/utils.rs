use chrono::{TimeDelta, Utc};
use chrono_tz::{OffsetComponents, Tz};

pub fn is_dst(tz: Tz) -> bool {
    let now = Utc::now().with_timezone(&tz);
    now.offset().dst_offset() != TimeDelta::try_seconds(0).unwrap()
}

#[cfg(test)]
pub fn expected_hour(tz: Tz, hour_dst: &str, hour_std: &str) -> String {
    if is_dst(tz) {
        hour_dst.into()
    } else {
        hour_std.into()
    }
}
