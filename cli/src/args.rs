use clap::{
    arg,
    builder::{Command, NonEmptyStringValueParser, TypedValueParser},
    error::{ContextKind, ContextValue, Error, ErrorKind, RichFormatter},
    Arg, ArgAction, ArgMatches, ColorChoice, FromArgMatches, Parser, Subcommand, ValueEnum,
    ValueHint,
};

#[derive(Parser, Debug, PartialEq)]
#[clap(name = "fimfic-tracker", version, author)]
/// An unnecessary CLI application for tracking Fimfiction stories.
pub struct Args {
    /// Extra config file to use.
    #[clap(
        short,
        long,
        value_name = "FILE",
        value_hint(ValueHint::FilePath),
        display_order = 1,
        value_parser(NonEmptyStringValueParser::new())
    )]
    pub config: Option<String>,
    /// Shows verbose output, can be used multiple times to set level of verbosity.
    #[clap(short, long, display_order = 2, action(ArgAction::Count))]
    pub verbose: u8,
    /// When to use colors.
    #[clap(long, display_order = 3, value_enum, default_value_t)]
    pub color: ColorChoice,
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum SubCommand {
    #[clap(display_order = 1)]
    Track(Track),
    #[clap(display_order = 2)]
    Untrack(Untrack),
    #[clap(display_order = 3)]
    List(List),
    #[clap(display_order = 4)]
    Download(Download),
}

#[derive(Clone)]
struct StoryValueParser;

/// Extract story ID from Fimfiction story URL.
///
/// Manual implementation of the following regular expression and retrieving the first capture
/// group: `^https?://(?:www\.)?fimfiction\.net/story/(\d+)`.
fn id_from_url(url: &str) -> Option<u32> {
    let (protocol, rest) = url.split_once("://")?;
    if !(protocol == "http" || protocol == "https") {
        return None;
    }

    let (mut domain, rest) = rest.split_once('/')?;
    if domain.starts_with("www.") {
        domain = &domain[4..];
    }

    if domain != "fimfiction.net" {
        return None;
    }

    let (path, rest) = rest.split_once('/')?;
    if path != "story" {
        return None;
    }

    let id = if let Some(index) = rest.find('/') {
        &rest[..index]
    } else {
        rest
    };

    id.parse::<u32>().ok()
}

impl TypedValueParser for StoryValueParser {
    type Value = u32;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        if let Ok(id) = clap::value_parser!(u32).parse_ref(cmd, arg, value) {
            return Ok(id);
        }

        match NonEmptyStringValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(|url| id_from_url(url.as_str()))?
        {
            Some(id) => Ok(id),
            None => {
                // TODO: Add to the error a "not a Fimfiction story URL nor a story ID".
                let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);

                if let Some(arg) = arg {
                    err.insert(
                        ContextKind::InvalidArg,
                        ContextValue::String(arg.to_string()),
                    );
                }
                err.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(value.to_string_lossy().into_owned()),
                );

                Err(err)
            }
        }
    }
}

#[derive(clap::Args, Debug, PartialEq)]
#[clap(visible_alias = "t")]
/// Adds stories for tracking and downloads them.
pub struct Track {
    /// Overwrites already present stories on cached data.
    #[clap(short, long, display_order = 1)]
    pub overwrite: bool,
    /// Don't download stories, only updates cached data.
    #[clap(short, long, display_order = 2)]
    pub skip_download: bool,
    /// IDs or URLs of stories to track.
    #[clap(
        value_name = "ID_OR_URL",
        required = true,
        value_hint(ValueHint::Url),
        value_parser(StoryValueParser)
    )]
    pub ids: Vec<u32>,
}

#[derive(clap::Args, Debug, PartialEq)]
#[clap(visible_alias = "u")]
/// Untracks stories.
pub struct Untrack {
    /// IDs or URLs of stories to untrack.
    #[clap(
        value_name = "ID_OR_URL",
        required = true,
        value_hint(ValueHint::Other),
        value_parser(StoryValueParser)
    )]
    pub ids: Vec<u32>,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum SortKey {
    Id,
    Title,
    Author,
    Chapters,
    Words,
    Update,
}

#[derive(clap::Args, Debug, PartialEq)]
#[clap(visible_alias = "l", visible_alias = "ls")]
/// List all stories that are being tracked.
pub struct List {
    /// Show only the ID and title of each tracked story.
    #[clap(short, long, display_order = 1)]
    pub short: bool,
    /// Sort stories by the given key.
    #[clap(long, value_name = "KEY", display_order = 2, value_enum)]
    pub sort_by: Option<SortKey>,
    /// Reverse the order of the list.
    #[clap(short, long, display_order = 3)]
    pub reverse: bool,
}

#[derive(Debug, PartialEq)]
pub enum Prompt {
    AssumeYes,
    AssumeNo,
    Ask,
}

impl clap::Args for Prompt {
    fn augment_args(cmd: Command) -> Command {
        cmd.arg(
            arg!(-y --yes "Automatically answers prompts with Y")
                .display_order(50)
                .conflicts_with("no"),
        )
        .arg(arg!(-n --no "Automatically answers prompts with N").display_order(51))
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Prompt::augment_args(cmd)
    }
}

impl FromArgMatches for Prompt {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error<RichFormatter>> {
        Ok(if matches.get_flag("yes") {
            Prompt::AssumeYes
        } else if matches.get_flag("no") {
            Prompt::AssumeNo
        } else {
            Prompt::Ask
        })
    }

    fn update_from_arg_matches(
        &mut self,
        matches: &ArgMatches,
    ) -> Result<(), Error<RichFormatter>> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

#[derive(clap::Args, Debug, PartialEq)]
#[clap(visible_alias = "d")]
/// Checks for updates on tracking list and downloads them if so.
pub struct Download {
    /// Download no matter the presence of updates.
    #[clap(short, long, display_order = 1)]
    pub force: bool,
    #[clap(flatten)]
    pub prompt: Prompt,
    /// IDs or URLs of stories to check.
    #[clap(
        value_name = "ID_OR_URL",
        value_hint(ValueHint::Other),
        value_parser(StoryValueParser)
    )]
    pub ids: Vec<u32>,
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_id {
        ($id:literal / $name:literal, $expect:expr) => {
            assert_id!([trail] $id, $expect);
            assert_id!([trail] concat!($id, "/", $name), $expect);
        };
        ([trail] $path:expr, $expect:expr) => {
            assert_id!([prefixes] $path, $expect);
            assert_id!([prefixes] concat!($path, "/"), $expect);
        };
        ([prefixes] $path:expr, $expect:expr) => {
            assert_id!(concat!("https://www.fimfiction.net/story/", $path), $expect);
            assert_id!(concat!("http://www.fimfiction.net/story/", $path), $expect);
            assert_id!(concat!("https://fimfiction.net/story/", $path), $expect);
            assert_id!(concat!("http://fimfiction.net/story/", $path), $expect);
        };
        ($url:expr, $expect:expr) => {
            assert_eq!(id_from_url($url), Some($expect), "failed to extract ID from `{}`", $url);
        };
    }

    #[test]
    fn extract_story_id_from_url() {
        assert_id!("196256" / "the-moons-apprentice", 196256);
        assert_id!([prefixes] "196256/1/the-moons-apprentice/original-oneshot-prelude-a-dream-fulfilled", 196256);
    }
}
