//! Definitions for the [`TrackerError`] type.
use std::{error::Error, fmt, io};

#[doc(inline)]
pub use fimfiction_api::StoryError;

use crate::story::Id;

/// An alias of [`Result`] for all of its instances across the crate.
pub type Result<T> = std::result::Result<T, TrackerError>;

/// Representation of a configuration error by their source.
#[derive(Debug)]
pub enum ConfigSource {
    /// Error caused by a file.
    File {
        /// The configuration file that caused the error.
        path: String,
        /// The error being thrown.
        error: toml::de::Error,
    },
    /// Error caused by the environment.
    Env(envy::Error),
}

/// Meant for determining the action that the [`TrackerFormat`](ErrorKind::TrackerFormat) was in
/// the middle of without requiring any additional context.
#[derive(Debug)]
pub enum Action {
    /// Indicates that a serialization of the tracker data was being done.
    Serializing,
    /// Indicates that a deserialization of the tracker data was being done.
    Deserializing,
}

/// The different kinds of errors that [`TrackerError`] can be.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error in a I/O operation.
    Io(io::Error),
    /// A request error.
    Request(reqwest::Error),
    /// An error while parsing a Fimfiction response.
    UnexpectedResponse {
        /// The story ID that caused the error.
        id: Id,
        /// The raw response that caused the error.
        response: String,
        /// The error being thrown.
        error: StoryError,
    },
    /// An error made while trying to compare two [`Story`](crate::story::Story) structs of
    /// different IDs.
    BadStoryComparison {
        /// The ID of the story doing the comparison.
        id: u32,
        /// The ID of the story being compared.
        other_id: u32,
    },
    /// An error while parsing a configuration source.
    ConfigParsing(ConfigSource),
    /// An error while (de)serializing [`StoryData`](crate::StoryData).
    TrackerFormat {
        /// Path to the tracker file that caused the error, if relevant.
        path: Option<String>,
        /// The action that was being done when the error happened.
        action: Action,
        /// The error being thrown.
        error: serde_json::Error,
    },
    /// A custom error.
    Custom(String),
}

/// The error type for all errors present in the crate.
#[derive(Debug)]
pub struct TrackerError {
    context: Option<String>,
    /// The kind of error.
    pub kind: ErrorKind,
}

impl TrackerError {
    /// Constructs a new [`TrackerError`] of a given kind.
    pub fn with(kind: ErrorKind) -> Self {
        TrackerError {
            context: None,
            kind,
        }
    }

    /// Gives additional context to the error message.
    pub fn context<C>(mut self, context: C) -> Self
    where
        C: Into<String>,
    {
        let _ = self.context.insert(context.into());
        self
    }

    /// Constructs a [`TrackerError`] of kind [`Io`](ErrorKind::Io).
    pub fn io(err: io::Error) -> Self {
        TrackerError::with(ErrorKind::Io(err))
    }

    /// Constructs a [`TrackerError`] of kind [`Request`](ErrorKind::Request).
    pub fn request(err: reqwest::Error) -> Self {
        TrackerError::with(ErrorKind::Request(err))
    }

    /// Constructs a [`TrackerError`] of kind
    /// [`UnexpectedResponse`](ErrorKind::UnexpectedResponse).
    pub fn unexpected_response(err: StoryError, id: Id, response: String) -> Self {
        TrackerError::with(ErrorKind::UnexpectedResponse {
            id,
            response,
            error: err,
        })
    }

    /// Constructs a [`TrackerError`] of kind
    /// [`BadStoryComparison`](ErrorKind::BadStoryComparison).
    pub fn story_comparison(id: u32, other_id: u32) -> Self {
        TrackerError::with(ErrorKind::BadStoryComparison { id, other_id })
    }

    /// Constructs a [`TrackerError`] of kind
    /// [`ConfigParsing`](ErrorKind::ConfigParsing).
    pub fn config_parsing(source: ConfigSource) -> Self {
        TrackerError::with(ErrorKind::ConfigParsing(source))
    }

    /// Constructs a [`TrackerError`] of kind [`TrackerFormat`](ErrorKind::TrackerFormat).
    pub fn tracker_format<T>(path: T, error: serde_json::Error, action: Action) -> Self
    where
        T: Into<Option<String>>,
    {
        TrackerError::with(ErrorKind::TrackerFormat {
            path: path.into(),
            action,
            error,
        })
    }

    /// Constructs a [`TrackerError`] of kind [`Custom`](ErrorKind::Custom).
    pub fn custom<M>(message: M) -> Self
    where
        M: ToString,
    {
        TrackerError::with(ErrorKind::Custom(message.to_string()))
    }
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(message) = &self.context {
            write!(f, "{}: ", message)?;
        }

        match &self.kind {
            ErrorKind::UnexpectedResponse { id, ref error, .. } => {
                write!(f, "error parsing response for ID `{}`: {}", id, error)?;
            }
            ErrorKind::Io(ref err) => {
                write!(f, "IO error: {}", err)?;
            }
            ErrorKind::Request(ref err) => {
                write!(f, "{}", err)?;
            }
            ErrorKind::BadStoryComparison { id, other_id } => {
                write!(
                    f,
                    "cannot compare for update when story ID's aren't the same ({} != {})",
                    id, other_id
                )?;
            }
            ErrorKind::ConfigParsing(source) => {
                write!(f, "error parsing configuration ")?;

                match source {
                    ConfigSource::File { path, error } => {
                        write!(f, "in `{}`: {}", path, error.message())?;
                    }
                    ConfigSource::Env(error) => {
                        write!(f, "in `the environment`: {}", error)?;
                    }
                }
            }
            ErrorKind::TrackerFormat {
                ref path,
                ref action,
                ref error,
            } => {
                write!(f, "error in tracker format")?;

                if let Some(path) = path {
                    write!(f, " from `{}`", path)?;
                }

                write!(f, " while ")?;
                match action {
                    Action::Deserializing => write!(f, "deserializing")?,
                    Action::Serializing => write!(f, "serializing")?,
                };

                write!(f, ": {}", error)?;
            }
            ErrorKind::Custom(err) => {
                write!(f, "{}", err)?;
            }
        };

        Ok(())
    }
}

impl Error for TrackerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            ErrorKind::Io(ref err) => Some(err),
            ErrorKind::Request(ref err) => Some(err),
            ErrorKind::UnexpectedResponse { ref error, .. } => Some(error),
            ErrorKind::ConfigParsing(ref source) => Some(match source {
                ConfigSource::File { ref error, .. } => error,
                ConfigSource::Env(ref err) => err,
            }),
            ErrorKind::TrackerFormat { ref error, .. } => Some(error),
            ErrorKind::BadStoryComparison { .. } | ErrorKind::Custom(_) => None,
        }
    }
}
