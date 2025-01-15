use hyveos_core::{
    apps::RunningApp,
    grpc::{
        apps_client::AppsClient, DeployAppRequest, DockerApp, DockerImage, Empty,
        ListRunningAppsRequest, StopAppRequest,
    },
};
use libp2p_identity::PeerId;
use tonic::transport::Channel;
use ulid::Ulid;

use crate::{connection::Connection, error::Result};

/// Configuration for deploying an application.
///
/// To deploy to self, leave [`Config::target_peer_id`] unset.
///
/// # Example
///
/// ```no_run
/// use hyveos_sdk::{Connection, services::AppConfig};
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = Connection::new().await.unwrap();
/// let mut apps_service = connection.apps();
///
/// let config = AppConfig::new("my-docker-image:latest")
///     .local()
///     .expose_port(8080);
/// let app_id = apps_service.deploy(config).await.unwrap();
///
/// println!("Deployed app with id {app_id}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    pub image: String,
    pub local: bool,
    pub target_peer_id: Option<PeerId>,
    pub exposed_ports: Option<Vec<u16>>,
    pub persistent: bool,
}

impl Config {
    /// Creates a new configuration for deploying an application.
    ///
    /// `image` should be a docker image name, can contain a tag,
    /// e.g. `my-docker-image:latest`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed app with id {app_id}");
    /// # }
    /// ```
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            local: false,
            target_peer_id: None,
            exposed_ports: None,
            persistent: false,
        }
    }

    /// Sets the application to expect a local image which does not need to be
    /// pulled from a docker registry.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed app with id {app_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn local(mut self) -> Self {
        self.local = true;
        self
    }

    /// Sets the deployment target of the application to a specific peer in the network.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::StreamExt as _;
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service
    ///     .get_providers("identification", "example")
    ///     .await
    ///     .unwrap()
    ///     .next()
    ///     .await
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .target(peer_id);
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed app with id {app_id} on {peer_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn target(mut self, peer_id: PeerId) -> Self {
        self.target_peer_id = Some(peer_id);
        self
    }

    /// Adds a port which is exposed by the docker container running the application.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed app with id {app_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn expose_port(mut self, port: u16) -> Self {
        self.exposed_ports.get_or_insert_with(Vec::new).push(port);
        self
    }

    /// Sets the application to be persistent.
    ///
    /// When this is set, the app will be restarted when the runtime is restarted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .persistent();
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed persistent app with id {app_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }
}

/// A handle to the application management service.
///
/// Exposes methods to interact with the application management service,
/// like for deploying and stopping apps on peers in the network.
///
/// # Example
///
/// ```no_run
/// use hyveos_sdk::{Connection, services::AppConfig};
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = Connection::new().await.unwrap();
/// let mut apps_service = connection.apps();
///
/// let config = AppConfig::new("my-docker-image:latest")
///     .local()
///     .expose_port(8080);
/// let app_id = apps_service.deploy(config).await.unwrap();
///
/// println!("Deployed app with id {app_id}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: AppsClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = AppsClient::new(connection.channel.clone());

        Self { client }
    }

    /// Deploys an application in a docker image to a peer in the network.
    ///
    /// Returns the ULID of the deployed application.
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
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed app with id {app_id}");
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn deploy(&mut self, config: Config) -> Result<Ulid> {
        let request = DeployAppRequest {
            app: DockerApp {
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
            persistent: config.persistent,
        };

        self.client
            .deploy(request)
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }

    /// Lists the running apps on a peer in the network.
    ///
    /// To list the running apps on self, set [`target_peer_id`] to `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let apps = apps_service.list_running(None).await.unwrap();
    ///
    /// for app in apps {
    ///     println!("Running app {} ({})", app.id, app.image);
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn list_running(
        &mut self,
        target_peer_id: Option<PeerId>,
    ) -> Result<Vec<RunningApp>> {
        let request = ListRunningAppsRequest {
            peer: target_peer_id.map(Into::into),
        };

        self.client
            .list_running(request)
            .await?
            .into_inner()
            .apps
            .into_iter()
            .map(|app| app.try_into().map_err(Into::into))
            .collect::<Result<_>>()
            .map_err(Into::into)
    }

    /// Stops a running app with an ID on a peer in the network.
    ///
    /// To stop the running app on self, set [`target_peer_id`] to `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    /// let app_id = apps_service.get_own_app_id().await.unwrap();
    ///
    /// let apps = apps_service.list_running(None).await.unwrap();
    ///
    /// for app in apps {
    ///     if app.id != app_id {
    ///         println!("Stopping app {}", app.id);
    ///         apps_service.stop(app.id, None).await.unwrap();
    ///     }
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn stop(&mut self, id: Ulid, target_peer_id: Option<PeerId>) -> Result<()> {
        let request = StopAppRequest {
            id: id.into(),
            peer: target_peer_id.map(Into::into),
        };

        self.client
            .stop(request)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    /// Get the ID of the current app.
    ///
    /// This can only be called from a running app.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    /// let app_id = apps_service.get_own_app_id().await.unwrap();
    ///
    /// println!("App ID: {}", app_id);
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_own_app_id(&mut self) -> Result<Ulid> {
        self.client
            .get_own_app_id(Empty {})
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }
}
