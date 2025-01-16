use std::path::PathBuf;

use clap::{Args, Command, CommandFactory, Parser, Subcommand};

use crate::families::{apps, debug, discovery, file, init, kv, pub_sub, reqres, whoami};

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Families,
    #[command(flatten)]
    pub global: GlobalArgs,
}

#[derive(Subcommand)]
pub enum Families {
    /// Key-Value Store
    #[command(subcommand)]
    KV(kv::Kv),
    /// Discovery Service
    #[command(subcommand)]
    Discovery(discovery::Discovery),
    /// Publisher-Subscriber Service
    #[command(subcommand)]
    PubSub(pub_sub::PubSub),
    /// Network Debugging Service
    #[command(subcommand)]
    Debug(debug::Debug),
    /// Request-Response Service
    #[command(subcommand)]
    ReqRes(reqres::ReqRes),
    /// Distributed Application Service
    #[command(subcommand)]
    Apps(apps::Apps),
    /// File Transfer Service
    #[command(subcommand)]
    File(file::File),
    /// Prints the local Peer-id
    Whoami(whoami::Whoami),
    /// Initialize a new hyveOS instance. This should only be used during installation.
    Init(init::Init),
}

#[derive(Args)]
pub struct GlobalArgs {
    /// Output results in JSON format
    #[arg(short, long)]
    pub json: bool,
    // TODO: merge these into an enum, as soon as https://github.com/clap-rs/clap/issues/2621 is resolved
    /// Path to the hyved bridge directory. Mutually exclusive with --bridge-sock, --files-dir, and --bridge-url.
    ///
    /// Expects bridge.sock and a files directory to be present in the given directory.
    #[arg(long)]
    pub bridge_dir: Option<PathBuf>,
    /// Path to the hyved bridge socket. Requires --files-dir to be set. Mutually exclusive with --bridge-dir and --bridge-url.
    #[arg(
        long,
        requires = "files_dir",
        conflicts_with = "bridge_dir",
        conflicts_with = "bridge_url"
    )]
    pub bridge_sock: Option<PathBuf>,
    /// Path to the files directory. Requires --bridge-sock to be set. Mutually exclusive with --bridge-dir and --bridge-url.
    #[arg(
        long,
        requires = "bridge_sock",
        conflicts_with = "bridge_dir",
        conflicts_with = "bridge_url"
    )]
    pub files_dir: Option<PathBuf>,
    /// URL of the hyved network bridge. Mutually exclusive with --bridge-dir, --bridge-sock, and --files-dir.
    #[arg(
        long,
        conflicts_with = "bridge_dir",
        conflicts_with = "bridge_sock",
        conflicts_with = "files_dir"
    )]
    pub bridge_url: Option<String>,
}

pub fn build_cli() -> Command {
    Cli::command()
}
