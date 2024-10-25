use std::ops::{Deref, DerefMut};

use libp2p::{
    swarm::{NetworkBehaviour, NewExternalAddrCandidate, NewListenAddr},
    Multiaddr,
};
use tracing::instrument;

pub struct Behaviour<B> {
    inner: B,
    whitelist: Option<Multiaddr>,
}

impl<B> Deref for Behaviour<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<B> DerefMut for Behaviour<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<B> Behaviour<B> {
    pub fn new(inner: B) -> Self {
        Behaviour {
            inner,
            whitelist: None,
        }
    }

    pub fn with_whitelist(&mut self, whitelist: Multiaddr) {
        self.whitelist = Some(whitelist);
    }
}

impl<B> NetworkBehaviour for Behaviour<B>
where
    B: NetworkBehaviour,
{
    type ConnectionHandler = B::ConnectionHandler;

    type ToSwarm = B::ToSwarm;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        self.inner.handle_established_inbound_connection(
            connection_id,
            peer,
            local_addr,
            remote_addr,
        )
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        addr: &Multiaddr,
        role_override: libp2p::core::Endpoint,
        port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        self.inner.handle_established_outbound_connection(
            connection_id,
            peer,
            addr,
            role_override,
            port_use,
        )
    }

    #[instrument(skip(self))]
    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        match &event {
            libp2p::swarm::FromSwarm::NewListenAddr(NewListenAddr { addr, .. })
            | libp2p::swarm::FromSwarm::NewExternalAddrCandidate(NewExternalAddrCandidate {
                addr,
            }) => {
                if let Some(whitelist) = &self.whitelist {
                    if whitelist != *addr {
                        tracing::debug!(?addr, ?whitelist, "ignoring address not in whitelist");
                        return;
                    }
                }
            }
            _ => {}
        }
        self.inner.on_swarm_event(event)
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: libp2p::PeerId,
        connection_id: libp2p::swarm::ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        self.inner
            .on_connection_handler_event(peer_id, connection_id, event)
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>>
    {
        self.inner.poll(cx)
    }
}
