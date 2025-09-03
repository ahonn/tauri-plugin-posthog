# PostHog Tauri Plugin Implementation Plan

## Overview

This document outlines the implementation plan for a PostHog analytics plugin for Tauri applications, based on the official PostHog Rust SDK (`posthog-rs`).

## PostHog-rs Supported Features

Based on the official PostHog Rust SDK, we need to support all these interfaces:

### Client Features
- **Client Creation**: Both async and blocking clients with custom options
- **Event Capture**: Single event capture
- **Batch Capture**: Multiple events in one request
- **Global Client**: Singleton client instance with global functions
- **Client Configuration**: Custom API endpoints, timeouts, etc.

### Event Features
- **Identified Events**: `Event::new(event, distinct_id)`
- **Anonymous Events**: `Event::new_anon(event)` with automatic UUID generation
- **Event Properties**: `insert_prop(key, value)` for custom properties
- **Group Analytics**: `add_group(group_type, group_id)` for group events
- **Timestamp Control**: `set_timestamp()` for historical events
- **Automatic Properties**: Library name/version injection

### Global Client Features
- **Global Initialization**: `init_global_client()`
- **Global Capture**: `capture(event)` using global client
- **Disable Global**: `disable()` and `is_disabled()`
- **Global State Management**: Singleton pattern with thread safety

## Architecture Design

### Core Principles
1. **Simplicity First**: Focus on essential PostHog features
2. **Async by Default**: Use async/await for all network operations
3. **Minimal Persistence**: Store only essential data (distinct_id, device_id)
4. **Type Safety**: Strong typing between Rust and TypeScript

### Component Structure

```
src/
├── lib.rs          # Plugin initialization
├── client.rs       # PostHog client wrapper
├── commands.rs     # Tauri command handlers
├── models.rs       # Data structures
└── error.rs        # Error handling

guest-js/
├── index.ts        # TypeScript API
└── types.ts        # TypeScript type definitions
```

## Implementation Details

### 1. Dependencies (`Cargo.toml`)

```toml
[dependencies]
tauri = { version = "2.8.5" }
posthog-rs = { version = "0.3", default-features = false, features = ["async-client"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2"
uuid = { version = "1.13", features = ["v7", "serde"] }

[build-dependencies]
tauri-plugin = { version = "2.4.0", features = ["build"] }
```

