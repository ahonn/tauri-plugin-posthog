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
pub use models::{
    default_api_host, AliasRequest, CaptureRequest, IdentifyRequest, PostHogConfig, PostHogOptions,
};

pub trait PostHogExt<R: Runtime> {
    fn posthog(&self) -> &PostHogClientWrapper;
}

impl<R: Runtime, T: Manager<R>> PostHogExt<R> for T {
    fn posthog(&self) -> &PostHogClientWrapper {
        self.state::<PostHogClientWrapper>().inner()
    }
}

/// Initialize PostHog plugin with configuration
pub fn init<R: Runtime>(config: PostHogConfig) -> TauriPlugin<R, ()> {
    Builder::<R>::new("posthog")
        .invoke_handler(tauri::generate_handler![
            commands::capture,
            commands::identify,
            commands::alias,
            commands::reset,
            commands::get_distinct_id,
            commands::get_config,
        ])
        .setup(move |app, _api| {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                match PostHogClientWrapper::new(config).await {
                    Ok(client) => {
                        app_handle.manage(client);
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize PostHog client: {}", e);
                    }
                }
            });
            Ok(())
        })
        .build()
}
