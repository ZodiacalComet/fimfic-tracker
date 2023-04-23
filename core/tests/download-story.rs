// TODO: Example or test?
use tempdir::TempDir;

use fimfic_tracker::{downloader::ProgressListener, Config, ConfigBuilder, Id, Story};

// "The Moon's Apprentice" by Forthwith
static STORY_ID: Id = 196256;

struct SimpleListener;

impl ProgressListener for SimpleListener {
    fn download_progress(&self, bytes: usize, filepath: &str) {
        println!(
            "[Download] {} ({}) (started? {})",
            filepath,
            bytes,
            bytes == 0
        );
    }

    fn successfull_client_download(&self, story: &Story) {
        println!("Download of `{}` finished!", &story.title);
    }

    fn before_execute_command(&self, _story: &Story) {}
    fn successfull_command_execution(&self, _story: &Story) {}
}

#[test]
fn test_blocking_download() {
    use fimfic_tracker::downloader::BlockingRequester as Requester;

    let tmp_dir = TempDir::new("fft-blocking-download").expect("failed to create temp dir");
    let config: Config = ConfigBuilder::new()
        .download_dir(tmp_dir.path().to_string_lossy())
        .into();

    let requester = Requester::new(config, SimpleListener);

    let story: Story = requester
        .get_story_response(STORY_ID)
        .expect("failed to request story response")
        .into();
    println!("{:?}", story);

    requester
        .client_download(&story)
        .expect("failed to download story with client");
}

#[tokio::test]
async fn test_sync_download() {
    use fimfic_tracker::downloader::AsyncRequester as Requester;

    let tmp_dir = TempDir::new("fft-sync-download").expect("failed to create temp dir");
    let config: Config = ConfigBuilder::new()
        .download_dir(tmp_dir.path().to_string_lossy())
        .into();

    let requester = Requester::new(config, SimpleListener);

    let story: Story = requester
        .get_story_response(STORY_ID)
        .await
        .expect("failed to request story response")
        .into();
    println!("{:?}", story);

    requester
        .client_download(&story)
        .await
        .expect("failed to download story with client");
}
