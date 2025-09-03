# Tauri PostHog Plugin Example

This example application demonstrates the usage of the tauri-plugin-posthog plugin with Svelte and Vite.

## Setup

1. **Environment Variables**
   
   Create a `.env` file from the example:
   ```bash
   cp .env.example .env
   ```
   
   Update your `.env` file with your actual PostHog API key:
   ```env
   POSTHOG_API_KEY=phc_your_actual_api_key_here
   ```

2. **Install Dependencies**
   ```bash
   pnpm install
   ```

3. **Run the Development Server**
   ```bash
   pnpm tauri dev
   ```

## Features

This example app demonstrates:
- Event capture with custom properties
- User identification
- Alias creation  
- Anonymous events
- Batch event capture
- ID management (distinct ID and device ID)
- Session reset

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

