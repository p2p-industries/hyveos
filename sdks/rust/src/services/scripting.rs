use libp2p_identity::PeerId;
use p2p_industries_core::{
    grpc::{
        scripting_client::ScriptingClient, DeployScriptRequest, DockerImage, DockerScript, Empty,
        ListRunningScriptsRequest, StopScriptRequest,
    },
    scripting::RunningScript,
};
use tonic::transport::Channel;
use ulid::Ulid;

use crate::{connection::P2PConnection, error::Result};

/// Configuration for deploying a script.
///
/// To deploy to self, leave [`Config::target_peer_id`] unset.
///
/// # Example
///
/// ```no_run
/// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = P2PConnection::get().await.unwrap();
/// let mut scripting_service = connection.scripting();
///
/// let config = ScriptingConfig::new("my-docker-image:latest")
///     .local()
///     .expose_port(8080);
/// let script_id = scripting_service.deploy_script(config).await.unwrap();
///
/// println!("Deployed script with id on self: {script_id}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    pub image: String,
    pub local: bool,
    pub target_peer_id: Option<PeerId>,
    pub exposed_ports: Option<Vec<u16>>,
}

impl Config {
    /// Creates a new configuration for deploying a script.
    ///
    /// `image` should be a docker image name, optionally with a tag, e.g.
    /// `my-docker-image:latest`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    ///
    /// let config = ScriptingConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let script_id = scripting_service.deploy_script(config).await.unwrap();
    ///
    /// println!("Deployed script with id on self: {script_id}");
    /// # }
    /// ```
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            local: false,
            target_peer_id: None,
            exposed_ports: None,
        }
    }

    /// Sets the script to expect a local image which does not need to be pulled from a docker
    /// registry.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    ///
    /// let config = ScriptingConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let script_id = scripting_service.deploy_script(config).await.unwrap();
    ///
    /// println!("Deployed script with id on self: {script_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn local(mut self) -> Self {
        self.local = true;
        self
    }

    /// Sets the deployment target of the script to a specific peer in the network.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::StreamExt as _;
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let peer_id = dht_service
    ///     .get_providers("identification", "example")
    ///     .await
    ///     .unwrap()
    ///     .next()
    ///     .await
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// let mut scripting_service = connection.scripting();
    ///
    /// let config = ScriptingConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .target(peer_id);
    /// let script_id = scripting_service.deploy_script(config).await.unwrap();
    ///
    /// println!("Deployed script with id on {peer_id}: {script_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn target(mut self, peer_id: PeerId) -> Self {
        self.target_peer_id = Some(peer_id);
        self
    }

    /// Adds a port which is exposed by the docker container running the script.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    ///
    /// let config = ScriptingConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let script_id = scripting_service.deploy_script(config).await.unwrap();
    ///
    /// println!("Deployed script with id on self: {script_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn expose_port(mut self, port: u16) -> Self {
        self.exposed_ports.get_or_insert_with(Vec::new).push(port);
        self
    }
}

/// A handle to the scripting service.
///
/// Exposes methods to interact with the scripting service, like for deploying and stopping scripts
/// on peers in the network.
///
/// # Example
///
/// ```no_run
/// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = P2PConnection::get().await.unwrap();
/// let mut scripting_service = connection.scripting();
///
/// let config = ScriptingConfig::new("my-docker-image:latest")
///     .local()
///     .expose_port(8080);
/// let script_id = scripting_service.deploy_script(config).await.unwrap();
///
/// println!("Deployed script with id on self: {script_id}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: ScriptingClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &P2PConnection) -> Self {
        let client = ScriptingClient::new(connection.channel.clone());

        Self { client }
    }

    /// Deploys a script in a docker image to a peer in the network and returns the ULID of the
    /// deployed script.
    ///
    /// To deploy to self, leave [`Config::target_peer_id`] unset.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    ///
    /// let config = ScriptingConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let script_id = scripting_service.deploy_script(config).await.unwrap();
    ///
    /// println!("Deployed script with id on self: {script_id}");
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn deploy_script(&mut self, config: Config) -> Result<Ulid> {
        let request = DeployScriptRequest {
            script: DockerScript {
                image: DockerImage { name: config.image },
                ports: config
                    .exposed_ports
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
            local: config.local,
            peer: config.target_peer_id.map(Into::into),
        };

        self.client
            .deploy_script(request)
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }

    /// Lists the running scripts on a peer in the network.
    ///
    /// To list the running scripts on self, set [`Config::target_peer_id`] to `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    ///
    /// let scripts = scripting_service.list_running_scripts(None).await.unwrap();
    ///
    /// for script in scripts {
    ///     println!("Running script {} ({})", script.id, script.image);
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn list_running_scripts(
        &mut self,
        target_peer_id: Option<PeerId>,
    ) -> Result<Vec<RunningScript>> {
        let request = ListRunningScriptsRequest {
            peer: target_peer_id.map(Into::into),
        };

        self.client
            .list_running_scripts(request)
            .await?
            .into_inner()
            .scripts
            .into_iter()
            .map(|script| script.try_into().map_err(Into::into))
            .collect::<Result<_>>()
            .map_err(Into::into)
    }

    /// Stops a running script with an ID on a peer in the network.
    ///
    /// To stop the running script on self, set [`Config::target_peer_id`] to `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    /// let script_id = scripting_service.get_own_id().await.unwrap();
    ///
    /// let scripts = scripting_service.list_running_scripts(None).await.unwrap();
    ///
    /// for script in scripts {
    ///     if script.id != script_id {
    ///         println!("Stopping script {}", script.id);
    ///         scripting_service.stop_script(script.id, None).await.unwrap();
    ///     }
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn stop_script(&mut self, id: Ulid, target_peer_id: Option<PeerId>) -> Result<()> {
        let request = StopScriptRequest {
            id: id.into(),
            peer: target_peer_id.map(Into::into),
        };

        self.client
            .stop_script(request)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    /// Get the ID of the current script.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    /// let script_id = scripting_service.get_own_id().await.unwrap();
    ///
    /// println!("Script ID: {}", script_id);
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_own_id(&mut self) -> Result<Ulid> {
        self.client
            .get_own_id(Empty {})
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }
}
