use std::{
    env,
    fs,
    io::{self, Result},
    path::PathBuf,
    ffi::OsStr,
};

use clap_complete::{generate_to, shells::{Shell}, Generator};
use clap_mangen::Man;

fn generate_man_page() -> Result<()> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let cmd = hyvectl_commands::command::build_cli();

    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;

    let file_path = out_dir.join("hyvectl.1");
    fs::write(&file_path, &buffer)?;

    eprintln!("Man page generated at {:?}", file_path);
    Ok(())
}

fn generate_one_completion<G, P>(generator: G, out_dir: P, bin_name: &str)
where
    G: Generator + Copy,
    P: AsRef<OsStr>,
{
    let mut cmd = hyvectl_commands::command::build_cli();
    generate_to(generator, &mut cmd, bin_name, &out_dir)
        .expect("clap completion generation failed");
}

fn generate_all_completions() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let shells = [Shell::Bash, Shell::Elvish, Shell::Fish, Shell::PowerShell, Shell::Zsh];

    for shell in shells {
        generate_one_completion(shell, &out_dir, "hyvectl");
    }

    eprintln!("Completion scripts generated at {:?}", out_dir);
}

fn main() -> io::Result<()> {
    generate_all_completions();
    generate_man_page()?;
    Ok(())
}
