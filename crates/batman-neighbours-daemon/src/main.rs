#![warn(clippy::expect_used, clippy::unwrap_used, clippy::pedantic)]

use std::{
    fs::Permissions,
    io,
    os::unix::fs::{self, PermissionsExt},
    path::PathBuf,
    sync::Arc,
};

use batman_neighbours_core::{BatmanNeighbour, BatmanNeighboursServer, Error};
use clap::Parser;
use futures::{future, Future, StreamExt as _, TryStreamExt as _};
#[cfg(target_os = "linux")]
use genetlink::GenetlinkHandle;
#[cfg(target_os = "linux")]
use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_MATCH, NLM_F_REQUEST,
    NLM_F_ROOT,
};
#[cfg(target_os = "linux")]
use netlink_packet_generic::GenlMessage;
use tarpc::{
    context::Context,
    server::{self, Channel as _},
    tokio_serde::formats::Bincode,
};
use tokio::sync::Mutex;

#[cfg(target_os = "linux")]
mod batman;

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "/var/run/batman-neighbours.sock")]
    socket_path: PathBuf,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct BatmanNeighboursServerImpl {
    genetlink_handle: Arc<Mutex<GenetlinkHandle>>,
}

#[cfg(target_os = "linux")]
impl BatmanNeighboursServer for BatmanNeighboursServerImpl {
    async fn get_neighbours(
        self,
        _: Context,
        if_index: u32,
    ) -> Result<Vec<BatmanNeighbour>, Error> {
        let message =
            batman::Message::new_request(batman::MessageRequestCommand::GetNeighbours, if_index);

        let mut header = NetlinkHeader::default();
        header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_ROOT | NLM_F_MATCH;

        let mut message = NetlinkMessage::new(header, GenlMessage::from_payload(message).into());

        message.finalize();

        self.genetlink_handle
            .lock()
            .await
            .request(message)
            .await
            .map_err(|e| Error::FailedToSendRequest(e.to_string()))?
            .map_err(|e| Error::FailedToDecodeResponse(e.to_string()))
            .filter_map(|res| {
                future::ready(
                    res.and_then(|message| match message.payload {
                        NetlinkPayload::InnerMessage(inner) => match inner.payload {
                            batman::Message::Response(batman::MessageResponse {
                                cmd: batman::MessageResponseCommand::Neighbour(neighbour),
                            }) => Ok(Some(neighbour)),
                            _ => Err(Error::ExpectedResponseMessage),
                        },
                        NetlinkPayload::Error(err) => Err(Error::NetlinkError(err.to_string())),
                        _ => Ok(None),
                    })
                    .transpose(),
                )
            })
            .try_collect::<Vec<_>>()
            .await
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let (connection, genetlink_handle, _) = genetlink::new_connection()?;
    tokio::spawn(connection);

    let genetlink_handle = Arc::new(Mutex::new(genetlink_handle));

    if args.socket_path.exists() {
        std::fs::remove_file(&args.socket_path)?;
    }

    let transport =
        tarpc::serde_transport::unix::listen(&args.socket_path, Bincode::default).await?;

    fs::chown(
        &args.socket_path,
        None,
        uzers::get_group_by_name("batman-neighbours").map(|g| g.gid()),
    )?;
    tokio::fs::set_permissions(&args.socket_path, Permissions::from_mode(0o660)).await?;

    transport
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = BatmanNeighboursServerImpl {
                genetlink_handle: genetlink_handle.clone(),
            };

            channel.execute(server.serve()).for_each(spawn)
        })
        .buffer_unordered(10)
        .for_each(future::ready)
        .await;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[tokio::main]
async fn main() {
    println!("This program is only supported on Linux");
}
