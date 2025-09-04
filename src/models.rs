use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(default = "default_api_host")]
    pub api_host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PostHogOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PostHogOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_cookie: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_session_recording: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_pageview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_pageleave: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_profiles: Option<String>,
}

impl Default for PostHogConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_host: default_api_host(),
            options: None,
        }
    }
}


pub fn default_api_host() -> String {
    "https://us.i.posthog.com".to_string()
}
