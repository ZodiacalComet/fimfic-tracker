use crate::story::Story;

/// Listener for requesters in the download progress.
pub trait ProgressListener {
    /// Executed for each chunk of bytes that is written into `filepath`, where `bytes` is the
    /// total amount of bytes downloaded. It **always** start at `0`.
    ///
    /// On `client_download` method.
    fn download_progress(&self, bytes: usize, filepath: &str);
    /// Executed once the download of a story has finished.
    ///
    /// On `client_download` method.
    fn successfull_client_download(&self, story: &Story);
    /// Executed just before the execution of a command.
    ///
    /// On `exec_download` method.
    fn before_execute_command(&self, story: &Story);
    /// Executed once the command finishes its execution successfully.
    ///
    /// On `exec_download` method.
    fn successfull_command_execution(&self, story: &Story);
}

/// A [`ProgressListener`] implementation that does nothing.
pub struct SilentListener;

impl ProgressListener for SilentListener {
    fn download_progress(&self, _bytes: usize, _filepath: &str) {}
    fn successfull_client_download(&self, _story: &Story) {}
    fn before_execute_command(&self, _story: &Story) {}
    fn successfull_command_execution(&self, _story: &Story) {}
}
