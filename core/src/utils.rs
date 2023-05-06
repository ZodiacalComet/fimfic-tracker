//! Collection of utility functions, structs and traits.
use std::{
    borrow::Cow,
    fs, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use indexmap::IndexMap;
use shellexpand::env_with_context_no_errors;

use url::Url;

use crate::config::{Config, DownloadFormat};
use crate::errors::{self, Action, ErrorKind, TrackerError};
use crate::story::{Id, Story};

#[cfg(any(target_os = "windows", target_os = "macos"))]
static APPLICATION_NAME: &str = "Fimfiction Tracker";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
static APPLICATION_NAME: &str = "fimfic-tracker";

lazy_static! {
    pub(crate) static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "", APPLICATION_NAME)
        .expect("fimfic-tracker project directories should be retrievable");
}

// TODO: Is `Local` the correct directory for the configuration and tracker file on Windows?

/// Path to the default location of the user's `config.toml` file.
///
/// The directory corresponds to [`ProjectDirs::config_local_dir()`].
pub fn default_user_config_file() -> PathBuf {
    PROJECT_DIRS.config_local_dir().join("config.toml")
}

/// Path to the default location of the user's `track-data.json` file.
///
/// The directory corresponds to [`ProjectDirs::data_local_dir()`].
pub fn default_user_tracker_file() -> PathBuf {
    PROJECT_DIRS.data_local_dir().join("track-data.json")
}

/// Creates a Fimfiction story download [`Url`] to the [`Story`] in the given
/// [`format`](DownloadFormat).
pub fn download_url_format(story: &Story, format: DownloadFormat) -> Url {
    Url::parse("https://www.fimfiction.net/story/download/")
        .and_then(|u| u.join(&format!("{}/", story.id)))
        .and_then(|u| u.join(format.to_string().as_ref()))
        .expect("Fimficiton download URL parse failed")
}

/// Performs a shell-like environment expansion with [`shellexpand::env_with_context()`] using a
/// custom context.
///
/// Said context has the defined variables:
/// - `ID`: The value of `story.id`.
/// - `TITLE`: The value of `story.title`, safe to use as a filename.
/// - `AUTHOR`: The value of `story.author`, safe to use as a filename.
/// - `CHAPTERS`: The value of `story.chapter_count`.
/// - `WORDS`: The value of `story.words`.
/// - `UPDATE_TIMESTAMP`: The value of `story.update_datetime.timestamp()`.
/// - `URL`: The value of `story.url()`.
/// - `DOWNLOAD_URL`: Story download URL, in the form of
///   `"https://www.fimfiction.net/story/download/{ID}/{FORMAT}"`
/// - `DOWNLOAD_DIR`: The value of `config.download_dir`.
/// - `FORMAT`: The value of `config.download_format`.
///
/// Unexpected variables are left as is.
pub fn env_with_command_context<'a>(
    command: &'a str,
    story: &Story,
    config: &Config,
) -> Cow<'a, str> {
    env_with_context_no_errors(command, |var| -> Option<String> {
        match var {
            "ID" => Some(story.id.to_string()),
            "TITLE" => Some(sanitize_filename(story.title.clone())),
            "AUTHOR" => Some(sanitize_filename(story.author.clone())),
            "CHAPTERS" => Some(story.chapter_count.to_string()),
            "WORDS" => Some(story.words.to_string()),
            "UPDATE_TIMESTAMP" => Some(story.update_datetime.timestamp().to_string()),
            "URL" => Some(story.url()),
            "DOWNLOAD_URL" => Some(download_url_format(story, config.download_format).to_string()),
            "DOWNLOAD_DIR" => Some(config.download_dir.display().to_string()),
            "FORMAT" => Some(config.download_format.to_string()),
            _ => None,
        }
    })
}

/// Replaces forbidden characters present in `filename` with `_`.
///
/// The forbidden characters defined are `>`, `<`, `:`, `"`, `?`, `*`, `/` and `\`.
pub fn sanitize_filename<T>(filename: T) -> String
where
    T: AsRef<str>,
{
    filename
        .as_ref()
        .chars()
        .map(|c| match c {
            '>' | '<' | ':' | '"' | '?' | '*' | '/' | '\\' => '_',
            _ => c,
        })
        .collect::<String>()
}

