use std::{sync::Arc, time::Duration};

use posthog_rs::Event;
use tokio::{sync::mpsc, time::interval};

const QUEUE_SIZE: usize = 100;

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
        let mut client = option_env!("POSTHOG_TOKEN").map(posthog_rs::client);
        tokio::spawn(async move {
            let mut inner_queue = Vec::with_capacity(QUEUE_SIZE);
            let mut ticker = interval(Duration::from_secs(30));

            async fn drain_queue(queue: &mut Vec<Event>, client: &mut Option<posthog_rs::Client>) {
                if queue.is_empty() {
                    return;
                }
                if let Some(client) = client {
                    if let Err(e) = client.async_capture_batch(queue.drain(..)).await {
                        tracing::trace!(?e, "Failed to send telemetry");
                    }
                }
            }

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        drain_queue(&mut inner_queue, &mut client).await;
                    }
                    event = receiver.recv() => {
                        if let Some(event) = event {
                            inner_queue.push(event);
                        }
                        if inner_queue.len() >= QUEUE_SIZE {
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
        let mut event = Event::new(event, "distinct_id");
        let _ = event.insert_prop("component", "bridge");
        if let Some(image) = self.image.as_ref() {
            let _ = event.insert_prop("image", image);
        }
        if let Some(service) = self.service.as_ref() {
            let _ = event.insert_prop("service", service);
        }
        if !self.context.is_empty() {
            let _ = event.insert_prop("context", self.context.clone());
        }
        let _ = event.insert_prop("$process_person_profile", false);
        if !self.opt_out {
            let _ = self.sender.try_send(event);
        }
    }
}
