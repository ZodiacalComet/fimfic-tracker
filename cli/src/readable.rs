use std::fmt;

use chrono::{Local, TimeZone};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use number_prefix::NumberPrefix;

pub struct ReadableBytes(pub usize);

impl fmt::Display for ReadableBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match NumberPrefix::binary(self.0 as f64) {
            NumberPrefix::Standalone(bytes) => write!(f, "{:.0} bytes", bytes),
            NumberPrefix::Prefixed(prefix, n) => write!(f, "{:.2} {}B", n, prefix.symbol()),
        }
    }
}

pub struct ReadableDate(pub i64);

impl fmt::Display for ReadableDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date = Local.timestamp(self.0, 0);
        write!(
            f,
            "{} ({})",
            date.format("%F %X"),
            HumanTime::from(date).to_text_en(Accuracy::Rough, Tense::Past)
        )
    }
}
