use fimfiction_api::StoryError;
use std::{error::Error, fmt, io};

/// An alias of [`Result`] for all of its instances across the crate.
pub type Result<T> = std::result::Result<T, TrackerError>;

/// The different kinds of errors that [`TrackerError`] can be.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error in a I/O operation.
    Io(io::Error),
    /// An error while parsing a Fimfiction response.
    UnexpectedResponse {
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
    ConfigParsing {
        /// The source that caused the error.
        origin: Option<String>,
        /// The error being thrown.
        cause: Box<dyn Error + Send + Sync>,
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
        self.context = Some(context.into());
        self
    }

    /// Constructs a [`TrackerError`] of kind [`Io`](ErrorKind::Io).
    pub fn io(err: io::Error) -> Self {
        TrackerError::with(ErrorKind::Io(err))
    }

    /// Constructs a [`TrackerError`] of kind
    /// [`UnexpectedResponse`](ErrorKind::UnexpectedResponse).
    pub fn unexpected_response(err: StoryError, response: String) -> Self {
        TrackerError::with(ErrorKind::UnexpectedResponse {
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
    pub fn config_parsing<O, E>(origin: O, cause: E) -> Self
    where
        O: ToString,
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        TrackerError::with(ErrorKind::ConfigParsing {
            origin: Some(origin.to_string()),
            cause: cause.into(),
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
            ErrorKind::UnexpectedResponse { ref error, .. } => {
                write!(f, "error parsing response: {}", err)?;
            }
            ErrorKind::Io(ref err) => {
                write!(f, "IO error: {}", err)?;
            }
            ErrorKind::BadStoryComparison { id, other_id } => {
                write!(
                    f,
                    "cannot compare for update when story ID's aren't the same ({} != {})",
                    id, other_id
                )?;
            }
            ErrorKind::ConfigParsing {
                ref origin,
                ref cause,
            } => {
                write!(f, "error parsing configuration")?;

                if let Some(origin) = origin {
                    write!(f, " in `{}`", origin)?;
                }

                write!(": {}", cause)?;
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
            ErrorKind::UnexpectedResponse { ref error, .. } => Some(error),
            // ErrorKind::ConfigParsing { ref cause, .. } => Some(&cause),
            // ErrorKind::Custom(ref message) => Some(&message),
            _ => None,
        }
    }
}
