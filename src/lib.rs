use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod client;
mod commands;
mod error;
mod models;

use client::PostHogClientWrapper;
pub use error::{Error, Result};
pub use models::{PostHogConfig, default_api_endpoint};

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
            commands::capture,
            commands::identify,
            commands::alias,
            commands::reset,
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
