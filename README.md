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

## Development

### Prerequisites

- Rust 1.77.2+
- Node.js 20+
- pnpm 8+

### Setup

1. Clone the repository
2. Install pre-commit hooks (optional but recommended):

```bash
# Install pre-commit (if not already installed)
pip install pre-commit
# or
brew install pre-commit

# Install the hooks
./scripts/install-hooks.sh
```

3. Build the plugin:

```bash
# Build Rust plugin
cargo build

# Build TypeScript API
cd guest-js
pnpm install
pnpm build
```

### Running the Example

```bash
cd examples/tauri-app
pnpm tauri dev
```

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality:

- **Rust formatting** - `cargo fmt --check`
- **Rust linting** - `cargo clippy --all-targets --all-features`
- **Rust tests** - `cargo test --lib --all-features`
- **TypeScript build** - Ensures guest-js builds successfully

Run manually: `pre-commit run --all-files`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass and pre-commit hooks succeed
5. Submit a pull request

## License

This project is licensed under the MIT License.
