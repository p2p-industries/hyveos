use std::env::args;

use libp2p::{
    gossipsub::IdentTopic,
    identity::Keypair,
    kad::{
        BootstrapError, BootstrapOk, GetProvidersError, GetProvidersOk, GetRecordError,
        GetRecordOk, RecordKey,
    },
};
use p2p::{gossipsub::ReceivedMessage, FullActor};
use rustyline::{error::ReadlineError, hint::Hinter, history::DefaultHistory, Editor};
use tokio::time::Instant;
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;

mod p2p;

#[derive(rustyline::Completer, rustyline::Helper, rustyline::Validator, rustyline::Highlighter)]
struct DiyHinter {
    tree: patricia_tree::StringPatriciaMap<CommandHint>,
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl rustyline::hint::Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> Self {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.to_string(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for DiyHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        self.tree
            .iter_prefix(line)
            .filter_map(|(_, hint)| {
                if hint.display.starts_with(line) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}

fn diy_hints() -> DiyHinter {
    let mut tree = patricia_tree::StringPatriciaMap::new();
    tree.insert("KAD HELP", CommandHint::new("KAD HELP", "KAD HELP"));
    tree.insert(
        "KAD PUT key value",
        CommandHint::new("KAD PUT key value", "KAD PUT"),
    );
    tree.insert("KAD GET key", CommandHint::new("KAD GET key", "KAD GET"));
    tree.insert("KAD BOOT", CommandHint::new("KAD BOOT", "KAD BOOT"));
    tree.insert(
        "KAD PROVIDE key",
        CommandHint::new("KAD PROVIDE key", "KAD PROVIDE"),
    );
    tree.insert(
        "KAD PROVIDERS key",
        CommandHint::new("KAD PROVIDERS key", "KAD PROVIDERS"),
    );
    tree.insert("GOS HELP", CommandHint::new("GOS HELP", "GOS HELP"));
    tree.insert("GOS PING", CommandHint::new("GOS PING", "GOS PING"));
    tree.insert(
        "GOS PUB topic message",
        CommandHint::new("GOS PUB topic message", "GOS PUB"),
    );
    tree.insert(
        "GOS SUB topic",
        CommandHint::new("GOS SUB topic", "GOS SUB"),
    );
    tree.insert("PING", CommandHint::new("PING", "PING"));
    DiyHinter { tree }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let (client, mut actor) = FullActor::build(Keypair::generate_ed25519());

    actor.setup();

    tokio::spawn(async move {
        actor.drive().await;
    });

    let gos = client.gossipsub();

    let topic_handle = gos.get_topic(IdentTopic::new("PING"));

    let mut res = topic_handle.subscribe().await.expect("Failed to subscribe");
    let round_trip = client.round_trip();
    tokio::spawn(async move {
        loop {
            match res.recv().await {
                Ok(ReceivedMessage {
                    propagation_source,
                    message_id,
                    message,
                }) => {
                    if let Some(source) = message.source {
                        if let Ok(nonce_data) = message.data.try_into() {
                            let nonce = u64::from_le_bytes(nonce_data);
                            tracing::debug!("Received pong message from {source} via {propagation_source} and message id: {message_id}");
                            round_trip.report_round_trip(source, nonce).await;
                        }
                    } else {
                        println!("Received pong message from unkonwn source");
                        continue;
                    }
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    });

    let mut builder = rustyline::Config::builder();
    if args().any(|arg| &arg == "--vim" || &arg == "-v") {
        builder = builder.edit_mode(rustyline::EditMode::Vi);
    }

    let h = diy_hints();

    let mut rl: Editor<DiyHinter, DefaultHistory> = Editor::with_config(builder.build())?;
    rl.set_helper(Some(h));

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                if line.to_uppercase().starts_with("KAD") {
                    let kad = client.kad();
                    let split = line
                        .split_whitespace()
                        .enumerate()
                        .map(|(idx, word)| {
                            if idx == 0 || idx == 1 {
                                word.to_uppercase()
                            } else {
                                word.to_string()
                            }
                        })
                        .collect::<Vec<String>>();
                    let inner_split: Vec<&str> = split.iter().map(|e| e.as_str()).collect();
                    match &inner_split[..] {
                        ["KAD", "HELP"] => {
                            help_message(&[
                                ("KAD PUT key value", "Put a record"),
                                ("KAD GET key", "Get a record"),
                                ("KAD BOOT", "Bootstrap the DHT"),
                                ("KAD PROVIDE key", "Start providing a record"),
                                ("KAD PROVIDERS key", "Get providers of a record"),
                            ]);
                        }
                        ["KAD", "PUT", key, value] => {
                            let res = kad
                                .put_record(
                                    RecordKey::new(&key),
                                    value.as_bytes().to_vec(),
                                    None,
                                    libp2p::kad::Quorum::One,
                                )
                                .await;
                            println!("Put Result: {res:?}");
                        }
                        ["KAD", "GET", key] => {
                            let res: Result<Vec<GetRecordOk>, GetRecordError> = kad
                                .get_record(RecordKey::new(&key))
                                .await
                                .expect("Failed to issue command")
                                .collect()
                                .await;
                            println!("Get Result: {res:?}");
                        }
                        ["KAD", "BOOT"] => {
                            let res: Result<Vec<BootstrapOk>, BootstrapError> = kad
                                .bootstrap()
                                .await
                                .expect("Failed to issue command")
                                .collect()
                                .await;
                            println!("Bootstrap Result: {res:?}");
                        }
                        ["KAD", "PROVIDE", key] => {
                            let res = kad.start_providing(RecordKey::new(&key)).await;
                            println!("Provide Result: {res:?}");
                        }
                        ["KAD", "PROVIDERS", key] => {
                            let res: Result<Vec<GetProvidersOk>, GetProvidersError> = kad
                                .get_providers(RecordKey::new(&key))
                                .await
                                .unwrap()
                                .collect()
                                .await;
                            println!("Providers Result: {res:?}");
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                } else if line.to_uppercase().starts_with("GOS") {
                    let gos = client.gossipsub();
                    let split = line
                        .split_whitespace()
                        .enumerate()
                        .map(|(idx, word)| {
                            if idx == 0 || idx == 1 {
                                word.to_uppercase()
                            } else {
                                word.to_string()
                            }
                        })
                        .collect::<Vec<String>>();
                    let inner_split: Vec<&str> = split.iter().map(|e| e.as_str()).collect();
                    match &inner_split[..] {
                        ["GOS", "HELP"] => {
                            help_message(&[
                                ("GOS PUB topic message", "Publish a message"),
                                ("GOS SUB topic", "Subscribe to a topic"),
                                ("GOS PING", "Ping the network"),
                            ]);
                        }
                        ["GOS", "PING"] => {
                            let topic_handle = gos.get_topic(IdentTopic::new("PING"));
                            let nonce = rand::random();
                            let round_trip = client.round_trip();
                            let mut recv = round_trip.register(nonce).await;
                            let start = Instant::now();
                            let res = topic_handle.publish(nonce.to_le_bytes().to_vec()).await;
                            println!("Ping Send Result: {res:?}");
                            if res.is_err() {
                                continue;
                            }
                            tokio::spawn(async move {
                                loop {
                                    let msg = recv.recv().await.expect("Failed to receive");
                                    let from = msg.from;
                                    println!(
                                        "Round trip to {from} ({nonce}) took {:?}",
                                        start.elapsed()
                                    );
                                }
                            });
                        }
                        ["GOS", "PUB", topic, message] => {
                            let topic_handle = gos.get_topic(IdentTopic::new(*topic));
                            let res = topic_handle.publish(message.as_bytes().to_vec()).await;
                            println!("Publish Result: {res:?}");
                        }
                        ["GOS", "SUB", topic] => {
                            let topic_handle = gos.get_topic(IdentTopic::new(*topic));
                            let mut res =
                                topic_handle.subscribe().await.expect("Failed to subscribe");
                            tokio::spawn(async move {
                                loop {
                                    match res.recv().await {
                                        Ok(msg) => {
                                            println!("Received: {:?}", msg);
                                        }
                                        Err(err) => {
                                            println!("Error: {:?}", err);
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                } else if line.to_uppercase().trim() == "PING" {
                    let ping = client.ping();
                    if let Ok(mut sub) = ping.subscribe().await {
                        tokio::spawn(async move {
                            while let Ok(event) = sub.recv().await {
                                let peer = event.peer;
                                if let Ok(dur) = event.result {
                                    println!("Ping to {peer} took {dur:?}");
                                } else {
                                    println!("Ping to {peer} failed");
                                }
                            }
                        });
                    } else {
                        println!("Failed to subscribe");
                    }
                } else {
                    println!("Invalid command");
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn help_message(explains: &[(&str, &str)]) {
    let cmd_width_max = explains.iter().map(|(cmd, _)| cmd.len()).max().unwrap_or(0) + 5;
    let explains_width_max = explains
        .iter()
        .map(|(_, explain)| explain.len())
        .max()
        .unwrap_or(0)
        + 5;
    println!("Available commands:");
    for (cmd, explain) in explains {
        println!("{cmd:<cmd_width_max$}: {explain:>explains_width_max$}");
    }
}
