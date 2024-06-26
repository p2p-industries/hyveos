use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    ops::Not,
    pin::Pin,
    task::{Context, Poll},
};

use bollard::{
    container::{
        AttachContainerOptions, AttachContainerResults, Config, CreateContainerOptions, LogOutput,
        StopContainerOptions,
    },
    errors::Error as BollardError,
    image::{CreateImageOptions, ImportImageOptions},
    secret::{BuildInfo, HostConfig, Mount, MountTypeEnum, PortBinding},
    service::{CreateImageInfo, ProgressDetail},
};
use bytes::Bytes;
use futures::{
    future::{FusedFuture as _, FutureExt, OptionFuture},
    TryStreamExt as _,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_regex::regex;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{
        AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, Empty,
        ReadBuf, Sink,
    },
    sync::oneshot,
    task::JoinHandle,
};
use tokio_stream::StreamExt;

pub type Error = BollardError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Compression {
    None,
    #[cfg(feature = "zstd")]
    Zstd,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
}

impl NetworkMode {
    fn as_str(&self) -> &'static str {
        match self {
            NetworkMode::Bridge => "bridge",
            NetworkMode::Host => "host",
            NetworkMode::None => "none",
        }
    }
}

/// A manager for docker containers
#[derive(Debug, Clone)]
pub struct ContainerManager {
    inner: bollard::Docker,
}

fn new_progress_bar(total: u64) -> ProgressBar {
    ProgressBar::new(total).with_style(
        ProgressStyle::with_template(
            "{msg}: {bar} [{percent_precise}%] Since: {elapsed} ETA: {eta}",
        )
        .unwrap(),
    )
}

impl ContainerManager {
    /// Create a new container manager
    pub fn new() -> Result<Self, BollardError> {
        let docker = bollard::Docker::connect_with_local_defaults()?;
        Ok(Self { inner: docker })
    }

    pub async fn import_image(
        &self,
        root_fs: Bytes,
        compression: Compression,
    ) -> Result<PulledImage<'_>, BollardError> {
        let mut archive = Vec::new();
        match compression {
            Compression::None => FlexDecompressor::None(BufReader::new(&root_fs[..])),
            #[cfg(feature = "zstd")]
            Compression::Zstd => FlexDecompressor::Zstd(
                async_compression::tokio::bufread::ZstdDecoder::new(BufReader::new(&root_fs[..])),
            ),
        }
        .read_to_end(&mut archive)
        .await?;
        let root_fs = Bytes::from(archive);
        let mut events = self
            .inner
            .import_image(ImportImageOptions { quiet: true }, root_fs, None);
        while let Some(ev) = events.next().await {
            match ev {
                Ok(BuildInfo {
                    stream: Some(stream),
                    ..
                }) => {
                    let reg = regex!(r#"^Loaded image: (.+)$"#);
                    if let Some(caps) = reg.captures(stream.trim()) {
                        let id = caps.get(1).unwrap().as_str();
                        return Ok(PulledImage {
                            image: Cow::Owned(id.to_string()),
                            docker: Cow::Borrowed(&self.inner),
                        });
                    }
                }
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "No image loaded").into())
    }

    pub fn get_local_image<'a>(&'a self, image: &'a str) -> PulledImage<'a> {
        PulledImage {
            image: Cow::Borrowed(image),
            docker: Cow::Borrowed(&self.inner),
        }
    }

