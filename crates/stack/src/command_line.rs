use std::path::{Path, PathBuf};

use p2p_stack::Client;
use rustyline::{error::ReadlineError, hint::Hinter, history::DefaultHistory, Editor};

use crate::{printer::SharedPrinter, scripting::ScriptingClient};

pub async fn prep_interaction(
    store_directory: &Path,
    vim: bool,
) -> anyhow::Result<(Editor<FamilyHinter, DefaultHistory>, SharedPrinter, PathBuf)> {
    let mut builder = rustyline::Config::builder();
    if vim {
        builder = builder.edit_mode(rustyline::EditMode::Vi);
    }

    let h = FamilyHinter::all();

    let mut rl = Editor::with_config(builder.build())?;
    rl.set_helper(Some(h));

    let rl_printer = SharedPrinter::from(rl.create_external_printer()?);

    let history_file = store_directory.join("history.txt");

    if history_file.exists() {
        rl.load_history(&history_file)?;
    } else {
        tokio::fs::File::create(&history_file).await?;
    }
    Ok((rl, rl_printer, history_file))
}

pub async fn interaction_loop(
    mut rl: Editor<FamilyHinter, DefaultHistory>,
    rl_printer: SharedPrinter,
    history_file: PathBuf,
    client: Client,
    scripting_client: &ScriptingClient,
) -> anyhow::Result<()> {
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str())?;

                for family in ALL_FAMILIES {
                    if line.starts_with(family.family_prefix()) {
                        let input = line.split_whitespace().collect::<Vec<_>>();
                        let command = family.all_commands();
                        if command.is_command(&input) {
                            match command
                                .execute(&input, &client, &rl_printer, scripting_client)
                                .await
                            {
                                Ok(()) => {}
                                Err(err) => {
                                    println!("Error: {err}");
                                }
                            }
                            break;
                        }
                        println!("Invalid command");
                        family.print_help_message();
                        break;
                    }
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
                println!("Error: {err:?}");
                break;
            }
        }
    }
    rl.save_history(&history_file)?;
    Ok(())
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub struct CommandHint {
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

#[derive(
    Default, rustyline::Completer, rustyline::Helper, rustyline::Validator, rustyline::Highlighter,
)]
pub struct FamilyHinter {
    tree: patricia_tree::StringPatriciaMap<CommandHint>,
}

impl FamilyHinter {
    fn all() -> Self {
        let mut this = Self::default();
        for family in ALL_FAMILIES {
            this.add_family(family);
        }
        this.add_quit();
        this
    }

    fn add_family(&mut self, family: &impl CommandFamily) {
        for HelpMessage {
            completion_prefix,
            example_use,
            ..
        } in family.command().help_message()
        {
            let complete_up_to = if completion_prefix == example_use {
                completion_prefix.to_string()
            } else {
                format!("{completion_prefix} ")
            };
            self.tree.insert(
                completion_prefix,
                CommandHint::new(example_use, &complete_up_to),
            );
        }
    }

    fn add_quit(&mut self) {
        self.tree.insert("QUIT", CommandHint::new("QUIT", "QUIT"));
    }
}

impl Hinter for FamilyHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let line = line.to_uppercase();

        let line_str = line.as_str();

        let hint = self.tree.iter_prefix(line_str).find_map(|(_, hint)| {
            if hint.display.starts_with(line_str) {
                Some(hint.suffix(pos))
            } else {
                None
            }
        });

        hint
    }
}

pub(crate) trait CommandFamily {
    type Command: Command;
    fn name(&self) -> &'static str;
    fn family_prefix(&self) -> &'static str;
    fn command(&self) -> Self::Command;

    fn all_commands(&self) -> (&Self, Self::Command) {
        (self, self.command())
    }

    fn print_help_message(&self) {
        println!("{} Commands:", self.name());
        let max_example_len = self
            .command()
            .help_message()
            .map(|m| m.example_use.len())
            .max()
            .unwrap_or(0);
        let max_description_len = self
            .command()
            .help_message()
            .map(|m| m.description.len())
            .max()
            .unwrap_or(0);

        println!(
            "  {:<max_example_len$}  {:>max_description_len$}  ",
            "Example", "Description"
        );
        for message in self.command().help_message() {
            println!(
                "  {:<max_example_len$}  {:>max_description_len$}  ",
                message.example_use, message.description,
            );
        }
    }
}

