#[macro_use]
extern crate log;

use std::fs::create_dir_all;

use clap::Parser;

use fimfic_tracker::{
    downloader::BlockingRequester, Config, ConfigBuilder, ErrorKind, Result, StoryData,
    TrackerError,
};

#[macro_use]
mod macros;
mod args;
mod listener;
mod logger;
mod readable;
mod subcommands;

use args::{Args, SubCommand};
use listener::ProgressOutput;

pub type Requester = BlockingRequester<ProgressOutput>;

fn run(args: Args) -> Result<()> {
    debug!("Parsed arguments: {:?}", &args);

    let config: Config = ConfigBuilder::from_default_sources()
        .and_then(|builder| match args.config.as_ref() {
            Some(path) => ConfigBuilder::from_file(path).map(|c| builder.merge(c)),
            None => Ok(builder),
        })?
        .into();
    debug!("Loaded config: {:?}", &config);

    let requester = BlockingRequester::new(config.clone(), ProgressOutput::new(config.clone()));

    for path in [
        Some(config.download_dir.as_ref()),
        config.tracker_file.parent(),
    ]
    .iter()
    .flatten()
    .filter(|path| !path.is_dir())
    {
        debug!("Creating directories to {}", path.display());
        create_dir_all(path).map_err(|err| {
            TrackerError::io(err).context(format!(
                "Failed to create directories to {}",
                path.display()
            ))
        })?;
    }

    let mut story_data = StoryData::new(&config.tracker_file);
    story_data.load()?;
    debug!("Loaded story data: {:?}", &story_data);

    match args.subcommand {
        SubCommand::Track(track_args) => {
            subcommands::track(&config, &requester, &mut story_data, track_args)?;
        }
        SubCommand::Untrack(_) | SubCommand::List(_) | SubCommand::Download(_)
            if story_data.is_empty() =>
        {
            warn!("There are no stories in the tracking list!");
        }
        SubCommand::Untrack(untrack_args) => {
            subcommands::untrack(&mut story_data, untrack_args);
        }
        SubCommand::List(list_args) => {
            subcommands::list(&story_data, list_args);
        }
        SubCommand::Download(download_args) => {
            subcommands::download(&config, &requester, &mut story_data, download_args)?;
        }
    };

    story_data.save()?;
    debug!("Saved story data to tracker file");
    Ok(())
}

fn main() {
    let args = Args::parse();
    logger::configure(args.verbose);

    if let Err(err) = run(args) {
        match &err.kind {
            ErrorKind::UnexpectedResponse { response, .. } => {
                error!("{}\nResponse content: {}", &err, response)
            }
            ErrorKind::BadStoryComparison { .. } => {
                error!("{}\nThis is an internal error and it should't happen.", err)
            }
            ErrorKind::Io(_) | ErrorKind::ConfigParsing { .. } | ErrorKind::Custom(_) => {
                error!("{}", err)
            }
        };

        std::process::exit(1)
    }
}
