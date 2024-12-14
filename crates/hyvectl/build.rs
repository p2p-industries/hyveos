use std::{
    env::var_os,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use clap::CommandFactory;
use clap_complete::{
    aot::{Bash, Fish, Zsh},
    generate_to, Generator,
};

fn generate_completion(out_dir: &Path, g: impl Generator) -> std::io::Result<()> {
    let mut cmd = hyvectl_commands::command::Cli::command();
    let path = generate_to(g, &mut cmd, "hyvectl", out_dir)?;
    println!("cargo:warning=completion file is generated: {path:?}");
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../hyvectl-commands/");
    let out_dir = PathBuf::from(var_os("OUT_DIR").ok_or(ErrorKind::NotFound)?);

    let cmd = hyvectl_commands::command::Cli::command();
    let man = clap_mangen::Man::new(cmd);

    let mut buffer = Vec::new();
    man.render(&mut buffer)?;

    let man_path = out_dir.join("hyvectl.1");

    std::fs::write(&man_path, buffer)?;

    println!("cargo:warning=man page is generated to: {man_path:?}");

    generate_completion(&out_dir, Bash)?;
    generate_completion(&out_dir, Zsh)?;
    generate_completion(&out_dir, Fish)?;

    Ok(())
}
