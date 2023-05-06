use std::{fs::DirBuilder, path::Path};

use clap::{CommandFactory, Parser};
use clap_complete::{generate_to, Shell};
use eyre::{bail, Result, WrapErr};

#[allow(unused)]
mod args;

use args::Args;

macro_rules! generate {
    ([$($kind:ident),+], $app:expr, $name:expr, $out_dir:expr) => {
        $(
            eprintln!("Generate completions for {} ...", stringify!($kind));
            generate_to(Shell::$kind, &mut $app, &$name, &$out_dir)
                .wrap_err_with(|| format!("failed to generate completions for {}", stringify!($kind)))?;
        )+
    };
}

fn generate_completions() -> Result<()> {
    let output_dir = Path::new("completions");

    if !output_dir.is_dir() {
        DirBuilder::new()
            .create(output_dir)
            .wrap_err_with(|| format!("failed to crate directory to `{}`", output_dir.display()))?;
    }

    let mut app = Args::command();
    let name = app.get_name().to_string();
    generate!([Bash, Elvish, Fish, PowerShell, Zsh], app, name, output_dir);

    eprintln!("Saved in `{}`", output_dir.display());

    Ok(())
}

fn format_error_with_context(name: &str, lines: &[&str], start: usize, end: usize, message: String) {
    eprintln!("Error :: {}:{} :: {}", start + 1, end, name);
    eprintln!(" :: {}", message);
    eprintln!(
        "{}",
        lines[start..end]
            .iter()
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n")
    );
}

fn check_readme() -> Result<()> {
    let file = Path::new("README.md");
    let content = std::fs::read_to_string(&file)
        .wrap_err_with(|| format!("failed to read file `{}`", file.display()))?;

    let lines = content.lines().collect::<Vec<&str>>();
    let mut iter = lines.iter().cloned().enumerate();

    let prog_name = Args::command().get_name().to_string();

    let mut failed = false;
    'line_parse: loop {
        let Some((_, line)) = iter.next() else {
            break;
        };

        if !line.starts_with("<!-- CHECK: ") && !line.ends_with(" -->") {
            continue;
        }

        let name = {
            let Some(start) = line.find(":") else { continue; };
            &line[start + 1..].trim_end_matches(&['-', '>']).trim()
        };

        let Some((line_start, line)) = iter.next() else { break; };
        if line != "```sh" {
            continue;
        }

        let mut code_block_lines: Vec<&str> = Vec::new();
        let mut line_end = line_start + 2;
        for (_, line) in iter.by_ref().take_while(|(_, line)| line != &"```") {
            code_block_lines.push(line);
            line_end += 1;
        }

        let mut args: Vec<String> = Vec::new();
        for line in &code_block_lines {
            let cmd = line.trim_end_matches('\\').trim();
            match shlex::split(cmd) {
                Some(cmd_args) => args.extend_from_slice(&cmd_args),
                None => {
                    format_error_with_context(
                        name,
                        &lines,
                        line_start,
                        line_end,
                        format!("Failed to split command into arguments: {:?}", cmd),
                    );
                    failed = true;
                    continue 'line_parse;
                }
            };
        }

        if &args[0] != &prog_name {
            format_error_with_context(
                name,
                &lines,
                line_start,
                line_end,
                format!(
                    "Expected {} as first argument, got {:?}",
                    &prog_name, &args[0]
                ),
            );
            continue;
        }

        if let Err(error) = Args::try_parse_from(&args[0..]) {
            format_error_with_context(
                name,
                &lines,
                line_start,
                line_end,
                format!("Failed to parse command"),
            );
            eprintln!("{}", error.render().ansi());
            failed = true;
            continue;
        }

        eprintln!("Success :: {}:{} :: {}", line_start + 1, line_end, name);
    }

    if failed {
        eprintln!();
        bail!("check for README has failed");
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args();

    match args.nth(1).as_deref() {
        Some("completions") => generate_completions(),
        Some("check-readme") => check_readme(),
        Some(cmd) => bail!("unrecognized command: {}", cmd),
        None => bail!("command argument is required"),
    }
}
