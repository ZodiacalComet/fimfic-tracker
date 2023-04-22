use std::fmt;

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The different completion statuses a [`Story`](crate::story::Story) can have.
///
/// Implements [`Display`](fmt::Display) for `String` representations of each variant:
/// ```
/// # use fimfic_tracker::StoryStatus;
/// assert_eq!(StoryStatus::Complete.to_string(), "Complete");
/// assert_eq!(StoryStatus::Incomplete.to_string(), "Incomplete");
/// assert_eq!(StoryStatus::Hiatus.to_string(), "On Hiatus");
/// assert_eq!(StoryStatus::Cancelled.to_string(), "Cancelled");
/// ```
#[derive(Clone, Copy, Debug)]
pub enum StoryStatus {
    /// A story marked as `Completed`.
    Complete,
    /// A story marked as `Incomplete`.
    Incomplete,
    /// A story marked as `On Hiatus`.
    Hiatus,
    /// A story marked as `Cancelled`.
    Cancelled,
}

impl PartialEq for StoryStatus {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl fmt::Display for StoryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoryStatus::Complete => write!(f, "Complete"),
            StoryStatus::Incomplete => write!(f, "Incomplete"),
            StoryStatus::Hiatus => write!(f, "On Hiatus"),
            StoryStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl Serialize for StoryStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

struct StatusVisitor;

impl<'de> Visitor<'de> for StatusVisitor {
    type Value = StoryStatus;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid status string or an integer between 0 and 3")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            0 => Ok(StoryStatus::Complete),
            1 => Ok(StoryStatus::Incomplete),
            2 => Ok(StoryStatus::Hiatus),
            3 => Ok(StoryStatus::Cancelled),
            _ => Err(E::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "Complete" => Ok(StoryStatus::Complete),
            "Incomplete" => Ok(StoryStatus::Incomplete),
            "On Hiatus" => Ok(StoryStatus::Hiatus),
            "Cancelled" => Ok(StoryStatus::Cancelled),
            _ => Err(E::invalid_value(Unexpected::Str(value), &self)),
        }
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(value)
    }
}

impl<'de> Deserialize<'de> for StoryStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StatusVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors;

    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug)]
    struct Test {
        status: StoryStatus,
    }

    macro_rules! assert_deserialize {
        ([$($value:expr),+] => $variant:ident) => {
            $(
                let json = json!({ "status": $value });
                let test: Test = serde_json::from_value(json).expect("couldn't deserialize StoryStatus");
                assert_eq!(test.status, StoryStatus::$variant);
            )+
        }
    }

    macro_rules! assert_serialize {
        ($variant:ident => $value:expr) => {
            let test = Test {
                status: StoryStatus::$variant,
            };
            let json = serde_json::to_string(&test).expect("couldn't serialize StoryStatus");
            let expect = json!({ "status": $value }).to_string();
            assert_eq!(json, expect);
        };
    }

    #[test]
    fn test_deserialize() -> errors::Result<()> {
        assert_deserialize!([0, "Complete"] => Complete);
        assert_deserialize!([1, "Incomplete"] => Incomplete);
        assert_deserialize!([2, "On Hiatus"] => Hiatus);
        assert_deserialize!([3, "Cancelled"] => Cancelled);

        Ok(())
    }

    #[test]
    fn test_serialize() -> errors::Result<()> {
        assert_serialize!(Complete => 0);
        assert_serialize!(Incomplete => 1);
        assert_serialize!(Hiatus => 2);
        assert_serialize!(Cancelled => 3);

        Ok(())
    }
}