pub async fn async_read_to_string<P>(path: P) -> errors::Result<String>
where
    P: AsRef<Path>,
{
    tokio::fs::read_to_string(&path).await.map_err(|err| {
        TrackerError::io(err).context(format!("failed to read file `{}`", path.as_ref().display()))
    })
}

pub async fn async_write<P, C>(path: P, contents: C) -> errors::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    tokio::fs::write(&path, contents).await.map_err(|err| {
        TrackerError::io(err).context(format!(
            "failed to write into file `{}`",
            path.as_ref().display()
        ))
    })
}

pub fn read_to_string<P>(path: P) -> errors::Result<String>
where
    P: AsRef<Path>,
{
    fs::read_to_string(&path).map_err(|err| {
        TrackerError::io(err).context(format!("failed to read file `{}`", path.as_ref().display()))
    })
}

pub fn write<P, C>(path: P, contents: C) -> errors::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    fs::write(&path, contents).map_err(|err| {
        TrackerError::io(err).context(format!(
            "failed to write into file `{}`",
            path.as_ref().display()
        ))
    })
}

/// Struct to handle the loading and saving of the track data file.
#[derive(Debug)]
pub struct StoryData {
    path: String,
    data: IndexMap<Id, Story>,
}

impl Deref for StoryData {
    type Target = IndexMap<Id, Story>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for StoryData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl StoryData {
    /// Constructs a new [`StoryData`].
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        StoryData {
            path: path.as_ref().to_string_lossy().into(),
            data: IndexMap::new(),
        }
    }

    fn load_data_from_string(&mut self, content: String) -> errors::Result<()> {
        let mut stories: Vec<Story> = serde_json::from_str(&content).map_err(|err| {
            TrackerError::tracker_format(self.path.clone(), err, Action::Deserializing)
        })?;
        self.data = stories
            .drain(..)
            .map(|story| (story.id, story))
            .collect::<IndexMap<Id, Story>>();

        Ok(())
    }

    fn data_to_string(&self) -> errors::Result<String> {
        let stories = self.data.values().collect::<Vec<&Story>>();
        serde_json::to_string(&stories)
            .map_err(|err| TrackerError::tracker_format(None, err, Action::Serializing))
    }

    /// If the track data file exists maps its contents into the cached data, completely
    /// overwriting it. Otherwise, nothing is changed.
    ///
    /// # Errors
    ///
    /// - If [`std::fs::read_to_string()`] returns a no [`NotFound`](io::ErrorKind::NotFound)
    /// error.
    /// - On deserialization errors with the contents of the track data file.
    pub fn load(&mut self) -> errors::Result<()> {
        match read_to_string(&self.path) {
            Ok(content) => self.load_data_from_string(content),
            Err(TrackerError {
                kind: ErrorKind::Io(err),
                ..
            }) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Asynchronous version of [`StoryData::load()`].
    pub async fn async_load(&mut self) -> errors::Result<()> {
        match async_read_to_string(&self.path).await {
            Ok(content) => self.load_data_from_string(content),
            Err(TrackerError {
                kind: ErrorKind::Io(err),
                ..
            }) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Takes the cached track data and writes it into the track data file.
    pub fn save(&self) -> errors::Result<()> {
        let data = self.data_to_string()?;
        write(&self.path, data)?;
        Ok(())
    }

    /// Asynchronous version of [`StoryData::save()`].
    pub async fn async_save(&self) -> errors::Result<()> {
        let data = self.data_to_string()?;
        async_write(&self.path, data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::Utc;

    #[test]
    fn download_url_builder() {
        use crate::{config::DownloadFormat, StoryStatus};

        let story = Story {
            id: 165,
            title: "A Title".into(),
            author: "An Author".into(),
            chapter_count: 5,
            words: 15017,
            update_datetime: Utc::now(),
            status: StoryStatus::Complete,
        };

        macro_rules! assert_formats {
            ($($kind: ident),+) => {
                $(
                    assert_eq!(
                        format!(
                            "https://www.fimfiction.net/story/download/{}/{}",
                            story.id, DownloadFormat::$kind
                        ),
                        download_url_format(&story, DownloadFormat::$kind).as_str()
                    );
                )+
            }
        }

        assert_formats!(HTML, EPUB, TXT);
    }
}
