#[macro_use]
extern crate log;

use std::path::Path;

use clap::Parser;

use fimfic_tracker::{
    downloader::BlockingRequester, Config, ConfigBuilder, Result, StoryData, TrackerError,
};

#[macro_use]
mod macros;
mod args;
mod error;
mod listener;
mod logger;
mod readable;
mod subcommands;

use args::{Args, SubCommand};
use listener::ProgressOutput;

pub type Requester = BlockingRequester<ProgressOutput>;

pub fn create_dir_all<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|error| {
        TrackerError::io(error).context(format!(
            "failed to create directories to `{}`",
            path.display()
        ))
    })
}

mod backup {
    use super::*;

    use std::{io, path::Path};

    use chrono::Local;
    use fimfic_tracker::{default_user_tracker_file, errors::ErrorKind};

    fn is_permission_error(error: &TrackerError) -> bool {
        if let ErrorKind::Io(io_error) = &error.kind {
            if io_error.kind() == io::ErrorKind::PermissionDenied {
                return true;
            };
        }

        false
    }

    /// Function to wrap anything that can error while creating the backup.
    fn create_backup(path: &Path, mut data: StoryData) -> Result<()> {
        // The path leading to the file might not exist, try to create all the required parent
        // directories.
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        // The `path` field is private and cannot be changed, so we have to construct a new
        // `StoryData` and bring the `IndexMap` to it.
        let mut story_data = StoryData::new(path);
        *story_data = std::mem::take(&mut *data);
        story_data.save()
    }

    /// To be called when the application threw an error and the tracker data couldn't be saved.
    ///
    /// If the saving error is related to file permissions, this tries to save a backup to its
    /// default directory and then briefly inform of the result.
    pub fn story_data_on_error(error: &TrackerError, data: StoryData) {
        if is_permission_error(error) {
            let mut path = default_user_tracker_file();
            path.set_file_name({
                let now = Local::now();
                format!("backup-track-data_{}.json", now.format("%Y-%m-%d_%H-%M-%S"))
            });

            match create_backup(path.as_ref(), data) {
                Ok(_) => warn!("Saved backup of track data to `{}`", path.display()),
                Err(err) => error!(
                    "Unable to save backup of track data to `{}`: {}",
                    path.display(),
                    err
                ),
            };

            separate!();
        }
    }
}

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
        create_dir_all(path)?;
    }

    let mut story_data = StoryData::new(&config.tracker_file);
    story_data.load()?;
    debug!("Loaded story data: {:?}", &story_data);

    let result = match args.subcommand {
        SubCommand::Track(track_args) => {
            subcommands::track(&config, &requester, &mut story_data, track_args)
        }
        SubCommand::Untrack(_) | SubCommand::List(_) | SubCommand::Download(_)
            if story_data.is_empty() =>
        {
            warn!("There are no stories in the tracking list!");
            Ok(())
        }
        SubCommand::Untrack(untrack_args) => {
            subcommands::untrack(&mut story_data, untrack_args);
            Ok(())
        }
        SubCommand::List(list_args) => {
            subcommands::list(&story_data, list_args);
            Ok(())
        }
        SubCommand::Download(download_args) => {
            subcommands::download(&config, &requester, &mut story_data, download_args)
        }
    };

    match story_data.save() {
        Ok(_) => {
            debug!("Saved story data to tracker file");
        }
        Err(err) => {
            backup::story_data_on_error(&err, story_data);
            error::pretty_print(err);

            match result.as_ref() {
                // The saving error was the only one that the application has thrown, exit with a
                // non-zero code.
                Ok(_) => std::process::exit(1),
                // We still need to show the application error, put a separator between them.
                // TODO: Use a line separator that covers the entire width of the terminal window?
                Err(_) => separate!(),
            };
        }
    };

    result
}

fn main() {
    let args = Args::parse();
    logger::configure(args.verbose, args.color);

    if let Err(err) = run(args) {
        error::pretty_print(err);
        std::process::exit(1)
    }
}