pub(crate) struct HelpMessage {
    pub(crate) completion_prefix: &'static str,
    pub(crate) example_use: &'static str,
    pub(crate) description: &'static str,
}

pub(crate) trait Command {
    fn is_command(&self, input: &[&str]) -> bool;
    fn help_message(&self) -> impl Iterator<Item = HelpMessage>;
    async fn execute(
        &self,
        input: &[&str],
        client: &Client,
        rl_printer: &SharedPrinter,
        scripting_client: &ScriptingClient,
    ) -> anyhow::Result<()>;
}

const fn trim(s: &str) -> &str {
    macro_rules! matches_space {
        ($b:ident) => {
            matches!($b, b'\t' | b'\n' | b'\r' | b' ')
        };
    }

    const fn bytes_trim(this: &[u8]) -> &[u8] {
        bytes_trim_start(bytes_trim_end(this))
    }

    const fn bytes_trim_start(mut this: &[u8]) -> &[u8] {
        loop {
            match this {
                [b, rem @ ..] if matches_space!(b) => this = rem,
                _ => return this,
            }
        }
    }

    const fn bytes_trim_end(mut this: &[u8]) -> &[u8] {
        loop {
            match this {
                [rem @ .., b] if matches_space!(b) => this = rem,
                _ => return this,
            }
        }
    }

    unsafe { core::str::from_utf8_unchecked(bytes_trim(s.as_bytes())) }
}

macro_rules! impl_tuple {
    (
        $($idx:tt $t:tt),+
    ) => {
        impl<$($t,)+> Command for ($($t,)+)
        where
            $($t: Command,)+
        {

            fn is_command(&self, input: &[&str]) -> bool {
                $(
                    if self.$idx.is_command(input) {
                        return true;
                    }
                )+
                false
            }

            fn help_message(&self) -> impl Iterator<Item = HelpMessage> {
                let iter = std::iter::empty();
                $(
                    let iter = iter.chain(self.$idx.help_message());
                )+
                iter
            }

            async fn execute(
                &self,
                input: &[&str],
                client: &Client,
                rl_printer: &SharedPrinter,
                scripting_client: &ScriptingClient,
            ) -> anyhow::Result<()> {
                $(
                    if self.$idx.is_command(input) {
                        return self.$idx.execute(input, client, rl_printer, scripting_client).await;
                    }
                )+
                unreachable!()
            }
        }
    };
}

impl_tuple!(0 A);
impl_tuple!(0 A, 1 B);
impl_tuple!(0 A, 1 B, 2 C);
impl_tuple!(0 A, 1 B, 2 C, 3 D);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);

impl<T> Command for &T
where
    T: CommandFamily,
{
    fn is_command(&self, input: &[&str]) -> bool {
        matches!(input, [family, "HELP", ..] if family.to_uppercase() == self.family_prefix())
    }

    fn help_message(&self) -> impl Iterator<Item = HelpMessage> {
        std::iter::empty()
    }

    async fn execute(
        &self,
        _input: &[&str],
        _client: &Client,
        _rl_printer: &SharedPrinter,
        _scripting_client: &ScriptingClient,
    ) -> anyhow::Result<()> {
        self.print_help_message();
        Ok(())
    }
}

