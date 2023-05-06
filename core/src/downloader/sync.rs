use std::process::Stdio;

use futures_util::StreamExt;
use reqwest;
use tokio::{fs, io, process::Command};
use url::Url;

use crate::config::Config;
use crate::errors::{self, TrackerError};
use crate::story::{Id, Story};
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
        let chunk = chunk.map_err(TrackerError::request)?;

        total_bytes += chunk.len();
        progress.download_progress(total_bytes, &filepath);

        io::copy(&mut chunk.as_ref(), &mut dest)
            .await
            .map_err(TrackerError::io)?;
    }

    Ok(())
}

fn split_str_to_args(command: &str, story: &Story, config: &Config) -> errors::Result<Vec<String>> {
    shlex::split(command)
        .ok_or_else(|| TrackerError::custom("failed to split command into arguments"))
        .map(|args| {
            args.iter()
                .map(|arg| env_with_command_context(arg, story, config).into_owned())
                .collect::<Vec<String>>()
        })
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
/// let story = requester.get_story_response(196256).await?;
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

    /// Requests the [`StoryResponse`] of the given Fimfiction story ID.
    pub async fn get_story_response(&self, id: Id) -> errors::Result<StoryResponse> {
        let url = Url::parse_with_params(
            "https://www.fimfiction.net/api/story.php",
            &[("story", id.to_string())],
        )
        .expect("Fimficiton API URL parse failed");

        let json = self
            .client
            .get(url)
            .send()
            .await
            .map_err(TrackerError::request)?
            .text()
            .await
            .map_err(|err| {
                TrackerError::request(err)
                    .context("couldn't decode the Fimfiction API response body")
            })?;

        fimfiction_api::from_str(&json)
            .map_err(|err| TrackerError::unexpected_response(err, id, json))
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
            .map_err(|err| TrackerError::request(err).context("failed to start story download"))?;

        let dest = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filepath)
            .await
            .map_err(|err| {
                TrackerError::io(err)
                    .context(format!("failed to create file `{}`", filepath.display()))
            })?;

        download(res, dest, filepath.display(), &self.progress)
            .await
            .map_err(|err| {
                err.context(format!(
                    "failed to download story to `{}`",
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
        let args = split_str_to_args(command.as_ref(), story, &self.config)
            .map_err(|err| err.context("exec command should mimic a POSIX shell command"))?;

        let mut command = Command::new(&args[0]);
        if args.len() > 1 {
            command.args(&args[1..]);
        }

        if self.config.quiet {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }

        self.progress.before_execute_command(story);

        let status = command.status().await.map_err(|err| {
            TrackerError::io(err).context(format!("failed to execute command: {:?}", &args))
        })?;

        if !status.success() {
            let err = match status.code() {
                Some(code) => TrackerError::custom(format!(
                    "command process exited with status code {}",
                    code
                )),
                None => TrackerError::custom("command process was terminated by signal"),
            };

            return Err(err.context(format!("failed executing command: {:?}", &args)));
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

#[cfg(test)]
mod test {
    use super::*;

    use chrono::{TimeZone, Utc};

    use crate::config::ConfigBuilder;
    use crate::StoryStatus;

    #[test]
    fn argument_split() {
        let story = Story {
            id: 0,
            title: "A \"Story\" Title".into(),
            author: "An \"Author\"".into(),
            chapter_count: 10,
            words: 77_446,
            update_datetime: Utc
                .with_ymd_and_hms(2018, 3, 18, 13, 42, 7)
                .single()
                .expect("DateTime should be valid and with a single result"),
            status: StoryStatus::Hiatus,
        };

        let config: Config = ConfigBuilder::new()
            .download_dir("/path/to/download-dir")
            .tracker_file("/path/to/tracker-file.json")
            .into();

        macro_rules! assert_args {
            ($command:literal, $expect:expr) => {
                assert_eq!(
                    split_str_to_args($command, &story, &config)
                        .expect("command should be properly defined"),
                    $expect
                );
            };
        }

        assert_args!(
            "wget -O $DOWNLOAD_DIR/$TITLE.$FORMAT $DOWNLOAD_URL",
            &[
                "wget",
                "-O",
                "/path/to/download-dir/A _Story_ Title.html",
                "https://www.fimfiction.net/story/download/0/html",
            ]
        );

        assert_args!(
            "fimfic2epub --dir $DOWNLOAD_DIR $ID",
            &["fimfic2epub", "--dir", "/path/to/download-dir", "0"]
        );

        assert_args!(
            "fanficfare --format=$FORMAT --non-interactive \
            --option output_filename=\"$DOWNLOAD_DIR/$${title}-$${siteabbrev}_$${storyId}$${formatext}\" \
            $URL",
            &[
                "fanficfare", 
                "--format=html", 
                "--non-interactive", 
                "--option",
                "output_filename=/path/to/download-dir/${title}-${siteabbrev}_${storyId}${formatext}",
                "https://www.fimfiction.net/story/0",
            ]
        );
    }
}
