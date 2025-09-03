# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### JavaScript/TypeScript (Guest JS API)
```bash
# Build the JavaScript/TypeScript API
pnpm build

# Install dependencies
pnpm install
```

### Rust (Plugin Core)
```bash
# Build the Rust plugin
cargo build

# Run tests
cargo test

# Check code
cargo check
```

### Example Application
```bash
# Navigate to example app
cd examples/tauri-app

# Run the example Tauri app in development
pnpm tauri dev

# Build the example app
pnpm tauri build
```

## Architecture Overview

This is a Tauri v2 plugin for integrating PostHog analytics. The plugin follows Tauri's standard plugin architecture:

### Core Components

1. **Rust Backend (`src/`)**:
   - `lib.rs`: Main plugin initialization and trait definitions
   - `commands.rs`: Tauri command handlers exposed to frontend
   - `models.rs`: Shared data structures (request/response types)
   - `desktop.rs`: Desktop-specific implementation
   - `mobile.rs`: Mobile-specific implementation (iOS/Android)
   - `error.rs`: Error handling

2. **JavaScript/TypeScript API (`guest-js/`)**:
   - `index.ts`: TypeScript API that wraps the plugin commands for frontend usage
   - Built with Rollup to produce both ESM and CommonJS modules

3. **Plugin Integration Pattern**:
   - Commands are registered in `lib.rs` via `tauri::generate_handler!`
   - Frontend calls go through the guest-js API which invokes Rust commands
   - Uses the `plugin:posthog|` namespace for command invocation

### Key Implementation Details

- The plugin uses conditional compilation for desktop vs mobile platforms
- State management via Tauri's `Manager` trait and app state
- Currently implements a basic `ping` command as a template
- Ready for PostHog SDK integration in the platform-specific modules

### Adding New Features

When implementing PostHog functionality:
1. Add data models in `src/models.rs`
2. Create command handlers in `src/commands.rs`
3. Implement platform-specific logic in `desktop.rs`/`mobile.rs`
4. Expose TypeScript API in `guest-js/index.ts`
5. Register new commands in `lib.rs` handler list