//! Deserialization of the Fimfiction story API.
use std::convert::TryFrom;

use serde::Deserialize;

mod rating;
mod status;
mod vote;

use crate::errors::TrackerError;
use crate::story::Story;
pub use rating::StoryRating;
pub use status::StoryStatus;
use vote::deserialize_vote;

/// Container struct of the author response given by the Fimfiction story API.
#[derive(Deserialize, Debug)]
pub struct AuthorResponse {
    /// Author's ID.
    pub id: u32,
    /// Username of the author.
    pub name: String,
}

/// Container struct for all chapter response data given by the Fimfiction story API.
#[derive(Deserialize, Debug)]
pub struct ChapterResponse {
    /// Chapter's ID.
    pub id: u32,
    /// Title of the chapter.
    pub title: String,
    /// The amount of words the chapter has.
    pub words: u64,
    /// The amount of views the chapter has.
    pub views: u32,
    /// Fimfiction URL to the story's chapter.
    pub link: String,
    /// Last chapter update timestamp.
    pub date_modified: i64,
}

/// Container struct for all relevant story response data given by the Fimfiction story API.
#[derive(Deserialize, Debug)]
pub struct StoryResponse {
    /// Unique story ID.
    pub id: u32,
    /// Title of the story.
    pub title: String,
    /// Fimfiction URL to the story.
    pub url: String,
    /// Summary of the story. Showed on story cards present in the main page, groups and sidebars
    /// story listing.
    pub short_description: String,
    /// Complete story description, showed on the main story page.
    pub description: String,
    /// Last story update timestamp.
    pub date_modified: i64,
    /// Story cover image in thumbnail size if any.
    pub image: Option<String>,
    /// Story cover image in full size if any.
    pub full_image: Option<String>,
    /// The views the story has.
    pub views: u32,
    /// The total views the story has.
    pub total_views: u32,
    /// The amount of words the story has.
    pub words: u64,
    /// The amount of chapters the story has.
    pub chapter_count: u64,
    /// The amount of comments the story has.
    pub comments: u32,
    /// Author of the story.
    pub author: AuthorResponse,
    /// Story completion status.
    pub status: StoryStatus,
    /// Rating given to the story.
    pub content_rating: StoryRating,
    /// The amount of likes the story has, if not disabled.
    #[serde(deserialize_with = "deserialize_vote")]
    pub likes: Option<u32>,
    /// The amount of dislikes the story has, if not disabled.
    #[serde(deserialize_with = "deserialize_vote")]
    pub dislikes: Option<u32>,
    /// Chapters of the story.
    pub chapters: Vec<ChapterResponse>,
}

impl From<StoryResponse> for Story {
    fn from(response: StoryResponse) -> Self {
        Story {
            id: response.id,
            title: response.title,
            author: response.author.name,
            chapter_count: response.chapter_count,
            words: response.words,
            timestamp: response.date_modified,
            status: response.status,
        }
    }
}

/// Meant to be used for deserialization of the response given by the Fimfiction story API.
#[derive(Deserialize, Debug)]
pub struct FimfictionResponse {
    /// Story data.
    pub story: StoryResponse,
}

impl From<FimfictionResponse> for Story {
    fn from(response: FimfictionResponse) -> Self {
        response.story.into()
    }
}

impl TryFrom<String> for FimfictionResponse {
    type Error = TrackerError;
    fn try_from(content: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&content)
            .map_err(|err| TrackerError::unexpected_response(err, content))
    }
}