    /// Pull an image with the given name
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// let manager = docker::ContainerManager::new().unwrap();
    /// let image = manager.pull_image("alpine:latest", false).await.unwrap();
    /// # let _ = image.remove().await;
    /// # }
    ///
    /// ```
    pub async fn pull_image<'a>(
        &'a self,
        image: &'a str,
        verbose: bool,
    ) -> Result<PulledImage<'a>, BollardError> {
        let mut fetching_stream = self.inner.create_image(
            Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            }),
            None,
            None,
        );

        if verbose {
            let progress_bar = MultiProgress::new();

            let mut all_bars = HashMap::new();

            while let Some(event) = fetching_stream.next().await {
                match event {
                    Ok(CreateImageInfo {
                        progress_detail:
                            Some(ProgressDetail {
                                current: Some(current),
                                total: Some(total),
                            }),
                        id: Some(id),
                        status: Some(status),
                        ..
                    }) => {
                        if let (Ok(total), Ok(current)) = (total.try_into(), current.try_into()) {
                            let x = all_bars
                                .entry(id.clone())
                                .or_insert_with(|| progress_bar.add(new_progress_bar(total)));
                            x.set_position(current);
                            x.set_message(format!("{status:<12} {id:>14}"));
                            if current >= total {
                                x.finish();
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                    _ => {}
                }
            }
        } else {
            fetching_stream.try_collect::<Vec<_>>().await?;
        }

        Ok(PulledImage {
            image: Cow::Borrowed(image),
            docker: Cow::Borrowed(&self.inner),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PulledImage<'a> {
    pub image: Cow<'a, str>,
    docker: Cow<'a, bollard::Docker>,
}

impl PulledImage<'_> {
    pub fn into_owned(self) -> PulledImage<'static> {
        PulledImage {
            image: Cow::Owned(self.image.into_owned()),
            docker: Cow::Owned(self.docker.into_owned()),
        }
    }

    pub async fn get_id(&self) -> Result<Option<String>, BollardError> {
        self.docker.inspect_image(&self.image).await.map(|i| i.id)
    }

    /// Remove the image from the docker daemon
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// let manager = docker::ContainerManager::new().unwrap();
    /// let image = manager.pull_image("alpine:latest", false).await.unwrap();
    /// let _ = image.remove().await;
    /// # }
    pub async fn remove(self) -> Result<(), BollardError> {
        self.docker
            .remove_image(&self.image, None, None)
            .await
            .map(|_| ())
    }

    pub async fn export(&self, compression: Compression) -> Result<Bytes, BollardError> {
        let mut exporter = self
            .docker
            .export_image(&self.image)
            .map(|e| e.map_err(|e| BollardError::from(io::Error::new(io::ErrorKind::Other, e))));
        let mut archive = Vec::new();
        let mut compressor = match compression {
            Compression::None => FlexCompressor::None(&mut archive),
            #[cfg(feature = "zstd")]
            Compression::Zstd => FlexCompressor::Zstd(
                async_compression::tokio::write::ZstdEncoder::new(&mut archive),
            ),
        };
        while let Some(m) = exporter.next().await {
            let chunk = m?;
            compressor.write_all(&chunk[..]).await?;
        }
        compressor.flush().await?;
        Ok(Bytes::from(archive))
    }

    /// Create a container from the image
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// let manager = docker::ContainerManager::new().unwrap();
    /// let image = manager.pull_image("alpine:latest", false).await.unwrap();
    /// let container = image.create_container().enable_stream().run().await.unwrap();
    /// # let stopped = container.stop().await.unwrap();
    /// # let image = stopped.remove().await.unwrap();
    /// # let _ = image.remove().await;
    /// # }
    pub fn create_container(&self) -> ContainerBuilder<'_> {
        ContainerBuilder {
            docker: self.docker.clone(),
            image: self.image.clone(),
            cmd: None,
            stdin: None,
            stdout: None,
            stderr: None,
            stream: false,
            volumes: HashMap::new(),
            name: None,
            network_mode: None,
            privileged: None,
            exposed_ports: None,
            env: HashMap::new(),
        }
    }

    pub fn create_dyn_container(
        &self,
    ) -> ContainerBuilder<
        '_,
        Box<dyn AsyncRead + Unpin + Send + Sync + 'static>,
        Box<dyn AsyncWrite + Unpin + Send + Sync + 'static>,
        Box<dyn AsyncWrite + Unpin + Send + Sync + 'static>,
    > {
        ContainerBuilder {
            docker: self.docker.clone(),
            image: self.image.clone(),
            cmd: None,
            stdin: None,
            stdout: None,
            stderr: None,
            stream: false,
            volumes: HashMap::new(),
            name: None,
            network_mode: None,
            privileged: None,
            exposed_ports: None,
            env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerBuilder<'a, In = Empty, Out = Sink, Err = Sink> {
    docker: Cow<'a, bollard::Docker>,
    image: Cow<'a, str>,
    cmd: Option<Vec<String>>,
    stdin: Option<In>,
    stdout: Option<Out>,
    stderr: Option<Err>,
    stream: bool,
    volumes: HashMap<String, String>,
    name: Option<String>,
    network_mode: Option<NetworkMode>,
    privileged: Option<bool>,
    exposed_ports: Option<Vec<(u16, SocketAddr)>>,
    env: HashMap<String, String>,
}

impl<'a, In, Out, Err> ContainerBuilder<'a, In, Out, Err> {
    /// Enable streaming of logs (replaying logs that have happened before attaching)
    pub fn enable_stream(mut self) -> Self {
        self.stream = true;
        self
    }

    pub fn add_volume(mut self, host: impl Into<String>, container: impl Into<String>) -> Self {
        self.volumes.insert(host.into(), container.into());
        self
    }

    pub fn add_volumes(
        mut self,
        volumes: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.volumes
            .extend(volumes.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Set the command to run in the container
    ///
    /// ```
    /// # use tokio::io::AsyncBufReadExt;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let manager = docker::ContainerManager::new().unwrap();
    /// # let image = manager.pull_image("alpine:latest", false).await.unwrap();
    /// let (tx, rx) = tokio::io::duplex(4096);
    /// let container = image.create_container().cmd(vec!["echo", "hello"]).stdout(tx).enable_stream().run().await.unwrap();
    /// let mut rx = tokio::io::BufReader::new(rx).lines();
    /// let line = rx.next_line().await.unwrap().unwrap();
    /// assert_eq!(line, "hello");
    /// # let stopped_container = container.stop().await.unwrap();
    /// # let image = stopped_container.remove().await.unwrap();
    /// # let _ = image.remove().await;
    /// # }
    pub fn cmd(mut self, cmd: Vec<&str>) -> Self {
        self.cmd = Some(cmd.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn stdin<In2: AsyncRead + Unpin + Send + 'static>(
        self,
        stdin: In2,
    ) -> ContainerBuilder<'a, In2, Out, Err> {
        ContainerBuilder {
            docker: self.docker,
            image: self.image,
            cmd: self.cmd,
            stdin: Some(stdin),
            stdout: self.stdout,
            stderr: self.stderr,
            stream: self.stream,
            volumes: self.volumes,
            name: self.name,
            network_mode: self.network_mode,
            privileged: self.privileged,
            exposed_ports: self.exposed_ports,
            env: self.env,
        }
    }

    pub fn stdout<Out2: AsyncWrite + Unpin + Send + 'static>(
        self,
        stdout: Out2,
    ) -> ContainerBuilder<'a, In, Out2, Err> {
        ContainerBuilder {
            docker: self.docker,
            image: self.image,
            cmd: self.cmd,
            stdin: self.stdin,
            stdout: Some(stdout),
            stderr: self.stderr,
            stream: self.stream,
            volumes: self.volumes,
            name: self.name,
            network_mode: self.network_mode,
            privileged: self.privileged,
            exposed_ports: self.exposed_ports,
            env: self.env,
        }
    }

    pub fn stderr<Err2: AsyncWrite + Unpin + Send + 'static>(
        self,
        stderr: Err2,
    ) -> ContainerBuilder<'a, In, Out, Err2> {
        ContainerBuilder {
            docker: self.docker,
            image: self.image,
            cmd: self.cmd,
            stdin: self.stdin,
            stdout: self.stdout,
            stderr: Some(stderr),
            stream: self.stream,
            volumes: self.volumes,
            name: self.name,
            network_mode: self.network_mode,
            privileged: self.privileged,
            exposed_ports: self.exposed_ports,
            env: self.env,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn expose_port(mut self, container_port: u16, host: SocketAddr) -> Self {
        self.exposed_ports
            .get_or_insert_with(Vec::new)
            .push((container_port, host));
        self
    }

    pub fn network_mode(mut self, mode: NetworkMode) -> Self {
        self.network_mode = Some(mode);
        self
    }

    pub fn privileged(mut self, privileged: bool) -> Self {
        self.privileged = Some(privileged);
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

impl<'a, In, Out, Err> ContainerBuilder<'a, In, Out, Err>
where
    In: AsyncRead + Unpin + Send + 'static,
    Out: AsyncWrite + Unpin + Send + 'static,
    Err: AsyncWrite + Unpin + Send + 'static,
{
    /// Run the container
    pub async fn run(self) -> Result<RunningContainer<'a>, BollardError> {
        let ContainerBuilder {
            docker,
            image,
            cmd,
            stdin,
            stdout,
            stderr,
            stream,
            volumes,
            name,
            network_mode,
            privileged,
            exposed_ports,
            env,
        } = self;

        let (port_bindings, exposed_ports) = match exposed_ports.as_deref() {
            Some([]) => (None, None),
            Some(ports) => {
                let mut port_bindings = HashMap::new();
                let mut exposed_ports = HashMap::new();
                for (container, host) in ports {
                    let tcp_bindings = port_bindings
                        .entry(format!("{container}/tcp"))
                        .or_insert(Some(Vec::new()))
                        .as_mut()
                        .unwrap();
                    tcp_bindings.push(PortBinding {
                        host_ip: Some(host.ip().to_string()),
                        host_port: Some(host.port().to_string()),
                    });

                    let udp_bindings = port_bindings
                        .entry(format!("{container}/udp"))
                        .or_insert(Some(Vec::new()))
                        .as_mut()
                        .unwrap();
                    udp_bindings.push(PortBinding {
                        host_ip: Some(host.ip().to_string()),
                        host_port: Some(host.port().to_string()),
                    });

                    exposed_ports.insert(format!("{}/tcp", host.port()), Default::default());
                    exposed_ports.insert(format!("{}/udp", host.port()), Default::default());
                }
                (Some(port_bindings), Some(exposed_ports))
            }
            None => {
                let exposed_ports = docker
                    .inspect_image(&image)
                    .await?
                    .config
                    .and_then(|c| c.exposed_ports);

                if let Some(exposed_ports) = &exposed_ports {
                    println!("Found exposed ports in image: {exposed_ports:?}");
                }

                let port_bindings = exposed_ports.as_ref().map(|ports| {
                    let mut port_bindings = HashMap::new();

                    for port_name in ports.keys() {
                        if let Some(port) = port_name
                            .strip_suffix("/tcp")
                            .or_else(|| port_name.strip_suffix("/udp"))
                        {
                            let bindings = vec![
                                PortBinding {
                                    host_ip: Some(Ipv4Addr::UNSPECIFIED.to_string()),
                                    host_port: Some(port.to_string()),
                                },
                                PortBinding {
                                    host_ip: Some(Ipv6Addr::UNSPECIFIED.to_string()),
                                    host_port: Some(port.to_string()),
                                },
                            ];

                            port_bindings.insert(port_name.clone(), Some(bindings));
                        }
                    }

                    port_bindings
                });

                (port_bindings, exposed_ports)
            }
        };

        let host_config = HostConfig {
            mounts: volumes.is_empty().not().then(|| {
                volumes
                    .into_iter()
                    .map(|(k, v)| Mount {
                        target: Some(v),
                        source: Some(k),
                        typ: Some(MountTypeEnum::BIND),
                        ..Default::default()
                    })
                    .collect()
            }),
            network_mode: network_mode.map(|m| m.as_str().to_string()),
            port_bindings,
            privileged,
            ..Default::default()
        };

        let env = if env.is_empty() {
            None
        } else {
            Some(
                env.into_iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect(),
            )
        };

        let cmds = cmd
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str().to_string()).collect());
        let container_config = Config::<String> {
            image: Some(image.to_string()),
            cmd: cmds,
            attach_stdin: stdin.as_ref().map(|_| true),
            attach_stdout: stdout.as_ref().map(|_| true),
            attach_stderr: stderr.as_ref().map(|_| true),
            host_config: Some(host_config),
            exposed_ports,
            env,
            ..Default::default()
        };

        let id = docker
            .create_container::<String, String>(
                name.as_ref().map(|name| CreateContainerOptions {
                    name: name.clone(),
                    platform: None,
                }),
                container_config,
            )
            .await?
            .id;

        docker.start_container::<&str>(&id, None).await?;

        let AttachContainerResults { mut output, input } = docker
            .attach_container(
                &id,
                Some(AttachContainerOptions::<&str> {
                    stdin: Some(stdin.as_ref().is_some()),
                    stdout: Some(stdout.as_ref().is_some()),
                    stderr: Some(stderr.as_ref().is_some()),
                    stream: Some(stream),
                    logs: Some(stream),
                    ..Default::default()
                }),
            )
            .await?;

        let (input_abort, abort_rx) = oneshot::channel();

        let input_handle = stdin.map(|mut s| async move {
            let mut input = input;
            tokio::select! {
                _ = abort_rx => {
                }
                _ = tokio::io::copy(&mut s, &mut input) => {}
            }
            Ok::<_, BollardError>(())
        });

        let (output_abort, abort_rx) = oneshot::channel();

        let output_handle = tokio::spawn(async move {
            let mut stdout = stdout;
            let mut stderr = stderr;

            let transfer = async move {
                while let Some(m) = output.next().await {
                    match m? {
                        LogOutput::StdOut { message } => {
                            if message.is_empty() {
                                break;
                            }
                            if let Some(out) = stdout.as_mut() {
                                out.write_all(&message[..]).await?;
                                out.flush().await?;
                            }
                        }
                        LogOutput::StdErr { message } => {
                            if message.is_empty() {
                                break;
                            }
                            if let Some(out) = stderr.as_mut() {
                                out.write_all(&message[..]).await?;
                                out.flush().await?;
                            }
                        }
                        _ => {}
                    }
                }
                Ok(())
            };
            tokio::select! {
                _ = abort_rx => {}
                ret = transfer => {
                    return ret;
                }
            }

            Ok(())
        });

        Ok(RunningContainer {
            docker,
            id: Cow::Owned(id),
            image,
            output_handle: Some(output_handle),
            input_abort,
            output_abort,
            input_handle: input_handle.map(tokio::spawn),
        })
    }
}

#[derive(Debug)]
pub struct RunningContainer<'a> {
    docker: Cow<'a, bollard::Docker>,
    pub id: Cow<'a, str>,
    pub image: Cow<'a, str>,
    output_abort: oneshot::Sender<()>,
    output_handle: Option<JoinHandle<Result<(), BollardError>>>,
    input_handle: Option<JoinHandle<Result<(), BollardError>>>,
    input_abort: oneshot::Sender<()>,
}

impl<'a> RunningContainer<'a> {
    pub fn into_owned(self) -> RunningContainer<'static> {
        RunningContainer {
            docker: Cow::Owned(self.docker.into_owned()),
            id: Cow::Owned(self.id.into_owned()),
            image: Cow::Owned(self.image.into_owned()),
            output_abort: self.output_abort,
            output_handle: self.output_handle,
            input_handle: self.input_handle,
            input_abort: self.input_abort,
        }
    }

    async fn inner_stop(mut self, t: Option<i64>) -> Result<StoppedContainer<'a>, BollardError> {
        self.docker
            .stop_container(&self.id, t.map(|t| StopContainerOptions { t }))
            .await?;
        if let Some(mut handle) = self.output_handle.take() {
            let _ = self.output_abort.send(());
            if let Ok(inner) =
                tokio::time::timeout(std::time::Duration::from_secs(1), &mut handle).await
            {
                inner
                    .map_err(|e| BollardError::from(io::Error::new(io::ErrorKind::Other, e)))??;
            } else {
                handle.abort();
            }
        }
        if let Some(mut handle) = self.input_handle.take() {
            let _ = self.input_abort.send(());
            if let Ok(inner) =
                tokio::time::timeout(std::time::Duration::from_secs(1), &mut handle).await
            {
                inner
                    .map_err(|e| BollardError::from(io::Error::new(io::ErrorKind::Other, e)))??;
            } else {
                handle.abort();
            }
        }
        let image = PulledImage {
            image: self.image,
            docker: self.docker,
        };
        Ok(StoppedContainer { id: self.id, image })
    }

    /// Stop the containers
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let manager = docker::ContainerManager::new().unwrap();
    /// # let image = manager.pull_image("alpine:latest", false).await.unwrap();
    /// # let container = image.create_container().enable_stream().run().await.unwrap();
    /// let stopped = container.stop().await.unwrap();
    /// # let image = stopped.remove().await.unwrap();
    /// # let _ = image.remove().await;
    /// # }
    pub async fn stop(self) -> Result<StoppedContainer<'a>, BollardError> {
        self.inner_stop(None).await
    }

    /// Kill the container
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let manager = docker::ContainerManager::new().unwrap();
    /// # let image = manager.pull_image("alpine:latest", false).await.unwrap();
    /// # let container = image.create_container().cmd(vec!["sh", "-c", "sleep 1500"]).run().await.unwrap();
    /// let stopped = container.kill().await.unwrap();
    /// # let image = stopped.remove().await.unwrap();
    /// # let _ = image.remove().await;
    /// # }
    pub async fn kill(self) -> Result<StoppedContainer<'a>, BollardError> {
        self.inner_stop(Some(0)).await
    }

    /// Run the container to completion
    pub async fn run_to_completion(
        mut self,
        stop: Option<oneshot::Receiver<bool>>,
    ) -> Result<StoppedContainer<'a>, BollardError> {
        let mut wait_future = OptionFuture::from(Some(
            self.docker
                .wait_container::<&str>(&self.id, None)
                .try_collect::<Vec<_>>(),
        ));

        let mut output_future =
            OptionFuture::from(self.output_handle.as_mut().map(FutureExt::fuse));
        let mut input_future = OptionFuture::from(self.input_handle.as_mut().map(FutureExt::fuse));
        let mut stop_future = OptionFuture::from(stop);

        loop {
            tokio::select! {
                Some(res) = &mut wait_future => {
                    match res {
                        Ok(_) => {
                            wait_future = None.into();
                        }
                        Err(e) => return Err(e),
                    }
                }
                Some(res) = &mut output_future => {
                    match res {
                        Ok(Ok(())) => {
                            output_future = None.into();
                        }
                        Ok(Err(e)) => return Err(e),
                        Err(e) => return Err(BollardError::from(io::Error::from(e))),
                    }
                }
                Some(res) = &mut input_future => {
                    match res {
                        Ok(Ok(())) => {
                            input_future = None.into();
                        }
                        Ok(Err(e)) => return Err(e),
                        Err(e) => return Err(BollardError::from(io::Error::from(e))),
                    }
                }
                Some(res) = &mut stop_future => {
                    if output_future.is_terminated() {
                        self.output_handle = None;
                    }
                    if input_future.is_terminated() {
                        self.input_handle = None;
                    }

                    if res.unwrap_or(true) {
                        return self.kill().await;
                    } else {
                        return self.stop().await;
                    }
                }
                else => {
                    let image = PulledImage { image: self.image, docker: self.docker };

                    return Ok(StoppedContainer { id: self.id, image });
                }
            }
        }
    }
}

