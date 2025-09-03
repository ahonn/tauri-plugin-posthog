use tauri::{command, State};
use crate::client::PostHogClientWrapper;
use crate::models::*;
use crate::error::Result;

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
pub async fn alias(
    request: AliasRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.identify(request.distinct_id);
    client.alias(request.alias).await
}

#[command]
pub fn reset(
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.reset();
    Ok(())
}

#[command]
pub fn get_distinct_id(
    client: State<'_, PostHogClientWrapper>,
) -> Result<Option<String>> {
    Ok(client.get_distinct_id())
}

#[command]
pub fn get_device_id(
    client: State<'_, PostHogClientWrapper>,
) -> Result<String> {
    Ok(client.get_device_id())
}

#[command]
pub async fn capture_batch(
    request: BatchCaptureRequest,
    client: State<'_, PostHogClientWrapper>,
) -> Result<()> {
    client.capture_batch(request.events).await
}

// Legacy ping command for backward compatibility
#[command]
pub(crate) async fn ping(
    payload: PingRequest,
) -> Result<PingResponse> {
    Ok(PingResponse {
        value: payload.value,
    })
}
