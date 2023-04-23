//! Story storage data (de)serialization.
use chrono::{offset::Utc, DateTime};
use fimfiction_api::StoryStatus;
use serde::{Deserialize, Serialize};

use crate::errors::{self, TrackerError};
use crate::StoryResponse;

/// Alias for a [`Story`] ID.
pub type Id = u32;

/// Story data used for track data storage.
///
/// Meant to be constructed from a deserialized [`StoryResponse`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Story {
    /// Unique story ID.
    pub id: Id,
    /// Story title.
    pub title: String,
    /// Username of the author.
    pub author: String,
    /// The amount of chapters the story has.
    #[serde(rename = "chapter-amt")]
    pub chapter_count: u64,
    /// The amount of words the story has.
    pub words: u64,
    /// Datetime of the last update.
    #[serde(rename = "last-update-timestamp", with = "chrono::serde::ts_seconds")]
    pub update_datetime: DateTime<Utc>,
    /// Story completion status.
    #[serde(rename = "completion-status")]
    pub status: StoryStatus,
}

impl From<StoryResponse> for Story {
    fn from(response: StoryResponse) -> Self {
        Story {
            id: response.id,
            title: response.title,
            author: response.author.name,
            chapter_count: response.chapter_count,
            words: response.words,
            update_datetime: response.date_modified,
            status: response.status,
        }
    }
}

/// Kind of update present in a comparison between two [`Story`] structs.
///
/// Meant to be used as a result of [`Story::compare_to()`].
#[derive(Debug)]
pub enum StoryUpdate {
    /// Story had a chapter update.
    Chapters {
        /// Amount of chapters before the update.
        before: u64,
        /// Amount of chapters after the update.
        after: u64,
    },
    /// Story had a words update.
    Words {
        /// Amount of words before the update.
        before: u64,
        /// Amount of words after the update.
        after: u64,
    },
    /// Story had an update.
    DateTime {
        /// The timestamp before the update.
        before: DateTime<Utc>,
        /// The timestamp after the update.
        after: DateTime<Utc>,
    },
}

impl Story {
    /// Gets the Fimfiction URL to the story.
    pub fn url(&self) -> String {
        format!("https://www.fimfiction.net/story/{}", self.id)
    }

