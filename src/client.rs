use posthog_rs::{Client as PostHogClient, Event, ClientOptionsBuilder};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::models::*;
use crate::error::Result;

pub struct PostHogClientWrapper {
    client: PostHogClient,
    distinct_id: Arc<RwLock<Option<String>>>,
    device_id: String,
    config: PostHogConfig,
}

impl PostHogClientWrapper {
    pub async fn new(config: PostHogConfig) -> Result<Self> {
        let client_options = ClientOptionsBuilder::default()
            .api_key(config.api_key.clone())
            .api_endpoint(config.api_endpoint.clone())
            .request_timeout_seconds(config.request_timeout_seconds)
            .build()
            .map_err(|e| crate::error::Error::ClientOptions(e.to_string()))?;

        let client = posthog_rs::client(client_options).await;
        let device_id = Uuid::now_v7().to_string();
        
        Ok(Self {
            client,
            distinct_id: Arc::new(RwLock::new(None)),
            device_id,
            config,
        })
    }

    pub async fn capture(&self, request: CaptureRequest) -> Result<()> {
        let mut event = if request.anonymous || request.distinct_id.is_none() {
            Event::new_anon(&request.event)
        } else {
            Event::new(
                &request.event, 
                request.distinct_id.as_ref().unwrap_or(&self.get_distinct_id().unwrap_or_default())
            )
        };

        // Add device_id
        event.insert_prop("$device_id", &self.device_id)
            .map_err(|e| crate::error::Error::PostHogClient(e))?;

        // Add custom properties
        if let Some(properties) = request.properties {
            for (key, value) in properties {
                event.insert_prop(key, value)
                    .map_err(|e| crate::error::Error::PostHogClient(e))?;
            }
        }

        // Add groups
        if let Some(groups) = request.groups {
            for (group_type, group_id) in groups {
                event.add_group(&group_type, &group_id);
            }
        }

        // Set timestamp if provided
        if let Some(timestamp) = request.timestamp {
            event.set_timestamp(timestamp)
                .map_err(|e| crate::error::Error::PostHogClient(e))?;
        }

        self.client.capture(event).await
            .map_err(|e| crate::error::Error::PostHogClient(e))?;
        Ok(())
    }

    pub async fn capture_batch(&self, requests: Vec<CaptureRequest>) -> Result<()> {
        let mut events = Vec::new();
        
        for request in requests {
            let mut event = if request.anonymous || request.distinct_id.is_none() {
                Event::new_anon(&request.event)
            } else {
                Event::new(
                    &request.event, 
                    request.distinct_id.as_ref().unwrap_or(&self.get_distinct_id().unwrap_or_default())
                )
            };

            // Add device_id
            event.insert_prop("$device_id", &self.device_id)
                .map_err(|e| crate::error::Error::PostHogClient(e))?;

            // Add custom properties
            if let Some(properties) = request.properties {
                for (key, value) in properties {
                    event.insert_prop(key, value)
                        .map_err(|e| crate::error::Error::PostHogClient(e))?;
                }
            }

            // Add groups
            if let Some(groups) = request.groups {
                for (group_type, group_id) in groups {
                    event.add_group(&group_type, &group_id);
                }
            }

            // Set timestamp if provided
            if let Some(timestamp) = request.timestamp {
                event.set_timestamp(timestamp)
                    .map_err(|e| crate::error::Error::PostHogClient(e))?;
            }

            events.push(event);
        }

        self.client.capture_batch(events).await
            .map_err(|e| crate::error::Error::PostHogClient(e))?;
        Ok(())
    }

    pub fn identify(&self, distinct_id: String) {
        *self.distinct_id.write() = Some(distinct_id);
    }

    pub async fn alias(&self, alias: String) -> Result<()> {
        if let Some(distinct_id) = self.get_distinct_id() {
            let mut event = Event::new("$create_alias", &distinct_id);
            event.insert_prop("alias", alias)
                .map_err(|e| crate::error::Error::PostHogClient(e))?;
            self.client.capture(event).await
                .map_err(|e| crate::error::Error::PostHogClient(e))?;
        }
        Ok(())
    }

    pub fn reset(&self) {
        *self.distinct_id.write() = None;
    }

    pub fn get_distinct_id(&self) -> Option<String> {
        self.distinct_id.read().clone()
    }

    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    pub fn get_config(&self) -> &PostHogConfig {
        &self.config
    }
}