use crate::error::Result;
use crate::models::*;
use parking_lot::RwLock;
use posthog_rs::{Client as PostHogClient, ClientOptionsBuilder, Event};
use std::sync::Arc;

pub struct PostHogClientWrapper {
    client: PostHogClient,
    distinct_id: Arc<RwLock<Option<String>>>,
    device_id: String,
    config: PostHogConfig,
    auto_identify_enabled: bool,
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

        // Generate device_id using machine_uid like mixpanel-rs
        let device_id = Self::generate_device_id()?;

        // Auto-generate distinct_id similar to mixpanel-rs pattern
        let auto_identify_enabled = config.auto_capture;
        let auto_distinct_id = if auto_identify_enabled {
            Some(format!("$device:{}", device_id))
        } else {
            None
        };

        Ok(Self {
            client,
            distinct_id: Arc::new(RwLock::new(auto_distinct_id)),
            device_id,
            config,
            auto_identify_enabled,
        })
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
            .map_err(|e| crate::error::Error::PostHogClient(e))?;

        // Add custom properties
        if let Some(properties) = request.properties {
            for (key, value) in properties {
                event
                    .insert_prop(key, value)
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
            event
                .set_timestamp(timestamp)
                .map_err(|e| crate::error::Error::PostHogClient(e))?;
        }

        self.client
            .capture(event)
            .await
            .map_err(|e| crate::error::Error::PostHogClient(e))?;
        Ok(())
    }

    pub async fn capture_batch(&self, requests: Vec<CaptureRequest>) -> Result<()> {
        let mut events = Vec::new();

        for request in requests {
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
                .map_err(|e| crate::error::Error::PostHogClient(e))?;

            // Add custom properties
            if let Some(properties) = request.properties {
                for (key, value) in properties {
                    event
                        .insert_prop(key, value)
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
                event
                    .set_timestamp(timestamp)
                    .map_err(|e| crate::error::Error::PostHogClient(e))?;
            }

            events.push(event);
        }

        self.client
            .capture_batch(events)
            .await
            .map_err(|e| crate::error::Error::PostHogClient(e))?;
        Ok(())
    }

    pub fn identify(&self, distinct_id: String) {
        *self.distinct_id.write() = Some(distinct_id);
    }

    pub async fn alias(&self, alias: String) -> Result<()> {
        if let Some(distinct_id) = self.get_distinct_id() {
            let mut event = Event::new("$create_alias", &distinct_id);
            event
                .insert_prop("alias", alias)
                .map_err(|e| crate::error::Error::PostHogClient(e))?;
            self.client
                .capture(event)
                .await
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

    /// Get the effective distinct_id (either user-set or auto-generated)
    pub fn get_effective_distinct_id(&self) -> String {
        self.get_distinct_id()
            .unwrap_or_else(|| self.device_id.clone())
    }

    /// Check if auto-identify is enabled
    pub fn is_auto_identify_enabled(&self) -> bool {
        self.auto_identify_enabled
    }

    /// Force re-generate auto distinct_id (useful for testing or reset scenarios)
    /// Note: This will use the same machine_uid, so it's mainly for consistency
    pub fn regenerate_auto_distinct_id(&self) -> Result<()> {
        if self.auto_identify_enabled {
            let auto_distinct_id = format!("$device:{}", self.device_id);
            *self.distinct_id.write() = Some(auto_distinct_id);
        }
        Ok(())
    }
}

