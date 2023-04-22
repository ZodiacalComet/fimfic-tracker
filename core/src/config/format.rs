use std::fmt;

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

/// Represents the available story formats that Fimfiction provides.
///
/// Implements [`Display`](fmt::Display) for `String` represetations of each variant:
/// ```
/// # use fimfic_tracker::DownloadFormat;
/// assert_eq!(DownloadFormat::HTML.to_string(), "html");
/// assert_eq!(DownloadFormat::EPUB.to_string(), "epub");
/// assert_eq!(DownloadFormat::TXT.to_string(), "txt");
/// ```
///
/// Used for [`ConfigBuilder`](crate::ConfigBuilder) and [`Config`](crate::Config).
#[derive(Clone, Copy, Debug)]
pub enum DownloadFormat {
    /// Story in HTML format.
    HTML,
    /// Story in EPUB format.
    EPUB,
    /// Story in plain text format.
    TXT,
}

impl PartialEq for DownloadFormat {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl fmt::Display for DownloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadFormat::HTML => write!(f, "html"),
            DownloadFormat::EPUB => write!(f, "epub"),
            DownloadFormat::TXT => write!(f, "txt"),
        }
    }
}

struct FormatVisitor;

impl<'de> Visitor<'de> for FormatVisitor {
    type Value = DownloadFormat;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str(r#"one of the following valid download formats: "html", "epub" or "txt""#)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "html" => Ok(DownloadFormat::HTML),
            "epub" => Ok(DownloadFormat::EPUB),
            "txt" => Ok(DownloadFormat::TXT),
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

impl<'de> Deserialize<'de> for DownloadFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FormatVisitor)
    }
}
