use std::path::{Path, PathBuf};

use serde::Deserialize;

mod format;
mod sensibility;

use crate::errors::{self, TrackerError};
use crate::utils::{async_read_to_string, default_user_config_file, read_to_string};
pub use format::DownloadFormat;
pub use sensibility::SensibilityLevel;

/// Default prefix for configuration by environment variables.
pub const DEFAULT_ENVIRONMENT_PREFIX: &str = "FFT";

/// Used to construct [`Config`].
///
/// # Example
///
/// ```
/// # use fimfic_tracker::{Config, ConfigBuilder, Result, SensibilityLevel};
/// # fn main() -> Result<()> {
/// // Getting values from a toml configuration file
/// let config_file = ConfigBuilder::from_file("config/test-config.toml")?;
///
/// // Getting values from environment, prefixed with "FFT"
/// let config_env = ConfigBuilder::from_env("FFT")?;
///
/// // Programatically setting values
/// let config = ConfigBuilder::new()
///     .download_dir("~/other/download")
///     .sensibility_level(SensibilityLevel::Anything)
///     .quiet(true);
///
/// // Merging into one config
/// let config_merged = config_file.merge(config_env).merge(config);
///
/// // Constructing Config
/// let config: Config = config_merged.into();
///
/// println!("{:?}", config);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigBuilder {
    download_dir: Option<String>,
    tracker_file: Option<String>,
    download_format: Option<DownloadFormat>,
    download_delay: Option<u64>,
    sensibility_level: Option<SensibilityLevel>,
    exec: Option<String>,
    quiet: Option<bool>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    /// Constructs a new [`ConfigBuilder`] that results in [`Config`] with its default values.
    pub fn new() -> Self {
        ConfigBuilder {
            download_dir: None,
            tracker_file: None,
            download_format: None,
            download_delay: None,
            sensibility_level: None,
            exec: None,
            quiet: None,
        }
    }

    /// Sets the value of `download_dir`.
    pub fn download_dir<T>(mut self, directory: T) -> Self
    where
        T: Into<String>,
    {
        self.download_dir = Some(directory.into());
        self
    }

    /// Sets the value of `tracker_file`.
    pub fn tracker_file<T>(mut self, filename: T) -> Self
    where
        T: Into<String>,
    {
        self.tracker_file = Some(filename.into());
        self
    }

    /// Sets the value of `download_format`.
    pub fn download_format(mut self, format: DownloadFormat) -> Self {
        self.download_format = Some(format);
        self
    }

    /// Sets the value of `download_delay`.
    pub fn download_delay(mut self, delay: u64) -> Self {
        self.download_delay = Some(delay);
        self
    }

    /// Sets the value of `sensibility_level`.
    pub fn sensibility_level(mut self, sensibility: SensibilityLevel) -> Self {
        self.sensibility_level = Some(sensibility);
        self
    }

    /// Sets the value of `exec`.
    pub fn exec<T>(mut self, exec: T) -> Self
    where
        T: Into<String>,
    {
        self.exec = Some(exec.into());
        self
    }

    /// Sets the value of `quiet`.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = Some(quiet);
        self
    }

    /// Brings the non-default values of `other` into `self`, overwriting it.
    pub fn merge(mut self, other: ConfigBuilder) -> Self {
        macro_rules! set {
            ($field:ident) => {
                if let Some(_) = other.$field {
                    self.$field = other.$field;
                }
            };
        }

        set!(download_dir);
        set!(tracker_file);
        set!(download_format);
        set!(download_delay);
        set!(sensibility_level);
        set!(exec);
        set!(quiet);

        self
    }

    /// Constructs a [`ConfigBuilder`] from `filepath`, parsing it as a toml file.
    ///
    /// # Errors
    ///
    /// - If `filepath` doesn't already exist.
    /// - On deserialization errors. Ex: unexpected value types and toml syntax errors.
    pub fn from_file<P>(filepath: P) -> errors::Result<Self>
    where
        P: AsRef<Path>,
    {
        let contents = read_to_string(&filepath)?;

        toml::from_str(&contents)
            .map_err(|err| TrackerError::config_parsing(filepath.as_ref().display(), err))
    }

    /// Asynchronous version of [`ConfigBuilder::from_file()`].
    pub async fn async_from_file<P>(filepath: P) -> errors::Result<Self>
    where
        P: AsRef<Path>,
    {
        let contents = async_read_to_string(&filepath).await?;

        toml::from_str(&contents)
            .map_err(|err| TrackerError::config_parsing(filepath.as_ref().display(), err))
    }

    /// Constructs a [`ConfigBuilder`] from environment variables prefixed with `prefix`.
    ///
    /// # Errors
    ///
    /// On deserialization errors. Ex: unexpected value types.
    pub fn from_env(prefix: &str) -> errors::Result<Self> {
        let prefix = format!("{}_", prefix);

        envy::prefixed(prefix)
            .from_env()
            .map_err(|err| TrackerError::config_parsing("the environment", err))
    }

    /// Constructs a [`ConfigBuilder`] from the merge of [`default_user_config_file()`] and the
    /// environment with the [`DEFAULT_ENVIRONMENT_PREFIX`] prefix.
    ///
    /// # Errors
    ///
    /// They are returned according to [`ConfigBuilder::from_file()`] and
    /// [`ConfigBuilder::from_env()`].
    pub fn from_default_sources() -> errors::Result<Self> {
        let config = match default_user_config_file() {
            Some(config_file) if config_file.is_file() => ConfigBuilder::from_file(config_file)?,
            _ => ConfigBuilder::new(),
        };

        Ok(config.merge(ConfigBuilder::from_env(DEFAULT_ENVIRONMENT_PREFIX)?))
    }

    /// Asynchronous version of [`ConfigBuilder::from_default_sources()`] that makes use of
    /// [`ConfigBuilder::async_from_file()`] instead.
    pub async fn async_from_default_sources() -> errors::Result<Self> {
        let config = match default_user_config_file() {
            Some(config_file) if config_file.is_file() => {
                ConfigBuilder::async_from_file(config_file).await?
            }
            _ => ConfigBuilder::new(),
        };

        Ok(config.merge(ConfigBuilder::from_env(DEFAULT_ENVIRONMENT_PREFIX)?))
    }
}

