use crate::error::Result;
use crate::models::*;
use parking_lot::RwLock;
use posthog_rs::{Client as PostHogClient, ClientOptionsBuilder, Event};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct PostHogClientWrapper {
    client: OnceCell<PostHogClient>,
    distinct_id: Arc<RwLock<Option<String>>>,
    device_id: String,
    config: PostHogConfig,
}

impl PostHogClientWrapper {
    pub fn new(config: PostHogConfig) -> Result<Self> {
        let device_id = Self::generate_device_id()?;
        let auto_distinct_id = Some(format!("$device:{}", device_id));

        Ok(Self {
            client: OnceCell::new(),
            distinct_id: Arc::new(RwLock::new(auto_distinct_id)),
            device_id,
            config,
        })
    }

    async fn get_client(&self) -> Result<&PostHogClient> {
        self.client.get_or_try_init(|| async {
            // Convert api_host to api_endpoint for the Rust client
            let api_endpoint = if self.config.api_host.ends_with("/") {
                format!("{}i/v0/e/", self.config.api_host)
            } else {
                format!("{}/i/v0/e/", self.config.api_host)
            };

            let client_options = ClientOptionsBuilder::default()
                .api_key(self.config.api_key.clone())
                .api_endpoint(api_endpoint)
                .request_timeout_seconds(30) // Default timeout
                .build()
                .map_err(|e| crate::error::Error::ClientOptions(e.to_string()))?;

            let client = posthog_rs::client(client_options).await;
            Ok(client)
        }).await
    }

    /// Generate a stable device ID using machine UID
    /// This ensures the same device_id across app restarts
    fn generate_device_id() -> Result<String> {
        machine_uid::get().map_err(|e| {
            crate::error::Error::ClientOptions(format!(
                "Failed to get machine UID for device_id: {}",
                e
            ))
        })
    }

    pub async fn capture(&self, request: CaptureRequest) -> Result<()> {
        let client = self.get_client().await?;

        let mut event = if request.anonymous {
            Event::new_anon(&request.event)
        } else {
            // Use provided distinct_id, or auto-generated one, or device_id as fallback
            let distinct_id = request
                .distinct_id
                .or_else(|| self.get_distinct_id())
                .unwrap_or_else(|| self.device_id.clone());
            Event::new(&request.event, &distinct_id)
        };

        // Always add device_id as a property
        event
            .insert_prop("$device_id", &self.device_id)
            .map_err(crate::error::Error::PostHogClient)?;

        // Add custom properties
        if let Some(properties) = request.properties {
            for (key, value) in properties {
                event
                    .insert_prop(key, value)
                    .map_err(crate::error::Error::PostHogClient)?;
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
            event
                .set_timestamp(timestamp)
                .map_err(crate::error::Error::PostHogClient)?;
        }

        client
            .capture(event)
            .await
            .map_err(crate::error::Error::PostHogClient)?;
        Ok(())
    }

    pub fn identify(&self, distinct_id: String) {
        *self.distinct_id.write() = Some(distinct_id);
    }

    pub async fn alias(&self, alias: String) -> Result<()> {
        if let Some(distinct_id) = self.get_distinct_id() {
            let client = self.get_client().await?;
            let mut event = Event::new("$create_alias", &distinct_id);
            event
                .insert_prop("alias", alias)
                .map_err(crate::error::Error::PostHogClient)?;
            client
                .capture(event)
                .await
                .map_err(crate::error::Error::PostHogClient)?;
        }
        Ok(())
    }

    pub fn reset(&self) {
        *self.distinct_id.write() = None;
    }

    pub fn get_distinct_id(&self) -> Option<String> {
        self.distinct_id.read().clone()
    }

    pub fn get_config(&self) -> &PostHogConfig {
        &self.config
    }
}
