//! Shared logging library with structured logging, context propagation, redaction, and HTTP middleware.
//!
//! # Features
//!
//! - Structured logger wrapper with consistent field names
//! - Context propagation: trace_id, span_id, request_id, user_id, tenant_id
//! - Redaction helpers for PII, secrets, and tokens
//! - Standard event schema with consistent field names and error formatting
//! - HTTP middleware for request ID injection and request lifecycle logging
//! - Optional OpenTelemetry log correlation
//!
//! # Example
//!
//! ```no_run
//! use shared_logging::{init_logger, Logger};
//!
//! // Initialize the logger
//! init_logger("my-service", "info").unwrap();
//!
//! // Create a logger instance
//! let logger = Logger::new("my-module");
//!
//! // Log with context
//! logger.info("User logged in", |e| {
//!     e.field("user_id", "user123");
//! });
//! ```

pub mod context;
pub mod logger;
pub mod redaction;
pub mod schema;

#[cfg(feature = "http")]
pub mod middleware;

#[cfg(feature = "otel")]
pub mod otel;

pub use context::{Context, ContextBuilder};
pub use logger::{init_logger, Logger};
pub use redaction::{Redaction, Redactor};
pub use schema::{Event, Level, StandardFields};

