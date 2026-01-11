//! Context propagation for structured logging.
//!
//! Provides context fields that are automatically included in all log events:
//! - trace_id: OpenTelemetry trace ID
//! - span_id: OpenTelemetry span ID
//! - request_id: HTTP request identifier
//! - user_id: Authenticated user identifier
//! - tenant_id: Multi-tenant organization identifier

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Context fields that are propagated through log events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// OpenTelemetry trace ID
    pub trace_id: Option<String>,
    /// OpenTelemetry span ID
    pub span_id: Option<String>,
    /// HTTP request identifier
    pub request_id: Option<String>,
    /// Authenticated user identifier
    pub user_id: Option<String>,
    /// Multi-tenant organization identifier
    pub tenant_id: Option<String>,
}

impl Context {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            trace_id: None,
            span_id: None,
            request_id: None,
            user_id: None,
            tenant_id: None,
        }
    }

    /// Generate a new request ID.
    pub fn generate_request_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Set the trace ID.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set the span ID.
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Set the request ID.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set the user ID.
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the tenant ID.
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Extract context from the current tracing span.
    ///
    /// Note: This is a simplified implementation. For full OpenTelemetry support,
    /// use the `otel` module's `extract_context_from_otel()` function.
    pub fn from_span() -> Self {
        // For now, return empty context
        // Actual OTel trace/span IDs should be extracted using the otel module
        Self::new()
    }

    /// Merge another context into this one, preferring non-None values.
    pub fn merge(mut self, other: Context) -> Self {
        if other.trace_id.is_some() {
            self.trace_id = other.trace_id;
        }
        if other.span_id.is_some() {
            self.span_id = other.span_id;
        }
        if other.request_id.is_some() {
            self.request_id = other.request_id;
        }
        if other.user_id.is_some() {
            self.user_id = other.user_id;
        }
        if other.tenant_id.is_some() {
            self.tenant_id = other.tenant_id;
        }
        self
    }

    /// Convert context to a key-value map for logging.
    pub fn to_fields(&self) -> Vec<(&'static str, String)> {
        let mut fields = Vec::new();
        if let Some(ref trace_id) = self.trace_id {
            fields.push(("trace_id", trace_id.clone()));
        }
        if let Some(ref span_id) = self.span_id {
            fields.push(("span_id", span_id.clone()));
        }
        if let Some(ref request_id) = self.request_id {
            fields.push(("request_id", request_id.clone()));
        }
        if let Some(ref user_id) = self.user_id {
            fields.push(("user_id", user_id.clone()));
        }
        if let Some(ref tenant_id) = self.tenant_id {
            fields.push(("tenant_id", tenant_id.clone()));
        }
        fields
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating context instances.
#[derive(Debug, Default)]
pub struct ContextBuilder {
    context: Context,
}

impl ContextBuilder {
    /// Create a new context builder.
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    /// Set the trace ID.
    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.context.trace_id = Some(trace_id.into());
        self
    }

    /// Set the span ID.
    pub fn span_id(mut self, span_id: impl Into<String>) -> Self {
        self.context.span_id = Some(span_id.into());
        self
    }

    /// Set the request ID.
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.context.request_id = Some(request_id.into());
        self
    }

    /// Generate and set a new request ID.
    pub fn generate_request_id(mut self) -> Self {
        self.context.request_id = Some(Context::generate_request_id());
        self
    }

    /// Set the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.context.user_id = Some(user_id.into());
        self
    }

    /// Set the tenant ID.
    pub fn tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.context.tenant_id = Some(tenant_id.into());
        self
    }

    /// Build the context.
    pub fn build(self) -> Context {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let ctx = ContextBuilder::new()
            .trace_id("trace123")
            .request_id("req456")
            .user_id("user789")
            .build();

        assert_eq!(ctx.trace_id, Some("trace123".to_string()));
        assert_eq!(ctx.request_id, Some("req456".to_string()));
        assert_eq!(ctx.user_id, Some("user789".to_string()));
    }

    #[test]
    fn test_context_merge() {
        let ctx1 = ContextBuilder::new()
            .trace_id("trace1")
            .request_id("req1")
            .build();

        let ctx2 = ContextBuilder::new()
            .trace_id("trace2")
            .user_id("user1")
            .build();

        let merged = ctx1.merge(ctx2);
        assert_eq!(merged.trace_id, Some("trace2".to_string()));
        assert_eq!(merged.request_id, Some("req1".to_string()));
        assert_eq!(merged.user_id, Some("user1".to_string()));
    }
}