/// A storing struct for configuration values, meant to be used as a read-only struct.
///
/// It can be constructed from [`ConfigBuilder`].
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Path to the story download directory, expanding tilde into home directory.
    ///
    /// Defaults to directory obtained from
    /// [`dirs_next::download_dir()`](https://docs.rs/dirs-next/2.0.0/dirs_next/fn.download_dir.html).
    pub download_dir: PathBuf,
    /// Path of the tracker file, expanding tilde into home directory.
    ///
    /// Defaults to `fimfic-tracker/track-data.json` inside directory obtained from
    /// [`dirs_next::data_local_dir()`](https://docs.rs/dirs-next/2.0.0/dirs_next/fn.data_local_dir.html).
    pub tracker_file: PathBuf,
    /// The format in which to download the stories.
    ///
    /// Defaults to [`DownloadFormat::HTML`].
    pub download_format: DownloadFormat,
    /// The seconds to wait between each download.
    ///
    /// Defaults to `5`.
    pub download_delay: u64,
    /// The parameters to consider for the conclusion that a story has a relevant update or not.
    ///
    /// Defaults to [`SensibilityLevel::OnlyChapters`].
    pub sensibility_level: SensibilityLevel,
    /// If not `None`, this will be executed as a command in the download process
    /// instead of directly downloading from Fimfiction.
    ///
    /// If [`String`] is empty, it ends up being converted into a `None`.
    ///
    /// Defaults to `None`.
    pub exec: Option<String>,
    /// Whether or not to suppress the output of the command defined in `exec`.
    ///
    /// Defaults to `false`.
    pub quiet: bool,
}

