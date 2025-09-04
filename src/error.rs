use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("PostHog client error: {0}")]
    PostHogClient(posthog_rs::Error),
    #[error("Client options error: {0}")]
    ClientOptions(String),
    #[error("Timestamp parse error: {0}")]
    TimestampParse(String),
    #[error("Missing API key: Please set POSTHOG_API_KEY environment variable or configure apiKey in tauri.conf.json")]
    MissingApiKey,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
