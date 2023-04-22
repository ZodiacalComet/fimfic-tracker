use std::fmt;

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

/// The different ratings a [`Story`](crate::story::Story) can have.
///
/// Implements [`Display`](fmt::Display) for `String` representations of each variant:
/// ```
/// # use fimfic_tracker::api::StoryRating;
/// assert_eq!(StoryRating::Everyone.to_string(), "Everyone");
/// assert_eq!(StoryRating::Teen.to_string(), "Teen");
/// assert_eq!(StoryRating::Mature.to_string(), "Mature");
/// ```
#[derive(Clone, Copy, Debug)]
pub enum StoryRating {
    /// A story rated as for `Everyone`.
    Everyone,
    /// A story rated as `Teen`.
    Teen,
    /// A story rated as `Mature`.
    Mature,
}

impl PartialEq for StoryRating {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl fmt::Display for StoryRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoryRating::Everyone => write!(f, "Everyone"),
            StoryRating::Teen => write!(f, "Teen"),
            StoryRating::Mature => write!(f, "Mature"),
        }
    }
}

struct RatingVisitor;

impl<'de> Visitor<'de> for RatingVisitor {
    type Value = StoryRating;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 3")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            0 => Ok(StoryRating::Everyone),
            1 => Ok(StoryRating::Teen),
            2 => Ok(StoryRating::Mature),
            _ => Err(E::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for StoryRating {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RatingVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors;

    use serde_json::json;

    #[derive(Deserialize, Debug)]
    struct Test {
        content_rating: StoryRating,
    }

    macro_rules! assert_deserialize {
        ($value:expr => $variant:ident) => {
            let json = json!({ "content_rating": $value });
            let test: Test =
                serde_json::from_value(json).expect("couldn't deserialize StoryRating");
            assert_eq!(test.content_rating, StoryRating::$variant);
        };
    }

    #[test]
    fn test_deserialize() -> errors::Result<()> {
        assert_deserialize!(0 => Everyone);
        assert_deserialize!(1 => Teen);
        assert_deserialize!(2 => Mature);

        Ok(())
    }
}
