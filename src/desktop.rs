use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Posthog<R>> {
  Ok(Posthog(app.clone()))
}

/// Access to the posthog APIs.
pub struct Posthog<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Posthog<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
