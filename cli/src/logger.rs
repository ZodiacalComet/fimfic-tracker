use std::io::Write;

use clap::ColorChoice;
use console::style;
use log::{Level, LevelFilter};

pub const PROGRESS_PREFIX: &str = "  ";
pub const EXCLUDE_IN_VERBOSE_TARGET: &str = "::excluded_in_verbose";

pub fn configure(verbose: u8, color_choice: ColorChoice) {
    // By default, `Style` is made to "point" to stdout from `console`'s point of view.
    // This means that we only need to set colors for stdout to effectively affect all styling done
    // in the application.
    match color_choice {
        ColorChoice::Always => console::set_colors_enabled(true),
        ColorChoice::Never => console::set_colors_enabled(false),
        _ => {}
    };

    let mut builder = env_logger::builder();

    if verbose == 0 {
        builder.format(|buf, record| {
            let args = record.args();
            match record.level() {
                Level::Error => writeln!(buf, "{}", style(args).red()),
                Level::Warn => writeln!(buf, "{}", style(format_args!("! {}", args)).yellow()),
                _ => writeln!(buf, "{}", args),
            }
        });
    } else {
        builder.format(|buf, record| {
            let message = record.args().to_string();

            for line in message.lines() {
                let log_entry = format!(
                    "[{}] [{}] [{}] {}",
                    buf.timestamp(),
                    record.level(),
                    record.target(),
                    line
                );

                match record.level() {
                    Level::Error => writeln!(buf, "{}", style(log_entry).red())?,
                    Level::Warn => writeln!(buf, "{}", style(log_entry).yellow())?,
                    Level::Info => writeln!(buf, "{}", log_entry)?,
                    _ => writeln!(buf, "{}", style(log_entry).dim())?,
                }
            }

            Ok(())
        });
    }

    builder
        .filter_level(LevelFilter::Info)
        .filter_module("want", LevelFilter::Off)
        .filter_module("mio", LevelFilter::Off);

    if verbose > 0 {
        builder
            .filter_level(LevelFilter::Debug)
            .filter_module(EXCLUDE_IN_VERBOSE_TARGET, LevelFilter::Off)
            .filter_module("reqwest", LevelFilter::Debug);
    }

    if verbose > 1 {
        builder.filter_level(LevelFilter::Trace);
    }

    builder.init();
}