### 2. Core Models (`src/models.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub anonymous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyRequest {
    pub distinct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasRequest {
    pub distinct_id: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostHogConfig {
    pub api_key: String,
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: String,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,
    #[serde(default = "default_auto_capture")]
    pub auto_capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCaptureRequest {
    pub events: Vec<CaptureRequest>,
}

fn default_api_endpoint() -> String {
    "https://us.i.posthog.com".to_string()
}

fn default_request_timeout() -> u64 {
    30
}

fn default_auto_capture() -> bool {
    false
}
```

### 3. Client Wrapper (`src/client.rs`)

```rust
use posthog_rs::{Client as PostHogClient, Event, ClientOptionsBuilder};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
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
            let mut event = Event::new("$create_alias", distinct_id);
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
```

### 4. Command Handlers (`src/commands.rs`)

```rust
use tauri::{command, AppHandle, Runtime, State};
use crate::client::PostHogClientWrapper;
use crate::models::*;
use crate::error::Result;
use std::collections::HashMap;
use serde_json::Value;

#[command]
pub async fn capture<R: Runtime>(
    request: CaptureRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.capture(request).await
}

#[command]
pub async fn identify<R: Runtime>(
    request: IdentifyRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.identify(request.distinct_id.clone());
    
    // Send $identify event with properties if provided
    if let Some(properties) = request.properties {
        let capture_request = CaptureRequest {
            event: "$identify".to_string(),
            properties: Some(properties),
            distinct_id: Some(request.distinct_id),
            groups: None,
            timestamp: None,
            anonymous: false,
        };
        client.capture(capture_request).await?;
    }
    
    Ok(())
}

#[command]
pub async fn alias<R: Runtime>(
    request: AliasRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.identify(request.distinct_id);
    client.alias(request.alias).await
}

#[command]
pub fn reset<R: Runtime>(
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.reset();
    Ok(())
}

#[command]
pub fn get_distinct_id<R: Runtime>(
    client: State<'_, PostHogClientWrapper>,
) -> Result<Option<String>> {
    Ok(client.get_distinct_id())
}

#[command]
pub fn get_device_id<R: Runtime>(
    client: State<'_, PostHogClientWrapper>,
) -> Result<String> {
    Ok(client.get_device_id())
}

#[command]
pub async fn capture_batch<R: Runtime>(
    request: BatchCaptureRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.capture_batch(request.events).await
}

// Global client commands (optional, only if plugin initialized with global support)

#[command]
pub fn is_global_disabled<R: Runtime>() -> Result<bool> {
    Ok(posthog_rs::global_is_disabled())
}

#[command]
pub fn disable_global<R: Runtime>() -> Result<()> {
    posthog_rs::disable_global();
    Ok(())
}

#[command]
pub async fn global_capture<R: Runtime>(
    request: CaptureRequest,
) -> Result<()> {
    let mut event = if request.anonymous || request.distinct_id.is_none() {
        posthog_rs::Event::new_anon(&request.event)
    } else {
        posthog_rs::Event::new(&request.event, request.distinct_id.unwrap())
    };

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
        let parsed_time = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map_err(|e| crate::error::Error::TimestampParse(e.to_string()))?;
        event.set_timestamp(parsed_time)
            .map_err(|e| crate::error::Error::PostHogClient(e))?;
    }

    posthog_rs::capture(event).await
        .map_err(|e| crate::error::Error::PostHogClient(e))?;
    Ok(())
}
```

### 5. Plugin Initialization (`src/lib.rs`)

```rust
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod client;
mod commands;
mod error;
mod models;

pub use models::PostHogConfig;
use client::PostHogClientWrapper;

pub trait PostHogExt<R: Runtime> {
    fn posthog(&self) -> &PostHogClientWrapper;
}

impl<R: Runtime, T: Manager<R>> PostHogExt<R> for T {
    fn posthog(&self) -> &PostHogClientWrapper {
        self.state::<PostHogClientWrapper>().inner()
    }
}

/// Initialize PostHog plugin with configuration
pub fn init<R: Runtime>(config: PostHogConfig) -> TauriPlugin<R> {
    Builder::new("posthog")
        .invoke_handler(tauri::generate_handler![
            // Core tracking methods (matching PostHog-rs capabilities)
            commands::capture,
            commands::identify,
            commands::alias,
            commands::reset,
            
            // Utility methods
            commands::get_distinct_id,
            commands::get_device_id,
            commands::capture_batch,
        ])
        .setup(move |app, _api| {
            tauri::async_runtime::block_on(async {
                let client = PostHogClientWrapper::new(config).await?;
                app.manage(client);
                Ok(())
            })
        })
        .build()
}

/// Initialize PostHog plugin with global client support
pub fn init_with_global<R: Runtime>(config: PostHogConfig) -> TauriPlugin<R> {
    Builder::new("posthog")
        .invoke_handler(tauri::generate_handler![
            commands::capture,
            commands::identify,
            commands::alias,
            commands::reset,
            commands::get_distinct_id,
            commands::get_device_id,
            commands::capture_batch,
            commands::global_capture,
            commands::disable_global,
            commands::is_global_disabled,
        ])
        .setup(move |app, _api| {
            tauri::async_runtime::block_on(async {
                // Initialize both instance and global client
                let client = PostHogClientWrapper::new(config.clone()).await?;
                app.manage(client);
                
                // Initialize global PostHog client
                let client_options = posthog_rs::ClientOptionsBuilder::default()
                    .api_key(config.api_key)
                    .api_endpoint(config.api_endpoint)
                    .request_timeout_seconds(config.request_timeout_seconds)
                    .build()
                    .map_err(|e| crate::error::Error::ClientOptions(e.to_string()))?;

                posthog_rs::init_global(client_options).await
                    .map_err(|e| crate::error::Error::PostHogClient(e))?;
                
                Ok(())
            })
        })
        .build()
}
```

### 6. TypeScript API (`guest-js/index.ts`)

```typescript
import { invoke } from '@tauri-apps/api/core'

// Core interfaces matching PostHog JS SDK patterns
export interface Properties {
  [key: string]: any
}

export interface GroupObject {
  [groupType: string]: string | number
}

// Internal request interfaces for Tauri communication
interface CaptureRequest {
  event: string
  properties?: Properties
  distinctId?: string
  groups?: GroupObject
  timestamp?: string
  anonymous?: boolean
}

/**
 * PostHog client for Tauri applications
 * API designed to match PostHog JS SDK patterns
 */
export class PostHog {
  /**
   * Capture an event with optional properties
   * @param event - The event name
   * @param properties - Event properties (optional)
   */
  static async capture(event: string, properties?: Properties): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties
      } as CaptureRequest
    })
  }

  /**
   * Identify a user with a distinct ID and optional properties
   * @param distinctId - The unique identifier for the user
   * @param properties - User properties (optional)
   */
  static async identify(distinctId: string, properties?: Properties): Promise<void> {
    await invoke('plugin:posthog|identify', {
      request: {
        distinctId,
        properties
      }
    })
  }

  /**
   * Create an alias for the current user
   * @param alias - The alias to create
   */
  static async alias(alias: string): Promise<void> {
    const distinctId = await this.getDistinctId()
    if (!distinctId) {
      throw new Error('Cannot create alias without a distinct ID. Call identify() first.')
    }
    
    await invoke('plugin:posthog|alias', {
      request: {
        distinctId,
        alias
      }
    })
  }

  /**
   * Reset the current user (clears distinct ID and other user data)
   */
  static async reset(): Promise<void> {
    await invoke('plugin:posthog|reset')
  }


  /**
   * Get the current distinct ID
   */
  static async getDistinctId(): Promise<string | null> {
    return await invoke('plugin:posthog|get_distinct_id')
  }

  /**
   * Get the device ID
   */
  static async getDeviceId(): Promise<string> {
    return await invoke('plugin:posthog|get_device_id')
  }

  /**
   * Capture multiple events in batch
   * @param events - Array of events to capture
   */
  static async captureBatch(events: Array<{
    event: string
    properties?: Properties
    timestamp?: Date
  }>): Promise<void> {
    const formattedEvents = events.map(event => ({
      event: event.event,
      properties: event.properties,
      timestamp: event.timestamp?.toISOString()
    }))

    await invoke('plugin:posthog|capture_batch', {
      request: { events: formattedEvents }
    })
  }


  // Advanced methods for power users
  
  /**
   * Capture an anonymous event (does not affect user identification)
   * @param event - The event name
   * @param properties - Event properties (optional)
   */
  static async captureAnonymous(event: string, properties?: Properties): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties,
        anonymous: true
      } as CaptureRequest
    })
  }

  /**
   * Capture event with timestamp (for historical events)
   * @param event - The event name
   * @param properties - Event properties (optional)
   * @param timestamp - Event timestamp
   */
  static async captureWithTimestamp(event: string, properties: Properties | undefined, timestamp: Date): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties,
        timestamp: timestamp.toISOString()
      } as CaptureRequest
    })
  }

  /**
   * Capture event with groups
   * @param event - The event name
   * @param properties - Event properties (optional)
   * @param groups - Group associations
   */
  static async captureWithGroups(event: string, properties: Properties | undefined, groups: GroupObject): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties,
        groups
      } as CaptureRequest
    })
  }
}

