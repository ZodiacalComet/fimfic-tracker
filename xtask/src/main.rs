use std::{fs::DirBuilder, path::Path};

use clap::CommandFactory;
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

fn main() -> Result<()> {
    let mut args = std::env::args();

    match args.nth(1).as_deref() {
        Some("completions") => generate_completions(),
        Some(cmd) => bail!("unrecognized command: {}", cmd),
        None => bail!("command argument is required"),
    }
}
