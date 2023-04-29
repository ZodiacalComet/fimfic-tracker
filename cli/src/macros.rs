#[macro_export]
macro_rules! verbose_disabled {
    () => {
        log_enabled!(
            target: $crate::logger::EXCLUDE_IN_VERBOSE_TARGET,
            log::Level::Info
        )
    };
}

#[macro_export]
macro_rules! clear_last_lines {
    () => {
        clear_last_lines!(1);
    };
    ($amount:expr) => {
        if verbose_disabled!() {
            console::Term::stderr()
                .clear_last_lines($amount)
                .expect("unable to clear last lines in stderr");
        }
    };
}

#[macro_export]
macro_rules! separate {
    () => {
        if verbose_disabled!() {
            eprintln!();
        }
    };
}

#[macro_export]
macro_rules! progress {
    () => {
        info!(target: $crate::logger::EXCLUDE_IN_VERBOSE_TARGET, "");
    };
    ($($arg:tt)+) => {
        info!(target: $crate::logger::EXCLUDE_IN_VERBOSE_TARGET, "{}{}", $crate::logger::PROGRESS_PREFIX, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! progress_or_info {
    ($($arg:tt)+) => {
        if verbose_disabled!() {
            progress!($($arg)+);
        } else {
            info!($($arg)+);
        }
    };
}

#[macro_export]
macro_rules! format_story {
    ($story:expr) => {
        format_args!(
            "{} ({})",
            console::style(&$story.title).green().bold(),
            console::style($story.id).blue()
        )
    };
}

#[macro_export]
macro_rules! format_status {
    ($story:expr) => {
        match $story.status {
            fimfic_tracker::StoryStatus::Complete => console::Style::new().green(),
            fimfic_tracker::StoryStatus::Incomplete => console::Style::new().yellow(),
            fimfic_tracker::StoryStatus::Hiatus => console::Style::new().cyan(),
            fimfic_tracker::StoryStatus::Cancelled => console::Style::new().red(),
        }
        .bold()
        .apply_to($story.status)
    };
}

#[macro_export]
macro_rules! download_delay {
    ($wait:expr, $use_separator:expr, $delay:expr) => {
        if $wait {
            std::thread::sleep($delay);
            if $use_separator {
                separate!();
            }
        }
    };
}
