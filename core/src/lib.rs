//! Provides all of the non interface specific code used for `fimfic-tracker`.
//!
//! Out of the box, it allows for access to configuration values and track data file manipulation.
//! ```no_run
//! # use fimfic_tracker::Result;
//! # fn main() -> Result<()> {
//! use fimfic_tracker::{Config, ConfigBuilder, StoryData};
//! # use std::path::Path;
//!
//! // An specific tracker file.
//! let mut story_data = StoryData::new(Path::new("tracker-file.json"));
//!
//! // The tracker file as specified in the default config.
//! let config: Config = ConfigBuilder::from_default_sources()?.into();
//! let mut story_data = StoryData::new(&config.tracker_file);
//!
//! // Loads the content of the file.
//! story_data.load()?;
//!
//! // Listing tracked stories.
//! for story in story_data.values() {
//!     println!("{:?}", story);
//! }
//!
//! // Removing last story.
//! if let Some((id, story)) = story_data.pop() {
//!     println!("Removed {} from track data: {:?}", id, story);
//! }
//!
//! // Saving the modifications made into the tracker file.
//! story_data.save()?;
//! # Ok(())
//! # }
//! ```
//!
//! While is possible to manually construct a [`Story`] struct, is recommended to create it from
//! either a deserialized [`FimfictionResponse`] or [`StoryResponse`](api::StoryResponse).
//!
//! # Optional feature
//!
//! The `downloader` enables structs to easily create [`StoryResponse`](api::StoryResponse)s and
//! execute downloads for stories for either synchronous or asynchronous contexts.
#![warn(missing_docs)]

mod config;
mod errors;

pub mod api;
#[cfg(feature = "downloader")]
pub mod downloader;
pub mod story;
mod utils;

#[doc(inline)]
pub use api::{FimfictionResponse, StoryStatus};
pub use config::{
    Config, ConfigBuilder, DownloadFormat, SensibilityLevel, DEFAULT_ENVIRONMENT_PREFIX,
};
pub use errors::{ErrorKind, Result, TrackerError};
#[doc(inline)]
pub use story::{Story, StoryUpdate};
#[doc(inline)]
pub use utils::{
    default_user_config_file, download_url_format, env_with_command_context, StoryData,
};