    /// Checks for the existence of an update from the comparison with a more recent version of
    /// [`Story`].
    ///
    /// It is done by comparing fields, in the following order:
    /// 1. `chapter_count`, considered an update if both fields are different from each other. It
    ///    is the most meaningful and visible update, so it has priority.
    /// 2. `words`, considered an update if both fields aren't the same.
    /// 3. `update_datetime`, considered an update if `updated_story`'s timestamp is more recent.
    ///    It is the least noticeable so it comes last.
    ///
    /// # Error
    ///
    /// If the ID of `updated_story` isn't the same as of `self`.
    pub fn compare_to(&self, updated_story: &Story) -> errors::Result<Option<StoryUpdate>> {
        if self.id != updated_story.id {
            Err(TrackerError::story_comparison(self.id, updated_story.id))
        } else if self.chapter_count != updated_story.chapter_count {
            Ok(Some(StoryUpdate::Chapters {
                before: self.chapter_count,
                after: updated_story.chapter_count,
            }))
        } else if self.words != updated_story.words {
            Ok(Some(StoryUpdate::Words {
                before: self.words,
                after: updated_story.words,
            }))
        } else if self.update_datetime < updated_story.update_datetime {
            Ok(Some(StoryUpdate::DateTime {
                before: self.update_datetime,
                after: updated_story.update_datetime,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::TimeZone;
    use serde_json::json;

    use errors::ErrorKind;

    macro_rules! datetime {
        ($yy:expr, $mm:expr, $dd:expr, $h:expr, $m:expr, $s:expr) => {
            Utc.with_ymd_and_hms($yy, $mm, $dd, $h, $m, $s)
                .single()
                .unwrap()
        };
        ($timestamp:expr) => {
            Utc.timestamp_opt($timestamp, 0).single().unwrap()
        };
    }

    fn get_story(
        chapters: Option<u64>,
        words: Option<u64>,
        datetime: Option<DateTime<Utc>>,
    ) -> Story {
        Story {
            id: 100001,
            title: "An Active Story".into(),
            author: "A New Author".into(),
            chapter_count: chapters.unwrap_or(5),
            words: words.unwrap_or(12050),
            update_datetime: datetime.unwrap_or_else(|| datetime!(2021, 1, 19, 23, 0, 0)),
            status: StoryStatus::Incomplete,
        }
    }

    macro_rules! story {
        () => {
            get_story(None, None, None)
        };
        (chapter_count = $value:expr) => {
            get_story(Some($value), None, None)
        };
        (words = $value:expr) => {
            get_story(None, Some($value), None)
        };
        (datetime = $value:expr) => {
            get_story(None, None, Some($value))
        };
    }

    macro_rules! assert_update {
        ([$variant:ident $attr:ident]: $before:expr, $after:expr) => {
            match $before.compare_to(&$after) {
                Ok(Some(StoryUpdate::$variant { before, after })) => {
                    assert_eq!(before, $before.$attr);
                    assert_eq!(after, $after.$attr);
                }
                _ => unreachable!(),
            }
        };
    }

    macro_rules! assert_no_difference {
        ($before:expr, $after:expr) => {
            match $before.compare_to(&$after) {
                Ok(None) => {}
                _ => unreachable!(),
            };
        };
    }

    #[test]
    fn deserialize_and_serialize() {
        let story_json = json!({
            "id": 100000,
            "title": "A Story Title",
            "author": "An Author",
            "chapter-amt": 2,
            "words": 10000,
            "last-update-timestamp": 1607137200,
            "completion-status": 0
        })
        .to_string();

        let story: Story =
            serde_json::from_str(&story_json).expect("couldn't deserialize json into Story");
        assert_eq!(story.id, 100000);
        assert_eq!(&story.title, "A Story Title");
        assert_eq!(&story.author, "An Author");
        assert_eq!(story.chapter_count, 2);
        assert_eq!(story.words, 10000);
        assert_eq!(story.update_datetime, datetime!(1607137200));
        assert_eq!(story.status, StoryStatus::Complete);
        assert_eq!(story.url(), "https://www.fimfiction.net/story/100000");

        let json = serde_json::to_string(&story).expect("couldn't serialize Story into json");
        assert_eq!(json, story_json);
    }

    #[test]
    fn update_comparison() {
        let story = story!();

        assert_update!([Chapters chapter_count]: story, story!(chapter_count = 2));
        assert_update!([Chapters chapter_count]: story, story!(chapter_count = 9));
        assert_update!([Words words]: story, story!(words = 9506));
        assert_update!([Words words]: story, story!(words = 15042));
        assert_update!([DateTime update_datetime]: story, story!(datetime = datetime!(2021, 2, 14, 23, 0, 0)));
        assert_no_difference!(story, story);
        assert_no_difference!(story, story!(datetime = datetime!(2021, 1, 10, 12, 0, 0)));

        let another_story = Story {
            id: 100002,
            title: "Not 'An Active Story'".into(),
            author: "Another Author".into(),
            chapter_count: 12,
            words: 14012,
            update_datetime: datetime!(2021, 2, 28, 23, 0, 0),
            status: StoryStatus::Incomplete,
        };

        match story.compare_to(&another_story).unwrap_err().kind {
            ErrorKind::BadStoryComparison { id, other_id } => {
                assert_eq!(id, 100001);
                assert_eq!(other_id, 100002);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn update_comparison_order() {
        let story = story!();
        let datetime = datetime!(2021, 2, 14, 23, 0, 0);

        let update = get_story(Some(9), Some(15042), Some(datetime));
        assert_update!([Chapters chapter_count]: story, update);

        let update = get_story(Some(9), Some(15042), None);
        assert_update!([Chapters chapter_count]: story, update);

        let update = get_story(Some(9), None, Some(datetime));
        assert_update!([Chapters chapter_count]: story, update);

        let update = get_story(None, Some(15042), Some(datetime));
        assert_update!([Words words]: story, update);

        let update = get_story(Some(9), None, None);
        assert_update!([Chapters chapter_count]: story, update);

        let update = get_story(None, Some(15042), None);
        assert_update!([Words words]: story, update);

        let update = get_story(None, None, Some(datetime));
        assert_update!([DateTime update_datetime]: story, update);
    }
}
