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

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

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
            
            // Legacy ping command
            commands::ping,
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

// Legacy support for older plugin architecture
#[cfg(desktop)]
use desktop::Posthog;
#[cfg(mobile)]
use mobile::Posthog;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the posthog APIs.
pub trait PosthogExt<R: Runtime> {
    fn posthog(&self) -> &Posthog<R>;
}

impl<R: Runtime, T: Manager<R>> crate::PosthogExt<R> for T {
    fn posthog(&self) -> &Posthog<R> {
        self.state::<Posthog<R>>().inner()
    }
}

/// Legacy initializer for backward compatibility
pub fn init_legacy<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("posthog")
        .invoke_handler(tauri::generate_handler![commands::ping])
        .setup(|app, api| {
            #[cfg(mobile)]
            let posthog = mobile::init(app, api)?;
            #[cfg(desktop)]
            let posthog = desktop::init(app, api)?;
            app.manage(posthog);
            Ok(())
        })
        .build()
}
