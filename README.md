# Tauri Plugin PostHog

A Tauri v2 plugin for integrating PostHog analytics into your Tauri applications.

## Features

- Event tracking with custom properties
- User identification and aliasing
- Anonymous event tracking
- Batch event capture
- Device ID management
- TypeScript support

## Installation

Add the plugin to your Tauri project:

```bash
# Add the Rust plugin
cargo add tauri-plugin-posthog

# Add the JavaScript API
pnpm add tauri-plugin-posthog-api
```

## Usage

### Rust Setup

Initialize the plugin in your Tauri app:

```rust
use tauri_plugin_posthog::{PostHogConfig, init};

fn main() {
    tauri::Builder::default()
        .plugin(init(PostHogConfig {
            api_key: "your-posthog-api-key".to_string(),
            ..Default::default()
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Usage

```typescript
import { PostHog } from 'tauri-plugin-posthog-api';

// Capture an event
await PostHog.capture('button_clicked', {
  button: 'signup',
  page: 'landing'
});

// Identify a user
await PostHog.identify('user-123', {
  email: 'user@example.com',
  plan: 'pro'
});

// Capture anonymous events
await PostHog.captureAnonymous('page_view', {
  page: 'pricing'
});
```

## License

This project is licensed under the MIT License.
