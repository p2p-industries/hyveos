use clap::{Parser, Subcommand};
use crate::families::{kv, pubsub, inspect, reqres};

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Families,
}

#[derive(Subcommand)]
pub enum Families {
    #[command(subcommand, about = "Key-Value Store")]
    KV(kv::Kv),
    #[command(subcommand, about = "Publisher Subscriber Service")]
    PubSub(pubsub::PubSub),
    #[command(subcommand, about = "Publisher Subscriber Service")]
    Inspect(inspect::Inspect),
    #[command(subcommand, about = "Request-Response Service")]
    ReqRes(reqres::ReqRes),

    #[command(subcommand, about = "Prints the nodes Peer-id")]
    Whoami
}