use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod client;
mod commands;
mod error;
mod models;

pub use models::PostHogConfig;
pub use error::{Error, Result};
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
            // Core tracking methods
            commands::capture,
            commands::identify,
            commands::alias,
            commands::reset,
            
            // Utility methods
            commands::get_distinct_id,
            commands::get_device_id,
            commands::get_effective_distinct_id,
            commands::is_auto_identify_enabled,
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
