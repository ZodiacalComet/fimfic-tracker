use std::process::Stdio;

use futures_util::StreamExt;
use reqwest;
use tokio::{fs, io, process::Command};
use url::Url;

use crate::config::Config;
use crate::errors::{self, TrackerError};
use crate::story::Story;
use crate::utils::{download_url_format, env_with_command_context, sanitize_filename};
use crate::StoryResponse;

use super::listener::ProgressListener;

async fn download<S, P>(
    res: reqwest::Response,
    mut dest: fs::File,
    filepath: S,
    progress: &P,
) -> errors::Result<()>
where
    S: ToString,
    P: ProgressListener,
{
    let filepath = filepath.to_string();
    let mut total_bytes: usize = 0;

    progress.download_progress(total_bytes, &filepath);

    let mut stream = res.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(TrackerError::custom)?;

        total_bytes += chunk.len();
        progress.download_progress(total_bytes, &filepath);

        io::copy(&mut chunk.as_ref(), &mut dest)
            .await
            .map_err(TrackerError::io)?;
    }

    Ok(())
}

/// An asynchronous story downloader.
///
/// Makes use of an asynchronous [`Client`](reqwest::Client) for all of its requests.
///
/// ```no_run
/// # use tokio;
/// # use fimfic_tracker::Result;
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// # use fimfic_tracker::{Config, StoryData};
/// use fimfic_tracker::downloader::{AsyncRequester, SilentListener};
/// # let config = Config::default();
///
/// let requester = AsyncRequester::new(config, SilentListener {});
///
/// // Requesting "The Moon's Apprentice" by Forthwith
/// let story = requester.get_story_response("196256").await?;
/// println!("{:?}", story);
///
/// // Download story according to the configuration file.
/// requester.download(&story.into()).await?;
/// # Ok(())
/// # }
/// ```
pub struct AsyncRequester<P>
where
    P: ProgressListener,
{
    client: reqwest::Client,
    config: Config,
    progress: P,
}

impl<P> AsyncRequester<P>
where
    P: ProgressListener,
{
    /// Constructs a new [`AsyncRequester`].
    pub fn new(config: Config, progress: P) -> Self {
        AsyncRequester {
            client: reqwest::Client::new(),
            config,
            progress,
        }
    }

    /// Requests the [`StoryResponse`] of the given ID or URL from Fimfiction.
    pub async fn get_story_response<T>(&self, id_or_url: T) -> errors::Result<StoryResponse>
    where
        T: AsRef<str>,
    {
        let url = Url::parse_with_params(
            "https://www.fimfiction.net/api/story.php",
            &[("story", id_or_url)],
        )
        .expect("Fimficiton API URL parse failed");

        let json = self
            .client
            .get(url)
            .send()
            .await
            .map_err(TrackerError::custom)?
            .text()
            .await
            .map_err(|err| {
                TrackerError::custom(err)
                    .context("Couldn't decode the Fimfiction API response body")
            })?;

        fimfiction_api::from_str(&json).map_err(|err| TrackerError::unexpected_response(err, json))
    }

    /// Downloads `story` from Fimfiction into the download directory in the
    /// [`DownloadFormat`](crate::DownloadFormat) specified in the given [`Config`].
    ///
    /// Uses a sanitized `{TITLE}.{FORMAT}` as the filename.
    ///
    /// # Errors
    ///
    /// They are returned according to tokio's [`fs::OpenOptions::open()`] and [`io::copy()`].
    pub async fn client_download(&self, story: &Story) -> errors::Result<()> {
        let req = self
            .client
            .get(download_url_format(story, self.config.download_format));

        let filename =
            sanitize_filename(format!("{}.{}", &story.title, self.config.download_format));
        let filepath = self.config.download_dir.join(filename);

        let res = req
            .send()
            .await
            .map_err(|err| TrackerError::custom(err).context("Failed to prepare download page"))?;

        let dest = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filepath)
            .await
            .map_err(|err| {
                TrackerError::io(err)
                    .context(format!("Failed to create file {}", filepath.display()))
            })?;

        download(res, dest, filepath.display(), &self.progress)
            .await
            .map_err(|err| {
                TrackerError::custom(err).context(format!(
                    "Failed to download story to {}",
                    filepath.display()
                ))
            })?;

        self.progress.successfull_client_download(story);

        Ok(())
    }

    /// Expands shell-like variables present in `command` and then executes it with tokio's
    /// [`Command`], taking into account the value of `config.quiet`.
    ///
    /// More info on said expansion in [`env_with_command_context()`].
    ///
    /// # Errors
    ///
    /// Besides failing on a badly written `command` it can error according to
    /// [`Command::status()`].
    pub async fn exec_download<S>(&self, command: S, story: &Story) -> errors::Result<()>
    where
        S: AsRef<str>,
    {
        let exec = env_with_command_context(command, story, &self.config)?;
        let args = match shlex::split(&exec) {
            Some(args) => args,
            None => {
                return Err(TrackerError::custom(
                    "Exec command should mimic a POSIX shell command",
                ))
            }
        };

        let mut command = Command::new(&args[0]);
        if args.len() > 1 {
            command.args(&args[1..]);
        }

        if self.config.quiet {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }

        self.progress.before_execute_command(story);

        let status = command.status().await.map_err(|err| {
            TrackerError::io(err).context(format!("Failed to execute command: {:?}", &exec))
        })?;

        if !status.success() {
            let err = match status.code() {
                Some(code) => TrackerError::custom(format!(
                    "Command process exited with status code {}",
                    code
                )),
                None => TrackerError::custom("Command process was terminated by signal"),
            };

            return Err(err.context(format!("Failed executing command: {}", &exec)));
        }

        self.progress.successfull_command_execution(story);
        Ok(())
    }

    /// Downloads the given `story` from Fimfiction taking into account the given [`Config`].
    /// Where if `config.exec()`:
    /// - Is `None`, passes `story` through [`AsyncRequester::client_download()`].
    /// - Is `Some(exec)`, passes `story` and the present `exec` command through
    /// [`AsyncRequester::exec_download()`].
    pub async fn download(&self, story: &Story) -> errors::Result<()> {
        match self.config.exec.as_ref() {
            Some(exec) => self.exec_download(exec, story).await,
            None => self.client_download(story).await,
        }
    }
}
