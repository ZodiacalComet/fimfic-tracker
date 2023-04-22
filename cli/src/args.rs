use clap::{arg, App, ArgMatches, Error, FromArgMatches, Parser, Subcommand, ValueHint};

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
        forbid_empty_values(true)
    )]
    pub config: Option<String>,
    /// Shows verbose output, can be used multiple times to set level of verbosity.
    #[clap(short, long, display_order = 2, parse(from_occurrences))]
    pub verbose: u64,
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
        forbid_empty_values(true)
    )]
    pub stories: Vec<String>,
}

#[derive(clap::Args, Debug, PartialEq)]
#[clap(visible_alias = "u")]
/// Untracks stories.
pub struct Untrack {
    /// IDs of stories to untrack.
    #[clap(
        value_name = "ID",
        required = true,
        value_hint(ValueHint::Other),
        forbid_empty_values(true)
    )]
    pub ids: Vec<String>,
}

#[derive(clap::Args, Debug, PartialEq)]
#[clap(visible_alias = "l", visible_alias = "ls")]
/// List all stories that are being tracked.
pub struct List {
    /// Show only the ID and title of each tracked story.
    #[clap(short, long, display_order = 1)]
    pub short: bool,
}

#[derive(Debug, PartialEq)]
pub enum Prompt {
    AssumeYes,
    AssumeNo,
    Ask,
}

impl clap::Args for Prompt {
    fn augment_args(app: App<'_>) -> App<'_> {
        app.arg(
            arg!(-y --yes "Automatically answers prompts with Y")
                .display_order(50)
                .conflicts_with("no"),
        )
        .arg(arg!(-n --no "Automatically answers prompts with N").display_order(51))
    }

    fn augment_args_for_update(app: App<'_>) -> App<'_> {
        Prompt::augment_args(app)
    }
}

impl FromArgMatches for Prompt {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
        Ok(if matches.is_present("yes") {
            Prompt::AssumeYes
        } else if matches.is_present("no") {
            Prompt::AssumeNo
        } else {
            Prompt::Ask
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        *self = if matches.is_present("yes") {
            Prompt::AssumeYes
        } else if matches.is_present("no") {
            Prompt::AssumeNo
        } else {
            Prompt::Ask
        };

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
    /// IDs of stories to check.
    #[clap(
        value_name = "ID",
        value_hint(ValueHint::Other),
        forbid_empty_values(true)
    )]
    pub ids: Vec<String>,
}
