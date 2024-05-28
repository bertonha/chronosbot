use chrono::{TimeDelta, Utc};
use chrono_tz::{OffsetComponents, Tz};

pub fn is_dst(tz: Tz) -> bool {
    let now = Utc::now().with_timezone(&tz);
    now.offset().dst_offset() != TimeDelta::try_seconds(0).unwrap()
}
