use crate::client::PostHogClientWrapper;
use crate::error::Result;
use crate::models::*;
use tauri::{command, AppHandle, Runtime, State};

#[command]
pub async fn capture(
    request: CaptureRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.capture(request).await
}

#[command]
pub async fn identify(
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
pub async fn alias(request: AliasRequest, client: State<'_, PostHogClientWrapper>) -> Result<()> {
    client.identify(request.distinct_id);
    client.alias(request.alias).await
}

#[command]
pub fn reset(client: State<'_, PostHogClientWrapper>) -> Result<()> {
    client.reset();
    Ok(())
}

#[command]
pub fn get_distinct_id(client: State<'_, PostHogClientWrapper>) -> Result<Option<String>> {
    Ok(client.get_distinct_id())
}

#[command]
pub async fn get_config<R: Runtime>(app: AppHandle<R>) -> Result<PostHogConfig> {
    // Try to get config from environment variables or Tauri config
    let api_key = std::env::var("POSTHOG_API_KEY")
        .or_else(|_| {
            app.config()
                .plugins
                .0
                .get("posthog")
                .and_then(|v| v.get("apiKey"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or(std::env::VarError::NotPresent)
        })
        .map_err(|_| crate::error::Error::MissingApiKey)?;

    let api_host = std::env::var("POSTHOG_API_HOST")
        .or_else(|_| {
            app.config()
                .plugins
                .0
                .get("posthog")
                .and_then(|v| v.get("apiHost"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| default_api_host());

    // Build options from config
    let mut options = PostHogOptions::default();

    if let Some(plugin_config) = app.config().plugins.0.get("posthog") {
        if let Some(disable_cookie) = plugin_config.get("disableCookie").and_then(|v| v.as_bool()) {
            options.disable_cookie = Some(disable_cookie);
        }
        if let Some(disable_session_recording) = plugin_config
            .get("disableSessionRecording")
            .and_then(|v| v.as_bool())
        {
            options.disable_session_recording = Some(disable_session_recording);
        }
        if let Some(capture_pageview) = plugin_config
            .get("capturePageview")
            .and_then(|v| v.as_bool())
        {
            options.capture_pageview = Some(capture_pageview);
        }
        if let Some(capture_pageleave) = plugin_config
            .get("capturePageleave")
            .and_then(|v| v.as_bool())
        {
            options.capture_pageleave = Some(capture_pageleave);
        }
        if let Some(debug) = plugin_config.get("debug").and_then(|v| v.as_bool()) {
            options.debug = Some(debug);
        }
        if let Some(persistence) = plugin_config.get("persistence").and_then(|v| v.as_str()) {
            options.persistence = Some(persistence.to_string());
        }
        if let Some(person_profiles) = plugin_config.get("personProfiles").and_then(|v| v.as_str())
        {
            options.person_profiles = Some(person_profiles.to_string());
        }
    }

    Ok(PostHogConfig {
        api_key,
        api_host,
        options: if options == PostHogOptions::default() {
            None
        } else {
            Some(options)
        },
    })
}
