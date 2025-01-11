use std::{collections::HashMap, sync::Arc, time::Duration};

use bon::Builder;
use serde::Serialize;
use tokio::{sync::mpsc, time::interval};

const QUEUE_SIZE: usize = 100;

#[derive(Clone, Serialize, Builder, Debug)]
#[builder(on(Arc<str>, into))]
struct Event {
    #[expect(
        clippy::struct_field_names,
        reason = "We need to keep this name since it's important for the serialization."
    )]
    event: Arc<str>,
    distinct_id: Arc<str>,
    properties: HashMap<Arc<str>, serde_json::Value>,
    #[serde(rename = "$timestamp")]
    timestamp: chrono::NaiveDateTime,
}

impl Event {
    pub fn new(event: impl Into<Arc<str>>, distinct_id: impl Into<Arc<str>>) -> Self {
        Self {
            event: event.into(),
            distinct_id: distinct_id.into(),
            properties: HashMap::default(),
            timestamp: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn insert_property(
        &mut self,
        key: impl Into<Arc<str>>,
        value: impl Serialize,
    ) -> Result<(), serde_json::Error> {
        self.properties
            .insert(key.into(), serde_json::to_value(value)?);
        Ok(())
    }

    pub fn anonymous(mut self) -> Self {
        let _ = self.insert_property("$process_person_profile", false);
        self
    }

    pub fn now(mut self) -> Self {
        self.timestamp = chrono::Utc::now().naive_utc();
        self
    }
}

#[derive(Debug, Serialize)]
struct Batch {
    api_key: Arc<str>,
    historical_migration: bool,
    #[expect(
        clippy::struct_field_names,
        reason = "We need to keep this name since it's important for the serialization."
    )]
    batch: Vec<Event>,
}

#[derive(Builder)]
#[builder(on(Arc<str>, into))]
struct ClientOptions {
    #[builder(default = "https://us.i.posthog.com")]
    api_host: Arc<str>,

    api_key: Arc<str>,

    #[builder(default = Duration::from_secs(5))]
    timeout: Duration,
}

struct Client {
    options: ClientOptions,
    client: reqwest::Client,
}

impl Client {
    fn new(options: ClientOptions) -> Self {
        let client = reqwest::Client::builder()
            .timeout(options.timeout)
            .build()
            .expect("Failed to build reqwest client");
        Self { options, client }
    }

    async fn capture_batch(
        &self,
        events: impl Iterator<Item = Event>,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/batch/", self.options.api_host);
        let batch = Batch {
            api_key: self.options.api_key.clone(),
            historical_migration: false,
            batch: events.collect(),
        };
        let _ = self
            .client
            .post(&url)
            .json(&batch)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Telemetry {
    context: Vec<Arc<str>>,
    image: Option<Arc<str>>,
    service: Option<Arc<str>>,
    sender: mpsc::Sender<Event>,
    opt_out: bool,
}

impl Default for Telemetry {
    fn default() -> Self {
        let (sender, mut receiver) = mpsc::channel(QUEUE_SIZE);
        let mut client = option_env!("POSTHOG_TOKEN").map(|token| {
            let options = ClientOptions::builder().api_key(token).build();
            Client::new(options)
        });
        tokio::spawn(async move {
            async fn drain_queue(queue: &mut Vec<Event>, client: &mut Option<Client>) {
                if queue.is_empty() {
                    return;
                }
                if let Some(client) = client {
                    if let Err(e) = client.capture_batch(queue.drain(..)).await {
                        tracing::trace!(?e, "Failed to send telemetry");
                    }
                } else {
                    tracing::trace!("Telemetry client not initialized");
                }
            }

            let mut inner_queue = Vec::with_capacity(QUEUE_SIZE);
            let mut ticker = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        tracing::trace!("Draining telemetry queue, since it's been 30 seconds");
                        drain_queue(&mut inner_queue, &mut client).await;
                    }
                    event = receiver.recv() => {
                        if let Some(event) = event {
                            inner_queue.push(event);
                        }
                        if inner_queue.len() >= QUEUE_SIZE {
                            tracing::trace!("Draining telemetry queue, since it's full");
                            drain_queue(&mut inner_queue, &mut client).await;
                        }
                    }
                }
            }
        });
        Self {
            context: vec![],
            image: None,
            service: None,
            sender,
            opt_out: false,
        }
    }
}

impl Telemetry {
    #[must_use]
    pub fn context(mut self, context: impl Into<Arc<str>>) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn opt_out(&mut self) {
        self.opt_out = true;
    }

    #[must_use]
    pub fn image(mut self, image: impl Into<Arc<str>>) -> Self {
        self.image = Some(image.into());
        self
    }

    #[must_use]
    pub fn service(mut self, service: impl Into<Arc<str>>) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn track(&self, event: &str) {
        if self.opt_out {
            return;
        }
        let mut event = Event::new(event, "distinct_id").anonymous();
        let _ = event.insert_property("component", "bridge");
        if let Some(image) = self.image.as_ref() {
            let _ = event.insert_property("image", image);
        }
        if let Some(service) = self.service.as_ref() {
            let _ = event.insert_property("service", service);
        }
        if !self.context.is_empty() {
            let _ = event.insert_property("context", self.context.clone());
        }
        let _ = event.insert_property("$process_person_profile", false);
        let _ = self.sender.try_send(event.now());
    }
}
