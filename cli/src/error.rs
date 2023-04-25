use std::{borrow::Cow, error::Error};

use fimfic_tracker::errors::{Action, ErrorKind, StoryError, TrackerError};

static DEFAULT_INDENT: usize = 3;
static DEBUG_INDENT: usize = 2;
static ISSUE_URL: &str = "https://github.com/ZodiacalComet/fimfic-tracker/issues";

fn indent_msg(msg: &str, indent: usize) -> String {
    msg.lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line.into()
            } else {
                format!("{:>indent$}{}", "", line, indent = indent)
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Indicator of the additional information to add at the end of the error message.
enum ErrorMessage {
    /// Requires manual intervention to fix.
    ///
    /// This also indicates that a suggestion or a list of fixes were given to the user, so there
    /// is little need to add more to it.
    Fixable,
    /// An error that might fix itself later.
    ///
    /// This should suggest trying again later.
    TryAgain,
    /// Fimfiction threw an error that wasn't seen while developing its deserializer.
    ///
    /// This should suggest opening an issue with the raw API response and the story ID that causes
    /// it.
    ApiError,
    /// An error that while technically possible the program is taking care to not get it, that it
    /// happened anyway means that there is case that wasn't taken into account.
    ///
    /// This should suggest opening an issue.
    Internal,
    /// An error coming from a thrid-party with an unknown cause.
    ///
    /// This indicates indications on what's happening or how to fix it couldn't be given and that
    /// the user should really considerate opening an issue.
    Unknown,
}

pub fn pretty_print(error: TrackerError) {
    if verbose_disabled!() {
        error!("Error: {}", error);
    } else {
        debug!("{:?}", error);
        error!("Message: {}", indent_msg(&error.to_string(), DEBUG_INDENT));
    }

    // Show the raw API response if its contents cannot be inferred by the error.
    if let ErrorKind::UnexpectedResponse {
        response, error, ..
    } = &error.kind
    {
        if !matches!(error, StoryError::InvalidId) {
            separate!();
            error!("Response: {}", response);
        }
    };

    let indent = if verbose_disabled!() {
        DEFAULT_INDENT
    } else {
        DEBUG_INDENT
    };

    let mut stack = error.source();
    if stack.is_some() {
        separate!();
        if verbose_disabled!() {
            error!("Source:");
        }

        fn pretty_fmt(indent: usize, level: usize, msg: &str) {
            error!(
                "{:>indent$}: {}",
                level,
                indent_msg(msg, indent + 2),
                indent = indent
            );
        }

        fn verbose_fmt(indent: usize, level: usize, msg: &str) {
            error!("Source {:>02}: {}", level, indent_msg(msg, indent));
        }

        let fmt = if verbose_disabled!() {
            pretty_fmt
        } else {
            verbose_fmt
        };

        let mut level = 1;
        while let Some(err) = stack {
            fmt(indent, level, &err.to_string());
            debug!("{:?}", err);

            stack = err.source();
            level += 1;
        }
    }

    // For verbose imput, which is more of a debug output, we omit everthing after this.
    // Those are user facing messages and shoudn't be required to diagnose a problem.
    if !verbose_disabled!() {
        return;
    }

    let mut explanation: Option<Cow<'_, str>> = None;
    let mut help: Option<Cow<'_, str>> = None;
    let mut error_message: Option<ErrorMessage> = None;

    match &error.kind {
        ErrorKind::UnexpectedResponse { id, error, .. } => match error {
            StoryError::Json(err) => {
                let suffix = if err.is_data() {
                    let _ = error_message.insert(ErrorMessage::Internal);
                    Some("doesn't follow the expected format.")
                } else if err.is_syntax() || err.is_eof() {
                    let _ = error_message.insert(ErrorMessage::TryAgain);
                    Some("wasn't a valid JSON.")
                } else {
                    None
                };

                if let Some(suffix) = suffix {
                    let _ = explanation
                        .insert(format!("The response from the Fimfiction API {}", suffix).into());
                }
            }
            StoryError::InvalidId => {
                let _ = help.insert(
                    format!(
                        "ID `{}` didn't point to a valid story. Did you make a typo?",
                        id
                    )
                    .into(),
                );
                let _ = error_message.insert(ErrorMessage::Fixable);
            }
            StoryError::Api(_) => {
                let _ = explanation.insert("An unrecognized error came from the API.".into());
                let _ =
                    help.insert(
                        format!(
                            "Check that `https://fimfiction.net/story/{}` gets you an actual story from Fimfiction. \
                            If it doesn't then story was deleted or you probably made a typo while entering \
                            the ID or URL.",
                            id
                        ).into()
                    );
                let _ = error_message.insert(ErrorMessage::ApiError);
            }
        },
        ErrorKind::BadStoryComparison { .. } => {
            let _ = explanation
                .insert("The story changed its ID somehow while checking for an update.".into());
            let _ = error_message.insert(ErrorMessage::Internal);
        }
        ErrorKind::TrackerFormat { action, error, .. } => match action {
            Action::Serializing => {
                let _ = explanation
                    .insert("The cached story data couldn't be prepared to be saved.".into());
                let _ = error_message.insert(ErrorMessage::Internal);
            }
            Action::Deserializing => {
                let _ = explanation
                    .insert("The tracker file was read but couldn't be understood.".into());

                let mut fixes = Vec::new();

                if error.is_data() {
                    fixes.push(
                        "Did the tracker file come from the legacy version? \
                        The storing format has changed and you have to migrate to be able to use it \
                        with this version.",
                    );
                }

                if error.is_syntax() || error.is_eof() {
                    fixes.extend_from_slice(&[
                        "Did you manually modify it? \
                        Check that the file is still a valid JSON and that any modification still \
                        follows storing format.",
                        "Haven't touched that file? \
                        There is not much that can be done besides manually fixing it.",
                    ]);
                }

                if !fixes.is_empty() {
                    fixes.push(
                        "If worst come to worst you can still save your list by \
                        (1) taking note of any story ID that you can from it, \
                        (2) moving the tracker file to another location or delete it \
                        and (3) re-`track`ing them.",
                    );
                }

                if fixes.is_empty() {
                    // If we are here, `err.is_io()` is true which shouldn't be possible to get.
                    let _ = error_message.insert(ErrorMessage::Unknown);
                } else {
                    let _ = help.insert(
                        format!(
                            "\n{}",
                            fixes
                                .iter()
                                .map(|fix| format!("* {}", fix))
                                .collect::<Vec<String>>()
                                .join("\n")
                        )
                        .into(),
                    );
                    let _ = error_message.insert(ErrorMessage::Fixable);
                };
            }
        },
        // Io: Not much that can say about it.
        // Request: Could be nice to have messages in the style of a web browser, but I don't know
        //   how to determine each case or what even a good message would be.
        // ConfigParsing: The error and source give a pretty good idea on what the error is and how
        //   to fix it.
        // Custom: Cannot be relied upon to know what happened.
        ErrorKind::Io(_)
        | ErrorKind::Request(_)
        | ErrorKind::ConfigParsing(_)
        | ErrorKind::Custom(_) => {}
    };

    if let Some(message) = explanation {
        separate!();
        error!("Explanation: {}", indent_msg(&message, indent));
    }

    if let Some(message) = help {
        separate!();
        error!("Help: {}", indent_msg(&message, indent))
    }

    if let Some(kind) = error_message {
        match kind {
            ErrorMessage::Fixable => {}
            ErrorMessage::TryAgain => {
                separate!();
                error!(
                    "Try again later, if the problem persists then please open an issue in `{}`.",
                    ISSUE_URL
                );
            }
            ErrorMessage::ApiError => {
                separate!();
                error!(
                    "You have found an unhandled API error, I implore you to open an issue in `{}`\
                    with at least the `Response` content and, if possible, the story ID that caused it.",
                    ISSUE_URL
                );
            }
            ErrorMessage::Internal | ErrorMessage::Unknown => {
                separate!();

                let start = if matches!(kind, ErrorMessage::Internal) {
                    "This error is not meant to happen, it would be very appreciated if you could:"
                } else {
                    "I implore you to:"
                };

                let msg = format!(
                    "{}\n\
                    1. Run the command again with the verbose flag (`-vv`).\n\
                    2. If this error happened again copy its entire contents, otherwise do it with this one.\n\
                    3. Open an issue in `{}` with the copied error.",
                    start, ISSUE_URL
                );

                error!("{}", indent_msg(&msg, indent));
            }
        };
    }
}
