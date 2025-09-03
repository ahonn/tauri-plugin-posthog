use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub anonymous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyRequest {
    pub distinct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasRequest {
    pub distinct_id: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostHogConfig {
    pub api_key: String,
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: String,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,
    /// When enabled, automatically generates a device-based distinct_id for anonymous users
    /// Format: $device:{machine_id}. Similar to Mixpanel's auto-identification.
    #[serde(default = "default_auto_capture")]
    pub auto_capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCaptureRequest {
    pub events: Vec<CaptureRequest>,
}

fn default_api_endpoint() -> String {
    "https://us.i.posthog.com".to_string()
}

fn default_request_timeout() -> u64 {
    30
}

fn default_auto_capture() -> bool {
    true  // Enable by default to automatically identify users like mixpanel-rs
}

