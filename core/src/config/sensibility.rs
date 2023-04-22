use std::{cmp, fmt};

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

/// The different available options of update sensibility level.
///
/// Meant for the conditions of one level to be present in the ones above it, so for allowing a
/// less verbose evaluation it implements [`PartialOrd`].
/// ```
/// use fimfic_tracker::SensibilityLevel;
///
/// assert_eq!(SensibilityLevel::OnlyChapters, SensibilityLevel::OnlyChapters);
/// assert!(SensibilityLevel::Anything > SensibilityLevel::IncludeWords);
/// assert!(SensibilityLevel::IncludeWords < SensibilityLevel::Anything);
/// ```
///
/// Used for [`ConfigBuilder`](crate::ConfigBuilder) and [`Config`](crate::Config).
#[derive(Clone, Copy, Eq, Debug)]
pub enum SensibilityLevel {
    /// Only download an update if the amount of chapters is different than the one in the cached
    /// data.
    OnlyChapters,
    /// In addition to the `OnlyChapters` condition, takes into account the amount of words.
    IncludeWords,
    /// In addition to the `IncludeWords` conditions, considers the update date too.
    Anything,
}

impl PartialEq for SensibilityLevel {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl PartialOrd for SensibilityLevel {
    fn partial_cmp(&self, other: &SensibilityLevel) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &SensibilityLevel) -> bool {
        (*self as u8) < (*other as u8)
    }

    fn le(&self, other: &SensibilityLevel) -> bool {
        (*self as u8) <= (*other as u8)
    }

    fn gt(&self, other: &SensibilityLevel) -> bool {
        (*self as u8) > (*other as u8)
    }

    fn ge(&self, other: &SensibilityLevel) -> bool {
        (*self as u8) >= (*other as u8)
    }
}

impl Ord for SensibilityLevel {
    fn cmp(&self, other: &SensibilityLevel) -> cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

struct SensibilityVisitor;

impl<'de> Visitor<'de> for SensibilityVisitor {
    type Value = SensibilityLevel;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 2")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            0 => Ok(SensibilityLevel::OnlyChapters),
            1 => Ok(SensibilityLevel::IncludeWords),
            2 => Ok(SensibilityLevel::Anything),
            _ => Err(E::invalid_value(Unexpected::Signed(value), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for SensibilityLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(SensibilityVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comparison() {
        assert!(SensibilityLevel::OnlyChapters == SensibilityLevel::OnlyChapters);
        assert!(SensibilityLevel::OnlyChapters < SensibilityLevel::IncludeWords);
        assert!(SensibilityLevel::OnlyChapters < SensibilityLevel::Anything);

        assert!(SensibilityLevel::IncludeWords == SensibilityLevel::IncludeWords);
        assert!(SensibilityLevel::IncludeWords < SensibilityLevel::Anything);

        assert!(SensibilityLevel::Anything == SensibilityLevel::Anything);
    }
}
