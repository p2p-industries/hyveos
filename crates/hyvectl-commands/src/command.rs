use clap::{Command, CommandFactory, Parser, Subcommand};

use crate::families::{file, hyve, init, inspect, kv, pubsub, reqres, whoami};

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
    /// Publisher-Subscriber Service
    #[command(subcommand)]
    PubSub(pubsub::PubSub),
    /// Network Inspection Service
    #[command(subcommand)]
    Inspect(inspect::Inspect),
    /// Request-Response Service
    #[command(subcommand)]
    ReqRes(reqres::ReqRes),
    /// Distributed Application Service
    #[command(subcommand)]
    Hyve(hyve::Hyve),
    /// File Transfer Service
    #[command(subcommand)]
    File(file::File),
    /// Prints the local Peer-id
    Whoami(whoami::Whoami),
    /// Initialize a new hyveOS instance
    Init(init::Init),
}

pub fn build_cli() -> Command {
    Cli::command()
}
