use clap::{Parser, Subcommand, Command, CommandFactory};
use crate::families::{kv, pubsub, inspect, reqres, file, whoami, hyve};

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Families,
    /// Output results in JSON format
    #[arg(long)]
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
}

pub fn build_cli() -> Command {
    Cli::command()
}