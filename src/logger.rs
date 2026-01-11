//! Structured logger wrapper with context propagation.

use crate::context::Context;
use crate::redaction::redact_field;
use crate::schema::{Event, Level, StandardFields};
use tracing::{event, Level as TracingLevel};
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

/// Error type for logger initialization.
#[derive(Debug, thiserror::Error)]
pub enum LoggerError {
    #[error("Failed to initialize logger: {0}")]
    Initialization(String),
}

/// Initialize the global logger with structured JSON output.
///
/// # Arguments
///
/// * `service_name` - Name of the service (used in all log events)
/// * `default_level` - Default log level (e.g., "info", "debug")
///
/// # Example
///
/// ```no_run
/// use shared_logging::init_logger;
///
/// init_logger("my-service", "info").unwrap();
/// ```
pub fn init_logger(service_name: &str, default_level: &str) -> Result<(), LoggerError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    let fmt_layer = fmt::layer()
        .json()
        .with_timer(ChronoUtc::rfc_3339())
        .with_target(false)
        .with_current_span(false)
        .with_span_list(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stdout);

    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| LoggerError::Initialization(e.to_string()))?;

    // Set service name in a global context
    std::env::set_var("LOG_SERVICE_NAME", service_name);

    Ok(())
}

/// Structured logger with context propagation.
pub struct Logger {
    service_name: String,
    module: Option<String>,
    context: Context,
}

impl Logger {
    /// Create a new logger instance.
    ///
    /// # Arguments
    ///
    /// * `module` - Module/component name (optional)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared_logging::Logger;
    ///
    /// let logger = Logger::new("auth");
    /// logger.info("User authenticated");
    /// ```
    pub fn new(module: impl Into<Option<String>>) -> Self {
        let service_name = std::env::var("LOG_SERVICE_NAME")
            .unwrap_or_else(|_| "unknown-service".to_string());

        Self {
            service_name,
            module: module.into(),
            context: Context::new(),
        }
    }

    /// Create a logger with context.
    pub fn with_context(module: impl Into<Option<String>>, context: Context) -> Self {
        let service_name = std::env::var("LOG_SERVICE_NAME")
            .unwrap_or_else(|_| "unknown-service".to_string());

        Self {
            service_name,
            module: module.into(),
            context,
        }
    }

    /// Set context for this logger instance.
    pub fn set_context(&mut self, context: Context) {
        self.context = context;
    }

    /// Merge context into this logger instance.
    pub fn merge_context(&mut self, context: Context) {
        self.context = self.context.clone().merge(context);
    }

    /// Log a trace-level message.
    pub fn trace(&self, message: impl Into<String>) {
        self.log(Level::Trace, message, None::<fn(&mut EventBuilder)>);
    }

    /// Log a debug-level message.
    pub fn debug(&self, message: impl Into<String>) {
        self.log(Level::Debug, message, None::<fn(&mut EventBuilder)>);
    }

    /// Log an info-level message.
    pub fn info(&self, message: impl Into<String>) {
        self.log(Level::Info, message, None::<fn(&mut EventBuilder)>);
    }

    /// Log a warn-level message.
    pub fn warn(&self, message: impl Into<String>) {
        self.log(Level::Warn, message, None::<fn(&mut EventBuilder)>);
    }

    /// Log an error-level message.
    pub fn error(&self, message: impl Into<String>) {
        self.log(Level::Error, message, None::<fn(&mut EventBuilder)>);
    }

    /// Log a trace-level message with fields.
    pub fn trace_with<F>(&self, message: impl Into<String>, f: F)
    where
        F: FnOnce(&mut EventBuilder),
    {
        self.log(Level::Trace, message, Some(f));
    }

    /// Log a debug-level message with fields.
    pub fn debug_with<F>(&self, message: impl Into<String>, f: F)
    where
        F: FnOnce(&mut EventBuilder),
    {
        self.log(Level::Debug, message, Some(f));
    }

    /// Log an info-level message with fields.
    pub fn info_with<F>(&self, message: impl Into<String>, f: F)
    where
        F: FnOnce(&mut EventBuilder),
    {
        self.log(Level::Info, message, Some(f));
    }

    /// Log a warn-level message with fields.
    pub fn warn_with<F>(&self, message: impl Into<String>, f: F)
    where
        F: FnOnce(&mut EventBuilder),
    {
        self.log(Level::Warn, message, Some(f));
    }

