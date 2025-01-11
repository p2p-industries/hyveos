use std::sync::Arc;

use posthog_rs::Event;

#[derive(Clone)]
pub struct Telemetry {
    posthog: Option<posthog_rs::Client>,
    context: Vec<Arc<str>>,
    image: Option<Arc<str>>,
    service: Option<Arc<str>>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            posthog: option_env!("POSTHOG_TOKEN").map(posthog_rs::client),
            context: vec![],
            image: None,
            service: None,
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
        tracing::trace!("Opted out of telemetry");
        self.posthog = None;
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
        if let Some(posthog) = self.posthog.clone() {
            tokio::spawn(async move {
                if let Err(e) = posthog.async_capture(event).await {
                    tracing::trace!("Failed to send telemetry event: {e}");
                }
            });
        } else {
            tracing::trace!("Telemetry event: {event:?}");
        }
    }
}
