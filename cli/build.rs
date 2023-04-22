use std::env;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/args.rs");

macro_rules! generate {
    ([$($kind:ident),+], $app:expr, $name:expr, $out_dir:expr) => {
        $(
            generate_to(Shell::$kind, &mut $app, &$name, &$out_dir)
                .expect(&format!("failed to generate completions for {}", stringify!($kind)));
        )+
    };
}

fn main() {
    println!("cargo:rerun-if-changed=src/args.rs");

    let out_dir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let mut app = Args::command();
    let name = app.get_name().to_string();
    generate!([Bash, Elvish, Fish, PowerShell, Zsh], app, name, out_dir);
}
