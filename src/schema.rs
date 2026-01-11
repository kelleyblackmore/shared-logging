//! Standard event schema for consistent log formatting.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    /// Convert to tracing level.
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            Level::Trace => tracing::Level::TRACE,
            Level::Debug => tracing::Level::DEBUG,
            Level::Info => tracing::Level::INFO,
            Level::Warn => tracing::Level::WARN,
            Level::Error => tracing::Level::ERROR,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Trace => write!(f, "trace"),
            Level::Debug => write!(f, "debug"),
            Level::Info => write!(f, "info"),
            Level::Warn => write!(f, "warn"),
            Level::Error => write!(f, "error"),
        }
    }
}

/// Standard field names used across all log events.
pub struct StandardFields;

impl StandardFields {
    /// Service name field
    pub const SERVICE: &'static str = "service";
    /// Module/component name field
    pub const MODULE: &'static str = "module";
    /// Log level field
    pub const LEVEL: &'static str = "level";
    /// Message field
    pub const MESSAGE: &'static str = "message";
    /// Timestamp field
    pub const TIMESTAMP: &'static str = "timestamp";
    /// Error field (for error details)
    pub const ERROR: &'static str = "error";
    /// Error type field
    pub const ERROR_TYPE: &'static str = "error_type";
    /// Error message field
    pub const ERROR_MESSAGE: &'static str = "error_message";
    /// Error stack trace field
    pub const ERROR_STACK: &'static str = "error_stack";
    /// Trace ID field
    pub const TRACE_ID: &'static str = "trace_id";
    /// Span ID field
    pub const SPAN_ID: &'static str = "span_id";
    /// Request ID field
    pub const REQUEST_ID: &'static str = "request_id";
    /// User ID field
    pub const USER_ID: &'static str = "user_id";
    /// Tenant ID field
    pub const TENANT_ID: &'static str = "tenant_id";
    /// HTTP method field
    pub const HTTP_METHOD: &'static str = "http.method";
    /// HTTP path field
    pub const HTTP_PATH: &'static str = "http.path";
    /// HTTP status code field
    pub const HTTP_STATUS: &'static str = "http.status";
    /// HTTP duration field
    pub const HTTP_DURATION_MS: &'static str = "http.duration_ms";
    /// Client IP field
    pub const CLIENT_IP: &'static str = "client.ip";
    /// User agent field
    pub const USER_AGENT: &'static str = "user_agent";
}

/// Standard log event structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Service name
    #[serde(rename = "service")]
    pub service: String,
    /// Module/component name
    #[serde(rename = "module")]
    pub module: Option<String>,
    /// Log level
    #[serde(rename = "level")]
    pub level: Level,
    /// Log message
    #[serde(rename = "message")]
    pub message: String,
    /// Timestamp (ISO 8601)
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    /// Additional fields
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

impl Event {
    /// Create a new event.
    pub fn new(
        service: impl Into<String>,
        module: Option<String>,
        level: Level,
        message: impl Into<String>,
    ) -> Self {
        Self {
            service: service.into(),
            module,
            level,
            message: message.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            fields: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Format an error for logging.
    pub fn format_error(error: &dyn std::error::Error) -> serde_json::Value {
        let mut error_obj = serde_json::Map::new();
        error_obj.insert(
            StandardFields::ERROR_TYPE.to_string(),
            serde_json::Value::String(std::any::type_name_of_val(error).to_string()),
        );
        error_obj.insert(
            StandardFields::ERROR_MESSAGE.to_string(),
            serde_json::Value::String(error.to_string()),
        );

        // Try to get source chain
        let mut source_chain = Vec::new();
        let mut current: Option<&dyn std::error::Error> = Some(error);
        while let Some(err) = current {
            source_chain.push(err.to_string());
            current = err.source();
        }

        if source_chain.len() > 1 {
            error_obj.insert(
                StandardFields::ERROR_STACK.to_string(),
                serde_json::Value::Array(
                    source_chain
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }

        serde_json::Value::Object(error_obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_display() {
        assert_eq!(Level::Info.to_string(), "info");
        assert_eq!(Level::Error.to_string(), "error");
    }

    #[test]
    fn test_event_creation() {
        let event = Event::new("my-service", Some("my-module".to_string()), Level::Info, "test");
        assert_eq!(event.service, "my-service");
        assert_eq!(event.module, Some("my-module".to_string()));
        assert_eq!(event.level, Level::Info);
        assert_eq!(event.message, "test");
    }
}

