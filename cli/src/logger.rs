use std::{fmt::Arguments, io::Write};

use console::style;
use log::{Level, LevelFilter};

pub const PROGRESS_PREFIX: &str = "  ";
pub const EXCLUDE_IN_VERBOSE_TARGET: &str = "::excluded_in_verbose";

fn indent_args(args: &Arguments<'_>) -> String {
    args.to_string()
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line.into()
            } else {
                format!("  {}", line)
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn configure(verbose: u64) {
    let mut builder = env_logger::builder();

    if verbose == 0 {
        builder.format(|buf, record| {
            let args = indent_args(record.args());
            match record.level() {
                Level::Error => writeln!(buf, "{}", style(args).red()),
                Level::Warn => writeln!(buf, "{}", style(format_args!("! {}", args)).yellow()),
                _ => writeln!(buf, "{}", args),
            }
        });
    } else {
        builder.format(|buf, record| {
            let log_entry = format!(
                "[{}] [{}] [{}] {}",
                buf.timestamp(),
                record.level(),
                record.target(),
                indent_args(record.args())
            );

            match record.level() {
                Level::Error => writeln!(buf, "{}", style(log_entry).red()),
                Level::Warn => writeln!(buf, "{}", style(log_entry).yellow()),
                Level::Info => writeln!(buf, "{}", log_entry),
                _ => writeln!(buf, "{}", style(log_entry).dim()),
            }
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