pub struct StoppedContainer<'a> {
    id: Cow<'a, str>,
    pub image: PulledImage<'a>,
}

impl<'a> StoppedContainer<'a> {
    /// Remove the container from the docker daemon
    pub async fn remove(self) -> Result<PulledImage<'a>, BollardError> {
        self.image.docker.remove_container(&self.id, None).await?;
        Ok(self.image)
    }
}

#[pin_project(project = FlexCompressorProj)]
enum FlexCompressor<W> {
    None(#[pin] W),
    #[cfg(feature = "zstd")]
    Zstd(#[pin] async_compression::tokio::write::ZstdEncoder<W>),
}

impl<W: AsyncWrite + Unpin> FlexCompressor<W> {
    pub async fn flush(&mut self) -> Result<(), io::Error> {
        match self {
            FlexCompressor::None(w) => w.flush().await,
            #[cfg(feature = "zstd")]
            FlexCompressor::Zstd(w) => w.flush().await,
        }
    }
}

impl<W: AsyncWrite> AsyncWrite for FlexCompressor<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            FlexCompressorProj::None(w) => w.poll_write(cx, buf),
            #[cfg(feature = "zstd")]
            FlexCompressorProj::Zstd(w) => w.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            FlexCompressorProj::None(w) => w.poll_flush(cx),
            #[cfg(feature = "zstd")]
            FlexCompressorProj::Zstd(w) => w.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            FlexCompressorProj::None(w) => w.poll_shutdown(cx),
            #[cfg(feature = "zstd")]
            FlexCompressorProj::Zstd(w) => w.poll_shutdown(cx),
        }
    }
}

