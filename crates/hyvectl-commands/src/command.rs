use clap::{Parser, Subcommand};
use crate::families::{kv, pubsub};

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
}