use console::{style, Term};
use fimfic_tracker::{downloader::ProgressListener, Config, Story};

use crate::logger::PROGRESS_PREFIX;
use crate::readable::ReadableBytes;

#[derive(Clone)]
pub struct ProgressOutput {
    stderr: Term,
    quiet: bool,
}

impl ProgressOutput {
    pub fn new(config: Config) -> Self {
        Self {
            stderr: Term::stderr(),
            quiet: config.quiet,
        }
    }
}

impl ProgressListener for ProgressOutput {
    fn download_progress(&self, bytes: usize, filepath: &str) {
        let started = bytes != 0;

        if !verbose_disabled!() {
            if !started {
                info!("Starting download: {:?}", &filepath);
            }

            return;
        }

        let cols = {
            let cols = self.stderr.size().1;
            (cols - 1) as usize
        };

        if started {
            clear_last_lines!();
        };

        let suffix = format!(" [{}]", ReadableBytes(bytes));
        let used_cols = PROGRESS_PREFIX.len() + suffix.len();

        if cols > used_cols {
            let remaining = cols - used_cols;
            let center = if remaining >= filepath.len() {
                filepath
            } else {
                let idx = filepath.len() - remaining;
                &filepath[idx..]
            };

            progress!("{}{}", center, style(suffix).green());
        } else {
            progress!();
        };
    }

    fn successfull_client_download(&self, story: &Story) {
        clear_last_lines!();

        info!(
            "{} {} {}",
            style("Successfully downloaded").green(),
            style(&story.title).green().bold(),
            style(format!("({})", story.id)).green()
        );
    }

    fn before_execute_command(&self, story: &Story) {
        progress_or_info!(
            "{}",
            style(format!(
                "Executing command for {} ({})",
                &story.title, story.id
            ))
            .bold(),
        );
    }

    fn successfull_command_execution(&self, story: &Story) {
        if self.quiet {
            clear_last_lines!();
        }

        info!(
            "{} {} {}",
            style("Command finished successfully for").green(),
            style(&story.title).green().bold(),
            style(format!("({})", story.id)).green()
        );
    }
}