macro_rules! match_input {
    (
        $input:ident,
        [
        $(
            $label:literal
        ),*
        ],
        [
        $($p:pat),*
        ]
    ) => {{
        use std::borrow::Cow;
        const LABELS: usize = {
            let labels = 0;
            $(
                let labels = labels + 1;
                let _ = $label;
            )*
            labels
        };
        const PATTERN: usize = {
            let pattern = 0;
            $(
                let pattern = pattern + 1;
                stringify!($p);
            )*
            pattern
        };
        const LENGTH: usize = LABELS + PATTERN;
        if $input.len() != LENGTH {
            return false;
        }
        let upper_cased: smallvec::SmallVec<[Cow<'_, str>; LENGTH]> = $input
            .iter()
            .take(LABELS)
            .map(|s| Cow::from(s.to_uppercase()))
            .chain($input.iter().skip(LABELS).map(|e| Cow::Borrowed(*e)))
            .collect();
        let borrowed: smallvec::SmallVec<[&str; LENGTH]> = upper_cased.iter().map(|s| s.as_ref()).collect();
        #[allow(unused_attributes)]
        #[allow(unused_variables)]
        {
            matches!(&borrowed[..], [$($label),* $(, $p)*])
        }
    }};
}

macro_rules! extract_parameters {
    (
        $input:ident,
        $(
            $label:literal
        ),*
    ) => {{
        let mut iter = $input.iter();
        $(
            let m = iter.next().unwrap();
            if m.to_uppercase() != $label {
                return Err(anyhow::anyhow!("Invalid command. Expected {}, got {}", $label, m));
            }
        )*
        std::array::from_fn(|_| iter.next().unwrap())
    }};
}

macro_rules! command_instance {
    (
        $t:ident,
        $client:ident,
        $rl_printer:ident,
        $scripting_client:ident,
        $family_prefix:literal,
        [
        $(
            $label:literal
        ),*
        ],
        [
            $($p:pat),*
        ],
        $description:literal,
        $execute:expr
    ) => {
        #[derive(Debug, Default, Clone, Copy)]
        pub(crate) struct $t;

        impl Command for $t {
            fn is_command(&self, input: &[&str]) -> bool {
                match_input!(input, [$family_prefix, $($label),*], [$($p),*])
            }

            fn help_message(&self) -> impl Iterator<Item = HelpMessage> {
                const COMPLETION_PREFIX: &str = trim(concat!($family_prefix, " ", $($label, " "),*));
                const EXAMPLE_USE: &str = trim(concat!($family_prefix, " ", concat!($($label, " "),*), concat!($(stringify!($p), " "),*),));

                std::iter::once(HelpMessage {
                    completion_prefix: COMPLETION_PREFIX,
                    example_use: EXAMPLE_USE,
                    description: $description,
                })
            }

            #[allow(unused_variables)]
            async fn execute(
                &self,
                input: &[&str],
                $client: &Client,
                $rl_printer: &SharedPrinter,
                $scripting_client: &ScriptingClient,
            ) -> anyhow::Result<()> {
                debug_assert!(self.is_command(input));
                let [$( $p),*] = extract_parameters!(input, $family_prefix, $($label),*);
                $execute
            }
        }
    };
}

macro_rules! command_family {
    (
        $t:ident,
        $name:literal,
        $client:ident,
        $rl_printer:ident,
        $scripting_client:ident,
        $family_prefix:literal,
      [
            $(
                (
                    $command:ident,
                    $description:literal,
                    |
                        $(
                            $label:literal
                        ),*
                        $(,
                        $(
                            $p:ident
                        ),+
                        )?
                    | $execute:block
                )
            ),*
        ]
    ) => {
        $(
            command_instance!($command, $client, $rl_printer, $scripting_client, $family_prefix, [$($label),*], [$($($p),*)?], $description, $execute);
        )*

        #[derive(Debug, Default, Clone, Copy)]
        pub(crate) struct $t;

        impl CommandFamily for $t {
            type Command = ($($command,)+);

            fn name(&self) -> &'static str {
                $name
            }

            fn family_prefix(&self) -> &'static str {
                $family_prefix
            }

            fn command(&self) -> Self::Command {
                ($($command,)+)
            }
        }
    };
}

pub(super) mod prelude {
    pub(super) use std::fmt::Write;

    pub(super) use futures::stream::TryStreamExt;
    pub(super) use p2p_stack::Client;

    pub(super) use super::{trim, Command, CommandFamily, HelpMessage};
    pub(super) use crate::{scripting::ScriptingClient, SharedPrinter};
}