// Default export (matching PostHog JS SDK pattern)
export default PostHog

// Convenience exports for functional programming style
export const capture = PostHog.capture.bind(PostHog)
export const identify = PostHog.identify.bind(PostHog)
export const alias = PostHog.alias.bind(PostHog)
export const reset = PostHog.reset.bind(PostHog)

// Alias for PostHog class (common pattern)
export { PostHog as posthog }
```

### 7. TypeScript Types (`guest-js/types.ts`)

```typescript
export interface PostHogEvent {
  event: string
  distinctId?: string
  properties?: Record<string, any>
  groups?: Record<string, string>
}

export interface PostHogUser {
  distinctId: string
  properties?: Record<string, any>
}

export interface PostHogGroup {
  type: string
  key: string
}

export interface PostHogConfig {
  apiKey: string
  apiEndpoint?: string
  autoCapture?: boolean
}
```

## Usage Examples

### Basic Usage in Tauri App

```rust
// In src-tauri/src/main.rs
use tauri_plugin_posthog::{init, PostHogConfig};

fn main() {
    tauri::Builder::default()
        .plugin(init(PostHogConfig {
            api_key: "your-api-key".to_string(),
            api_endpoint: "https://app.posthog.com".to_string(),
            request_timeout_seconds: 30,
            auto_capture: false,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### With Global Client Support

```rust
// For apps that want to use PostHog's global client API
use tauri_plugin_posthog::{init_with_global, PostHogConfig};

fn main() {
    tauri::Builder::default()
        .plugin(init_with_global(PostHogConfig {
            api_key: "your-api-key".to_string(),
            api_endpoint: "https://app.posthog.com".to_string(),
            request_timeout_seconds: 30,
            auto_capture: false,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### TypeScript Usage

```typescript
import PostHog, { capture, identify } from 'tauri-plugin-posthog-api'
// or
import { PostHog } from 'tauri-plugin-posthog-api'

// Basic event tracking (matches PostHog JS SDK style)
await PostHog.capture('button_clicked', {
  button_name: 'signup',
  page: 'landing'
})

// Identify a user (matches PostHog JS SDK style)
await PostHog.identify('user_123', {
  email: 'user@example.com',
  name: 'John Doe',
  plan: 'premium'
})

// Anonymous event (uses PostHog-rs Event::new_anon)
await PostHog.captureAnonymous('page_view', {
  path: '/landing',
  referrer: 'google'
})

// Historical event with timestamp (uses PostHog-rs set_timestamp)
await PostHog.captureWithTimestamp('past_event', 
  { action: 'purchase' },
  new Date('2023-01-01T10:00:00Z')
)

// Event with groups (uses PostHog-rs add_group)
await PostHog.captureWithGroups('feature_used', 
  { feature_name: 'export' },
  { company: 'acme_inc', team: 'engineering' }
)

// Batch events (uses PostHog-rs capture_batch)
await PostHog.captureBatch([
  {
    event: 'page_view',
    properties: { page: '/home' }
  },
  {
    event: 'button_click', 
    properties: { button: 'cta' }
  }
])

// Create alias for user
await PostHog.alias('new_user_id')

// Convenience functions (functional style)
await capture('quick_event', { quick: true })
await identify('user_456')

// Reset on logout (clears distinct ID)
await PostHog.reset()

// Get current IDs
const distinctId = await PostHog.getDistinctId()
const deviceId = await PostHog.getDeviceId()
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capture_event() {
        // Test event capture
    }

    #[test]
    fn test_identify() {
        // Test user identification
    }

    #[test]
    fn test_device_id_generation() {
        // Test device ID is UUID v7
    }
}
```

### Integration Tests

Create an example app in `examples/tauri-app` to test:
- Event capture flow
- User identification
- Batch events
- Error handling

## Future Enhancements

### Phase 2: Feature Flags
```typescript
// Future API
const flags = await PostHog.getFeatureFlags()
const isEnabled = await PostHog.isFeatureEnabled('new-feature')
```

### Phase 3: Session Recording
```typescript
// Future API
await PostHog.startSessionRecording()
await PostHog.stopSessionRecording()
```

### Phase 4: A/B Testing
```typescript
// Future API
const variant = await PostHog.getExperimentVariant('experiment-key')
```

## Performance Considerations

1. **Async Operations**: All network calls are async to avoid blocking the UI
2. **Batch API**: Support for sending multiple events in one request
3. **Error Recovery**: Graceful handling of network failures
4. **Minimal Memory**: No event queuing or caching (rely on PostHog SDK)

## Security Considerations

1. **API Key**: Stored securely in Rust backend, never exposed to frontend
2. **HTTPS Only**: All communication over secure channels
3. **No PII Logging**: Ensure no sensitive data in debug logs
4. **User Consent**: Provide mechanisms for opt-out

## Conclusion

This implementation provides a clean, PostHog-focused API for Tauri applications. The design prioritizes:
- Simple, intuitive API matching PostHog's concepts
- Type safety across Rust and TypeScript
- Minimal dependencies and overhead
- Room for future feature expansion

The plugin can be incrementally enhanced with additional PostHog features while maintaining backward compatibility.