#[pin_project::pin_project(project = FlexDecompressorProj)]
enum FlexDecompressor<R> {
    None(#[pin] R),
    #[cfg(feature = "zstd")]
    Zstd(#[pin] async_compression::tokio::bufread::ZstdDecoder<R>),
}

impl<R: AsyncBufRead> AsyncRead for FlexDecompressor<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.project() {
            FlexDecompressorProj::None(r) => r.poll_read(cx, buf),
            #[cfg(feature = "zstd")]
            FlexDecompressorProj::Zstd(r) => r.poll_read(cx, buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Write as _},
    };

    use once_cell::sync::Lazy;
    use tempfile::{tempdir, NamedTempFile};
    use tokio::{
        io::{AsyncBufReadExt, BufReader},
        sync::{oneshot, Barrier},
    };

    use crate::Compression;

    #[cfg(feature = "zstd")]
    const ZSTD_NUM: usize = 1;

    #[cfg(not(feature = "zstd"))]
    const ZSTD_NUM: usize = 0;

    const TEST_NUM: usize = 10 + ZSTD_NUM;

    static BARRIER: Lazy<Barrier> = Lazy::new(|| Barrier::new(TEST_NUM));

    macro_rules! wait_before_remove {
        ($image:ident) => {
            BARRIER.wait().await;
            let _ = $image.remove().await;
        };
    }

    #[tokio::test]
    async fn test_pull_image() {
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image("alpine:latest", true).await.unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_create_and_stop_container() {
        const IMAGE: &str = "redis:alpine";
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image(IMAGE, false).await.unwrap();
        let container = image
            .create_container()
            .enable_stream()
            .run()
            .await
            .unwrap();
        let stopped = container.stop().await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_create_and_stop_container_with_io() {
        const IMAGE: &str = "alpine:latest";
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image(IMAGE, false).await.unwrap();
        let (stdin, stdin_out) = tokio::io::duplex(4096);
        let (stdout, stdout_out) = tokio::io::duplex(4096);
        let container = image
            .create_container()
            .enable_stream()
            .cmd(vec!["echo", "Hello"])
            .stdin(stdin)
            .stdout(stdout)
            .run()
            .await
            .unwrap();
        let stopped = container.stop().await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);
        drop(stdin_out);
        drop(stdout_out);
    }

    #[tokio::test]
    async fn test_create_and_run_to_completion() {
        const IMAGE: &str = "alpine:latest";
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image(IMAGE, false).await.unwrap();
        let container = image
            .create_container()
            .enable_stream()
            .cmd(vec!["sh", "-c", "echo hello"])
            .run()
            .await
            .unwrap();
        let stopped = container.run_to_completion(None).await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_container_io() {
        const IMAGE: &str = "alpine:latest";
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image(IMAGE, false).await.unwrap();

        let (rx, stdout) = tokio::io::duplex(4096);
        let container = image
            .create_container()
            .enable_stream()
            .env("TEST", "hello")
            .cmd(vec!["sh", "-c", "echo $TEST"])
            .stdout(stdout)
            .enable_stream()
            .run()
            .await
            .unwrap();
        let mut rx = BufReader::new(rx).lines();
        let line = rx.next_line().await.unwrap().unwrap();
        assert_eq!(line, "hello");
        let stopped = container.stop().await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_kill_container() {
        const IMAGE: &str = "alpine:latest";
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image(IMAGE, false).await.unwrap();
        let container = image
            .create_container()
            .enable_stream()
            .cmd(vec!["sh", "-c", "sleep 1000"])
            .run()
            .await
            .unwrap();
        let stopped = container.kill().await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_cancel_run_to_completion() {
        const IMAGE: &str = "alpine:latest";
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image(IMAGE, false).await.unwrap();

        let (rx, stdout) = tokio::io::duplex(4096);
        let container = image
            .create_container()
            .enable_stream()
            .cmd(vec!["sh", "-c", "while true; do echo hello; sleep 1; done"])
            .stdout(stdout)
            .run()
            .await
            .unwrap()
            .into_owned();

        let (stop_sender, stop_receiver) = oneshot::channel();

        let handle = tokio::spawn(container.run_to_completion(Some(stop_receiver)));

        let mut rx = BufReader::new(rx).lines();
        for _ in 0..5 {
            let line = rx.next_line().await.unwrap().unwrap();
            assert_eq!(line, "hello");
        }

        stop_sender.send(false).unwrap();

        let stopped = handle.await.unwrap().unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_export_import() {
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image("alpine:latest", false).await.unwrap();
        let archive = image.export(Compression::None).await.unwrap();
        manager
            .import_image(archive, Compression::None)
            .await
            .unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    #[cfg(feature = "zstd")]
    async fn test_export_import_zstd() {
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image("alpine:latest", false).await.unwrap();
        let archive = image.export(Compression::Zstd).await.unwrap();
        manager
            .import_image(archive, Compression::Zstd)
            .await
            .unwrap();
        wait_before_remove!(image);
    }

    #[tokio::test]
    async fn test_volume_read() {
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image("alpine:latest", false).await.unwrap();
        let (rx, stdout) = tokio::io::duplex(4096);

        let mut tempfile = NamedTempFile::new().unwrap();
        let path = tempfile.path().display().to_string();
        writeln!(tempfile, "Hello").unwrap();
        tempfile.flush().unwrap();

        let container = image
            .create_container()
            .enable_stream()
            .cmd(vec!["cat", "/tmp/hello"])
            .stdout(stdout)
            .add_volume(path.clone(), "/tmp/hello")
            .run()
            .await
            .unwrap();

        let mut rx = BufReader::new(rx).lines();
        let line = rx.next_line().await.unwrap().unwrap();

        let stopped = container.stop().await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);

        assert_eq!(line, "Hello");
        drop(tempfile);
    }

    #[tokio::test]
    async fn test_volume_write() {
        let manager = super::ContainerManager::new().unwrap();
        let image = manager.pull_image("alpine:latest", false).await.unwrap();

        let tempfile = tempdir().unwrap();
        let path = tempfile.path().display().to_string();

        let container = image
            .create_container()
            .enable_stream()
            .cmd(vec!["sh", "-c", "echo 'Hello' > /tmp/hello/file"])
            .add_volume(path.clone(), "/tmp/hello")
            .run()
            .await
            .unwrap();

        let stopped = container.run_to_completion(None).await.unwrap();
        let image = stopped.remove().await.unwrap();
        wait_before_remove!(image);

        let mut line = String::new();
        let mut file = File::open(path.clone() + "/file").unwrap();
        file.read_to_string(&mut line).unwrap();
        drop(file);

        let _ = std::fs::remove_file(path);

        assert_eq!(line, "Hello\n");
        drop(tempfile);
    }
}