macro_rules! special_family {
    (
        $(
            $(#[$attr:meta])*
            $family:ident($commands:path)
        ),+
    ) => {
        const ALL_FAMILIES: &[SpecialFamily] = &[
            $(
                $(#[$attr])*
                SpecialFamily::$family($commands),
            )+
        ];

        enum SpecialFamily {
            $(
                $(#[$attr])*
                $family($commands),
            )+
        }

        impl CommandFamily for SpecialFamily {
            type Command = SpecialCommand;

            fn name(&self) -> &'static str {
                match self {
                    $(
                        $(#[$attr])*
                        SpecialFamily::$family(c) => c.name(),
                    )+
                }
            }

            fn family_prefix(&self) -> &'static str {
                match self {
                    $(
                        $(#[$attr])*
                        SpecialFamily::$family(c) => c.family_prefix(),
                    )+
                }
            }

            fn command(&self) -> Self::Command {
                match self {
                    $(
                        $(#[$attr])*
                        SpecialFamily::$family(c) => SpecialCommand::$family(c.command()),
                    )+
                }
            }
        }

        enum SpecialCommand {
            $(
                $(#[$attr])*
                $family(<$commands as CommandFamily>::Command),
            )+
        }

        impl Command for SpecialCommand {
            fn is_command(&self, input: &[&str]) -> bool {
                match self {
                    $(
                        $(#[$attr])*
                        SpecialCommand::$family(c) => c.is_command(input),
                    )+
                }
            }

            fn help_message(&self) -> impl Iterator<Item = HelpMessage> {
                let ret: Box<dyn Iterator<Item = HelpMessage>> = match self {
                    $(
                        $(#[$attr])*
                        SpecialCommand::$family(c) => Box::new(c.help_message()),
                    )+
                };
                ret
            }

            async fn execute(
                &self,
                input: &[&str],
                client: &Client,
                rl_printer: &SharedPrinter,
                scripting_client: &ScriptingClient,
            ) -> anyhow::Result<()> {
                match self {
                    $(
                        $(#[$attr])*
                        SpecialCommand::$family(c) => c.execute(input, client, rl_printer, scripting_client).await,
                    )+
                }
            }
        }
    };
}

special_family! {
    Kad(kad::KadCommands),
    GossipSub(gos::GossipSubCommands),
    Ping(ping::PingCommands),
    #[cfg(feature = "batman")]
    Neigh(neigh::NeighCommands),
    Script(script::ScriptCommands),
    Me(me::MeCommands),
    File(file::FileCommands)
}

pub mod kad {
    use libp2p::kad::{
        BootstrapError, BootstrapOk, GetProvidersError, GetProvidersOk, GetRecordError,
        GetRecordOk, Quorum, RecordKey,
    };

    use super::prelude::*;

    command_family!(
        KadCommands,
        "Kademlia",
        client,
        _rl_printer,
        _scripting_client,
        "KAD",
        [
            (
                KadPutCommand,
                "Publish the record for the given key",
                |"PUT", key, value| {
                    let res = client
                        .kad()
                        .put_record(
                            RecordKey::new(key),
                            value.as_bytes().to_vec(),
                            None,
                            Quorum::One,
                        )
                        .await?;
                    println!("Put Result: {res:?}");
                    Ok(())
                }
            ),
            (
                KadGetCommand,
                "Get the record for the given key",
                |"GET", key| {
                    let res: Result<Vec<GetRecordOk>, GetRecordError> = client
                        .kad()
                        .get_record(RecordKey::new(&key))
                        .await?
                        .try_collect()
                        .await;
                    println!("Get Result: {res:?}");
                    Ok(())
                }
            ),
            (
                KadBootstrapCommand,
                "Bootstrap the Kademlia DHT",
                |"BOOT"| {
                    let res: Result<Vec<BootstrapOk>, BootstrapError> =
                        client.kad().bootstrap().await?.try_collect().await;
                    println!("Bootstrap Result: {res:?}");
                    Ok(())
                }
            ),
            (
                KadProvideCommand,
                "Announce that this node can provide the value for the given key",
                |"PROVIDE", key| {
                    let res = client.kad().start_providing(RecordKey::new(&key)).await?;
                    println!("Provide Result: {res:?}");
                    Ok(())
                }
            ),
            (
                KadGetProvidersCommand,
                "Get the providers for the given key",
                |"PROVIDERS", key| {
                    let res: Result<Vec<GetProvidersOk>, GetProvidersError> = client
                        .kad()
                        .get_providers(RecordKey::new(&key))
                        .await?
                        .try_collect()
                        .await;
                    println!("Get Providers Result: {res:?}");
                    res?;
                    Ok(())
                }
            )
        ]
    );

    #[cfg(test)]
    mod tests {
        use crate::command_line::Command;

        #[test]
        fn test_kad_put_command() {
            let input = ["KAD", "PUT", "key", "value"];
            assert!(super::KadPutCommand.is_command(&input));
            let m = super::KadPutCommand
                .help_message()
                .next()
                .unwrap()
                .example_use;
            assert_eq!(m, "KAD PUT key value");
        }

        #[test]
        fn test_kad_get_command() {
            let input = ["KAD", "GET", "key"];
            assert!(super::KadGetCommand.is_command(&input));
            let m = super::KadGetCommand
                .help_message()
                .next()
                .unwrap()
                .example_use;
            assert_eq!(m, "KAD GET key");
        }

        #[test]
        fn test_kad_bootstrap_command() {
            let input = ["KAD", "BOOT"];
            assert!(super::KadBootstrapCommand.is_command(&input));
            let m = super::KadBootstrapCommand
                .help_message()
                .next()
                .unwrap()
                .example_use;
            assert_eq!(m, "KAD BOOT");
        }

        #[test]
        fn test_kad_provide_command() {
            let input = ["KAD", "PROVIDE", "key"];
            assert!(super::KadProvideCommand.is_command(&input));
            let m = super::KadProvideCommand
                .help_message()
                .next()
                .unwrap()
                .example_use;
            assert_eq!(m, "KAD PROVIDE key");
        }

        #[test]
        fn test_kad_get_providers_command() {
            let input = ["KAD", "PROVIDERS", "key"];
            assert!(super::KadGetProvidersCommand.is_command(&input));
            let m = super::KadGetProvidersCommand
                .help_message()
                .next()
                .unwrap()
                .example_use;
            assert_eq!(m, "KAD PROVIDERS key");
        }
    }
}

pub mod gos {
    use std::time::Instant;

    use libp2p::gossipsub::IdentTopic;

    use super::prelude::*;

    command_family!(
        GossipSubCommands,
        "GossipSub",
        client,
        rl_printer,
        _scripting_client,
        "GOS",
        [
            (GosPingCommand, "Respond to ping messages", |"PING"| {
                let topic_handle = client.gossipsub().get_topic(IdentTopic::new("PING"));
                let nonce = rand::random();
                let round_trip = client.round_trip();
                let mut recv = round_trip.register(nonce).await;
                let start = Instant::now();
                let res = topic_handle.publish(nonce.to_le_bytes().to_vec()).await;
                println!("Ping Send Result: {res:?}");
                res?;

                let local_printer = rl_printer.clone();
                tokio::spawn(async move {
                    let mut printer = local_printer;

                    loop {
                        let msg = recv.recv().await.expect("Failed to receive");
                        let from = msg.from;
                        writeln!(
                            printer,
                            "Round trip to {from} ({nonce}) took {:?}",
                            start.elapsed()
                        )
                        .expect("Failed to print");
                    }
                });
                Ok(())
            }),
            (
                GosPubCommand,
                "Publish a message to a given topic",
                |"PUB", topic, message| {
                    let topic_handle = client.gossipsub().get_topic(IdentTopic::new(*topic));
                    let res = topic_handle.publish(message.as_bytes().to_vec()).await;
                    println!("Publish Result: {res:?}");
                    res?;
                    Ok(())
                }
            ),
            (
                GosSubCommand,
                "Subscribe to a given topic",
                |"SUB", topic| {
                    let topic_handle = client.gossipsub().get_topic(IdentTopic::new(*topic));
                    let mut res = topic_handle.subscribe().await.expect("Failed to subscribe");

                    let local_printer = rl_printer.clone();
                    tokio::spawn(async move {
                        let mut printer = local_printer;

                        loop {
                            match res.recv().await {
                                Ok(msg) => {
                                    writeln!(printer, "Received: {msg:?}")
                                        .expect("Failed to print");
                                }
                                Err(err) => {
                                    writeln!(printer, "Error: {err:?}").expect("Failed to print");
                                    break;
                                }
                            }
                        }
                    });
                    Ok(())
                }
            )
        ]
    );
}

pub mod ping {
    use super::prelude::*;

    command_family!(
        PingCommands,
        "Ping",
        client,
        rl_printer,
        _scripting_client,
        "PING",
        [(PingCommand, "Ping the network", |"PING"| {
            let mut sub = client.ping().subscribe().await?;
            let local_printer = rl_printer.clone();
            tokio::spawn(async move {
                let mut printer = local_printer;

                while let Ok(event) = sub.recv().await {
                    let peer = event.peer;
                    if let Ok(dur) = event.result {
                        writeln!(printer, "Ping to {peer} took {dur:?}").expect("Failed to print");
                    } else {
                        writeln!(printer, "Ping to {peer} failed").expect("Failed to print");
                    }
                }
            });
            Ok(())
        })]
    );
}

#[cfg(feature = "batman")]
pub mod neigh {
    use std::sync::Arc;

    use p2p_stack::NeighbourEvent;
    use tokio::sync::broadcast::Receiver;

    use super::prelude::*;

    #[cfg(feature = "batman")]
    async fn neighbours_task(mut sub: Receiver<Arc<NeighbourEvent>>, mut printer: SharedPrinter) {
        while let Ok(event) = sub.recv().await {
            match event.as_ref() {
                NeighbourEvent::ResolvedNeighbour(neighbour) => {
                    writeln!(
                        printer,
                        "Resolved neighbour {}: {}",
                        neighbour.peer_id, neighbour.direct_addr
                    )
                    .expect("Failed to print");
                }
                NeighbourEvent::LostNeighbour(neighbour) => {
                    writeln!(
                        printer,
                        "Lost neighbour {}: {}",
                        neighbour.peer_id, neighbour.direct_addr
                    )
                    .expect("Failed to print");
                }
            }
        }
    }

    command_family!(
        NeighCommands,
        "Neighbourhood",
        client,
        rl_printer,
        _scripting_client,
        "NEIGH",
        [
            (
                NeighSubCommand,
                "Subscribe to neighbourhood events",
                |"SUB"| {
                    let sub = client.neighbours().subscribe().await?;
                    tokio::spawn(neighbours_task(sub, rl_printer.clone()));
                    Ok(())
                }
            ),
            (NeighListCommand, "List resolved neighbours", |"LIST"| {
                let neighbours = client.neighbours().get_resolved().await?;
                if neighbours.is_empty() {
                    println!("No resolved neighbours");
                } else {
                    println!("Resolved neighbours:");
                    for (peer_id, neighbours) in neighbours {
                        if let [neighbour] = &neighbours[..] {
                            println!("  Peer {peer_id}: {}", neighbour.direct_addr);
                        } else {
                            println!("  Peer {peer_id}:");
                            for neighbour in neighbours {
                                println!("    - {}", neighbour.direct_addr);
                            }
                        }
                    }
                }
                Ok(())
            }),
            (
                NeighUnresolvedCommand,
                "List unsresolved neighbours",
                |"LIST", "UNRESOLVED"| {
                    let neighbours = client.neighbours().get_unresolved().await?;
                    if neighbours.is_empty() {
                        println!("No unresolved neighbours");
                    } else {
                        println!("Unresolved neighbours:");
                        for neighbour in neighbours {
                            println!("  - {}", neighbour.mac);
                        }
                    }
                    Ok(())
                }
            )
        ]
    );
}

mod script {
    use std::num::ParseIntError;

    use libp2p::PeerId;

    use super::prelude::*;

    fn parse_ports(ports: &str) -> Result<Vec<u16>, ParseIntError> {
        ports
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
    }

    command_family!(
        ScriptCommands,
        "Scripting",
        client,
        rl_printer,
        scripting_client,
        "SCRIPT",
        [
            (
                ScriptDeploySelfCommand,
                "Deploy a docker image locally from the docker registry",
                |"DEPLOY", "SELF", image, ports| {
                    let ports = parse_ports(ports)?;
                    let ulid = scripting_client
                        .self_deploy_image(image, false, true, ports)
                        .await?;
                    println!("Self-deployed image {image} with ULID: {ulid}");
                    Ok(())
                }
            ),
            (
                ScriptDeployRemoteCommand,
                "Deploy a docker image to a remote peer from the docker registry",
                |"DEPLOY", peer_id, image, ports| {
                    let peer: PeerId = peer_id.parse()?;
                    let ports = parse_ports(ports)?;
                    let ulid = scripting_client
                        .deploy_image(image, false, peer, true, ports)
                        .await?;
                    println!("Deployed image {image} to {peer} with ULID: {ulid}");
                    Ok(())
                }
            ),
            (
                ScriptLocalDeploySelfCommand,
                "Deploy a local docker image to a remote peer",
                |"LOCAL", "DEPLOY", "SELF", image, ports| {
                    let ports = parse_ports(ports)?;
                    let ulid = scripting_client
                        .self_deploy_image(image, true, true, ports)
                        .await?;
                    println!("Self-deployed local image {image} with ULID: {ulid}");
                    Ok(())
                }
            ),
            (
                ScriptLocalDeployRemoteCommand,
                "Deploy a local docker image to a remote peer",
                |"LOCAL", "DEPLOY", peer_id, image, ports| {
                    let peer: PeerId = peer_id.parse()?;
                    let ports = parse_ports(ports)?;
                    let ulid = scripting_client
                        .deploy_image(image, true, peer, true, ports)
                        .await?;
                    println!("Deployed local image {image} to {peer} with ULID: {ulid}");
                    Ok(())
                }
            ),
            (
                ScriptListCommand,
                "List running scripts",
                |"LIST", peer_id| {
                    let peer_id = if peer_id.to_uppercase() == "SELF" {
                        None
                    } else {
                        Some(peer_id.parse()?)
                    };
                    let scripts = scripting_client.list_containers(peer_id).await?;
                    if scripts.is_empty() {
                        println!("No running scripts");
                    } else {
                        println!("Running scripts:");
                        for (ulid, script) in scripts {
                            println!("  {ulid}: {script}");
                        }
                    }
                    Ok(())
                }
            ),
            (
                ScriptStopAllCommand,
                "Stop all containers",
                |"STOP", "ALL", peer_id| {
                    let peer_id = if peer_id.to_uppercase() == "SELF" {
                        None
                    } else {
                        Some(peer_id.parse()?)
                    };
                    scripting_client.stop_all_containers(false, peer_id).await?;
                    println!("Stopped all containers");
                    Ok(())
                }
            ),
            (
                ScriptStopOneCommand,
                "Stop a container",
                |"STOP", peer_id, id| {
                    let peer_id = if peer_id.to_uppercase() == "SELF" {
                        None
                    } else {
                        Some(peer_id.parse()?)
                    };
                    let ulid = id.parse()?;
                    scripting_client.stop_container(ulid, peer_id).await?;
                    println!("Stopped container {ulid}");
                    Ok(())
                }
            )
        ]
    );
}

pub mod me {
    use super::prelude::*;

    command_family!(
        MeCommands,
        "Me",
        client,
        _rl_printer,
        _scripting_client,
        "ME",
        [(MeIdCommand, "Print the local peer ID", |"ID"| {
            println!("Peer ID: {}", client.peer_id());
            #[cfg(feature = "clipboard")]
            {
                arboard::Clipboard::new()?.set_text(client.peer_id().to_string())?;
            }
            Ok(())
        })]
    );
}

pub mod file {
    use std::path::PathBuf;

    use base64_simd::{Out, URL_SAFE};
    use p2p_stack::file_transfer::Cid;

    use super::prelude::*;

    command_family!(
        FileCommands,
        "File Transfer",
        client,
        _rl_printer,
        _scripting_client,
        "FILE",
        [
            (FileListCommand, "List files", |"LIST"| {
                let files = client.file_transfer().list().await?;
                let all_files = files.try_collect::<Vec<_>>().await?;
                if all_files.is_empty() {
                    println!("No files");
                } else {
                    println!("Files:");
                    for Cid { hash, id } in all_files {
                        let hash_base = URL_SAFE.encode_to_string(&hash[..]);
                        println!("-  {id}: {hash_base}");
                    }
                }
                Ok(())
            }),
            (FileImportCommand, "Import a file", |"IMPORT", path| {
                let path = path.parse::<PathBuf>()?;
                let Cid { id, hash } = client
                    .file_transfer()
                    .import_new_file(path.as_path().canonicalize()?.as_path())
                    .await?;
                let hash_base = URL_SAFE.encode_to_string(&hash[..]);
                println!("Imported file with CID {id}: {hash_base}");
                #[cfg(feature = "clipboard")]
                {
                    arboard::Clipboard::new()?.set_text(format!("{id} {hash_base}"))?;
                }

                Ok(())
            }),
            (
                FileGetCommand,
                "Retrieve a file",
                |"GET", id, hash, path| {
                    let cid_from_parts = || {
                        let id = id.parse().ok()?;
                        let mut hash_bytes = [0u8; 32];
                        let out = Out::from_slice(&mut hash_bytes[..]);
                        let hash_out = URL_SAFE.decode(hash.as_bytes(), out).ok()?;
                        if hash_out.len() == 32 {
                            Some(Cid {
                                id,
                                hash: hash_bytes,
                            })
                        } else {
                            None
                        }
                    };
                    let cid = cid_from_parts().ok_or_else(|| anyhow::anyhow!("Invalid CID"))?;
                    let cid_path = client.file_transfer().get_cid(cid).await?;
                    tokio::fs::copy(cid_path, path).await?;
                    println!("Retrieved file to {path}");
                    Ok(())
                }
            )
        ]
    );
}
