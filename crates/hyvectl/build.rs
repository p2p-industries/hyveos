#[cfg(feature = "generate-completions")]
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, Result},
    path::PathBuf,
};

#[cfg(feature = "generate-completions")]
use clap_complete::{generate_to, shells::Shell, Generator};
#[cfg(feature = "generate-completions")]
use clap_mangen::Man;
#[cfg(feature = "generate-completions")]
use flate2::{write::GzEncoder, Compression};

#[cfg(feature = "generate-completions")]
fn generate_man_page(out_dir: PathBuf) -> Result<()> {
    let cmd = hyvectl_commands::command::build_cli();

    let man = Man::new(cmd);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    man.render(&mut encoder)?;

    let buffer = encoder.finish()?;

    let file_path = out_dir.join("hyvectl.1.gz");
    fs::write(&file_path, &buffer)?;

    eprintln!("Man page generated at {file_path:?}");
    Ok(())
}

#[cfg(feature = "generate-completions")]
fn generate_one_completion<G, P>(generator: G, out_dir: P, bin_name: &str) -> Result<PathBuf>
where
    G: Generator + Copy,
    P: AsRef<OsStr>,
{
    let mut cmd = hyvectl_commands::command::build_cli();
    generate_to(generator, &mut cmd, bin_name, &out_dir)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[cfg(feature = "generate-completions")]
fn generate_all_completions(out_dir: impl AsRef<OsStr>) -> Result<()> {
    let shells = [Shell::Bash, Shell::Fish, Shell::Zsh];

    for shell in shells {
        let path = generate_one_completion(shell, &out_dir, "hyvectl")?;
        eprintln!("Completion generated at {path:?}");
    }

    Ok(())
}

#[cfg(feature = "generate-completions")]
fn main() -> Result<()> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let target_triple = env::var("TARGET").expect("TARGET not set");
    let host_triple = env::var("HOST").expect("HOST not set");
    let profile = env::var("PROFILE").expect("PROFILE not set");
    let mut target_dir = PathBuf::from(manifest_dir)
        .join("..")
        .join("..")
        .join("target");

    if target_triple != host_triple {
        target_dir = target_dir.join(target_triple);
    }

    let target_dir = target_dir
        .join(profile)
        .canonicalize()
        .expect("target dir not found");
    let debian_dir = target_dir.join("debian");
    if !debian_dir.exists() {
        fs::create_dir(&debian_dir)?;
    }

    let completions_dir = debian_dir.join("completions");
    if !completions_dir.exists() {
        fs::create_dir(&completions_dir)?;
    }

    generate_all_completions(completions_dir)?;

    let man_page_dir = debian_dir.join("man");
    if !man_page_dir.exists() {
        fs::create_dir(&man_page_dir)?;
    }

    generate_man_page(man_page_dir)?;
    Ok(())
}

#[cfg(not(feature = "generate-completions"))]
fn main() {}
