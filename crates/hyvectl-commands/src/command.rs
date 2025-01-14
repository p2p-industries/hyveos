use clap::{Command, CommandFactory, Parser, Subcommand};

use crate::families::{apps, debug, discovery, file, init, kv, pub_sub, reqres, whoami};

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Families,
    /// Output results in JSON format
    #[arg(short, long)]
    pub json: bool,
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

pub fn build_cli() -> Command {
    Cli::command()
}
