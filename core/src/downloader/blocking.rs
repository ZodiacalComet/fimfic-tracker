use tokio::runtime::Runtime;

use crate::config::Config;
use crate::errors;
use crate::story::{Id, Story};
use crate::StoryResponse;

use super::listener::ProgressListener;
use super::sync::AsyncRequester;

/// A blocking story downloader.
///
/// Makes use of an [`AsyncRequester`] under the hood, so reference its documentation in each
/// method.
///
/// ```no_run
/// # use fimfic_tracker::Result;
/// # fn main() -> Result<()> {
/// # use fimfic_tracker::{Config, StoryData};
/// use fimfic_tracker::downloader::{BlockingRequester, SilentListener};
/// # let config = Config::default();
///
/// let requester = BlockingRequester::new(config, SilentListener {});
///
/// // Requesting "The Moon's Apprentice" by Forthwith
/// let story = requester.get_story_response(196256)?;
/// println!("{:?}", story);
///
/// // Download story according to the configuration file.
/// requester.download(&story.into())?;
/// # Ok(())
/// # }
/// ```
pub struct BlockingRequester<P>
where
    P: ProgressListener,
{
    inner: AsyncRequester<P>,
    rt: Runtime,
}

impl<P> BlockingRequester<P>
where
    P: ProgressListener,
{
    /// Constructs a new [`BlockingRequester`].
    pub fn new(config: Config, progress: P) -> Self {
        BlockingRequester {
            inner: AsyncRequester::new(config, progress),
            rt: Runtime::new().unwrap(),
        }
    }

    /// Executes [`AsyncRequester::get_story_response()`] on a synchronous context.
    pub fn get_story_response(&self, id: Id) -> errors::Result<StoryResponse> {
        self.rt
            .block_on(async { self.inner.get_story_response(id).await })
    }

    /// Executes [`AsyncRequester::client_download()`] on a synchronous context.
    pub fn client_download(&self, story: &Story) -> errors::Result<()> {
        self.rt
            .block_on(async { self.inner.client_download(story).await })
    }

    /// Executes [`AsyncRequester::exec_download()`] on a synchronous context.
    pub fn exec_download<S>(&self, command: S, story: &Story) -> errors::Result<()>
    where
        S: AsRef<str>,
    {
        self.rt
            .block_on(async { self.inner.exec_download(command, story).await })
    }

    /// Executes [`AsyncRequester::download()`] on a synchronous context.
    pub fn download(&self, story: &Story) -> errors::Result<()> {
        self.rt.block_on(async { self.inner.download(story).await })
    }
}