impl From<ConfigBuilder> for Config {
    fn from(c: ConfigBuilder) -> Self {
        Config {
            download_dir: c
                .download_dir
                .map(|s| shellexpand::tilde(&s).into_owned().into())
                .unwrap_or_else(|| {
                    dirs_next::download_dir().expect("failed to get default download dir")
                }),
            tracker_file: c
                .tracker_file
                .map(|s| shellexpand::tilde(&s).into_owned().into())
                .unwrap_or_else(|| {
                    dirs_next::data_local_dir()
                        .map(|p| p.join("fimfic-tracker").join("track-data.json"))
                        .expect("failed to get default tracker file path")
                }),
            download_format: c.download_format.unwrap_or(DownloadFormat::HTML),
            download_delay: c.download_delay.unwrap_or(5),
            sensibility_level: c
                .sensibility_level
                .unwrap_or(SensibilityLevel::OnlyChapters),
            exec: c
                .exec
                .and_then(|cmd| if cmd.is_empty() { None } else { Some(cmd) }),
            quiet: c.quiet.unwrap_or(false),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        ConfigBuilder::new().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    const ENV_PREFIX_TEST: &str = "FFT_TEST";

    #[test]
    #[ignore]
    fn print_default() {
        println!("{:#?}", Config::default());
    }

    macro_rules! assert_config_source {
        (
            [$origin:ident: $config:expr]
            $($field:ident = $value:expr;)+
        ) => {
            let config: Config = ConfigBuilder::$origin($config)?.into();
            let expect: Config = ConfigBuilder::new()
                $(.$field($value))+
                .into();

            assert_eq!(config, expect);
        };
    }

    macro_rules! set_config_vars {
        ($($name:expr => $value:expr),+) => {
            $(
                env::set_var(format!("{}_{}", ENV_PREFIX_TEST, $name), $value);
            )+
        };
    }

    // TODO: Add test for async_from_file

    macro_rules! config_path {
        ($filename:literal) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/config/", $filename)
        };
    }

    #[test]
    fn deserializing_sources() -> errors::Result<()> {
        assert_config_source!(
            [from_file: config_path!("default.toml")]
            download_dir = "~/Downloads";
            tracker_file = "~/.local/share/fimfic-tracker/track-data.json";
        );

        assert_config_source!(
            [from_file: config_path!("test-config.toml")]
            download_dir = "~/some/path/to/dir";
            tracker_file = "~/path/of/file.json";
            download_format = DownloadFormat::EPUB;
            download_delay = 10;
            sensibility_level = SensibilityLevel::IncludeWords;
            exec = "wget -O ${download_dir}/${safe_title} https://www.fimfiction.net/story/download/${id}/${html}";
            quiet = false;
        );

        set_config_vars!(
            "DOWNLOAD_DIR" => "~/some/path/to/dir",
            "TRACKER_FILE" => "~/path/of/file.json",
            "DOWNLOAD_FORMAT" => "txt",
            "DOWNLOAD_DELAY" => "0",
            "SENSIBILITY_LEVEL" => "2",
            "EXEC" => "/path/to/some/script --dir ${download_dir} $id",
            "QUIET" => "false"
        );

        assert_config_source!(
            [from_env: ENV_PREFIX_TEST]
            download_dir = "~/some/path/to/dir";
            tracker_file = "~/path/of/file.json";
            download_format = DownloadFormat::TXT;
            download_delay = 0;
            sensibility_level = SensibilityLevel::Anything;
            exec = "/path/to/some/script --dir ${download_dir} $id";
            quiet = false;
        );

        Ok(())
    }

    #[test]
    fn merging_result() {
        let config = ConfigBuilder::new()
            .download_dir("~/Download")
            .download_format(DownloadFormat::EPUB)
            .download_delay(0)
            .quiet(false);

        let other_config = ConfigBuilder::new()
            .download_dir("/path/to/download")
            .tracker_file("/path/to/tracker-cache.json")
            .sensibility_level(SensibilityLevel::Anything)
            .quiet(true);

        {
            let config: Config = config.clone().merge(other_config.clone()).into();

            let expect: Config = ConfigBuilder::new()
                .download_dir("/path/to/download")
                .tracker_file("/path/to/tracker-cache.json")
                .download_format(DownloadFormat::EPUB)
                .download_delay(0)
                .sensibility_level(SensibilityLevel::Anything)
                .quiet(true)
                .into();

            assert_eq!(config, expect);
        }

        let another_config = ConfigBuilder::new()
            .download_format(DownloadFormat::TXT)
            .download_delay(1)
            .sensibility_level(SensibilityLevel::IncludeWords)
            .exec("/path/to/script $id")
            .quiet(false);

        {
            let config: Config = config.merge(other_config).merge(another_config).into();

            let expect: Config = ConfigBuilder::new()
                .download_dir("/path/to/download")
                .tracker_file("/path/to/tracker-cache.json")
                .download_format(DownloadFormat::TXT)
                .download_delay(1)
                .sensibility_level(SensibilityLevel::IncludeWords)
                .exec("/path/to/script $id")
                .quiet(false)
                .into();

            assert_eq!(config, expect);
        }
    }
}