    /// Log an error-level message with fields.
    pub fn error_with<F>(&self, message: impl Into<String>, f: F)
    where
        F: FnOnce(&mut EventBuilder),
    {
        self.log(Level::Error, message, Some(f));
    }

    /// Log an error.
    pub fn log_error(&self, message: impl Into<String>, error: &dyn std::error::Error) {
        self.error_with(message, |e| {
            e.error(error);
        });
    }

    /// Internal logging method.
    fn log<F>(&self, level: Level, message: impl Into<String>, fields_fn: Option<F>)
    where
        F: FnOnce(&mut EventBuilder),
    {
        let message = message.into();
        let tracing_level = level.to_tracing_level();

        // Build event fields
        let mut builder = EventBuilder::new();
        
        // Add standard fields
        builder.field(StandardFields::SERVICE, &self.service_name);
        if let Some(ref module) = self.module {
            builder.field(StandardFields::MODULE, module);
        }
        builder.field(StandardFields::LEVEL, level.to_string());
        builder.field(StandardFields::MESSAGE, &message);

        // Add context fields
        for (key, value) in self.context.to_fields() {
            builder.field(key, &value);
        }

        // Add custom fields
        if let Some(f) = fields_fn {
            f(&mut builder);
        }

        // Extract fields for logging
        let fields = builder.build();
        let fields_map = fields.as_object().unwrap();

        // Build dynamic event macro call - we'll use a helper function
        self.log_with_fields(tracing_level, &message, fields_map);
    }

    /// Log with fields map.
    fn log_with_fields(
        &self,
        level: TracingLevel,
        message: &str,
        fields: &serde_json::Map<String, serde_json::Value>,
    ) {
        // Serialize all fields as JSON for inclusion in the log event
        // The JSON formatter will parse and include all fields in the structured output
        let fields_json = serde_json::to_string(fields).unwrap_or_else(|_| "{}".to_string());
        
        // Emit the event with fields as JSON
        // The JSON formatter will parse and include all fields in the structured output
        // Note: We include module in the fields JSON instead of as target
        match level {
            TracingLevel::TRACE => {
                event!(
                    TracingLevel::TRACE,
                    message = %message,
                    service = %self.service_name,
                    fields = %fields_json
                );
            }
            TracingLevel::DEBUG => {
                event!(
                    TracingLevel::DEBUG,
                    message = %message,
                    service = %self.service_name,
                    fields = %fields_json
                );
            }
            TracingLevel::INFO => {
                event!(
                    TracingLevel::INFO,
                    message = %message,
                    service = %self.service_name,
                    fields = %fields_json
                );
            }
            TracingLevel::WARN => {
                event!(
                    TracingLevel::WARN,
                    message = %message,
                    service = %self.service_name,
                    fields = %fields_json
                );
            }
            TracingLevel::ERROR => {
                event!(
                    TracingLevel::ERROR,
                    message = %message,
                    service = %self.service_name,
                    fields = %fields_json
                );
            }
        }
    }
}


/// Builder for log event fields.
pub struct EventBuilder {
    fields: serde_json::Map<String, serde_json::Value>,
}

impl EventBuilder {
    /// Create a new event builder.
    pub fn new() -> Self {
        Self {
            fields: serde_json::Map::new(),
        }
    }

    /// Add a field to the event.
    ///
    /// Values are automatically redacted based on field name and content.
    pub fn field(&mut self, name: &str, value: impl ToString) -> &mut Self {
        let value_str = value.to_string();
        let redacted = redact_field(name, &value_str);
        self.fields.insert(name.to_string(), serde_json::Value::String(redacted));
        self
    }

    /// Add a field without redaction (use with caution).
    pub fn field_raw(&mut self, name: &str, value: impl Into<serde_json::Value>) -> &mut Self {
        self.fields.insert(name.to_string(), value.into());
        self
    }

    /// Add an error to the event.
    pub fn error(&mut self, error: &dyn std::error::Error) -> &mut Self {
        let error_obj = Event::format_error(error);
        self.fields.insert(StandardFields::ERROR.to_string(), error_obj);
        self
    }

    /// Build the fields map.
    pub fn build(self) -> serde_json::Value {
        serde_json::Value::Object(self.fields)
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let mut builder = EventBuilder::new();
        builder.field("user_id", "user123");
        builder.field("action", "login");
        
        let fields = builder.build();
        assert!(fields.is_object());
    }
}
