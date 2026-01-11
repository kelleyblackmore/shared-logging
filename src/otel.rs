//! OpenTelemetry log correlation support.

#[cfg(feature = "otel")]
use crate::context::Context;
#[cfg(feature = "otel")]
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
#[cfg(feature = "otel")]
use opentelemetry::Context as OtelContext;
#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Extract context from OpenTelemetry span.
#[cfg(feature = "otel")]
pub fn extract_context_from_otel() -> Context {
    let span = tracing::Span::current();
    let otel_ctx = span.context();
    let span_ref = otel_ctx.span();
    let span_context = span_ref.span_context();
    
    let mut ctx = Context::new();
    
    if span_context.is_valid() {
        ctx = ctx.with_trace_id(format!("{:032x}", span_context.trace_id()));
        ctx = ctx.with_span_id(format!("{:016x}", span_context.span_id()));
    }
    
    ctx
}

/// Set OpenTelemetry context from a Context.
#[cfg(feature = "otel")]
pub fn set_otel_context(context: &Context) {
    // This would typically be done when creating a new span
    // The actual implementation depends on your OTel setup
    if let Some(ref trace_id) = context.trace_id {
        // Parse trace ID and set it
        // This is a simplified version - actual implementation would use OTel SDK
    }
    if let Some(ref span_id) = context.span_id {
        // Parse span ID and set it
        // This is a simplified version - actual implementation would use OTel SDK
    }
}

/// Initialize OpenTelemetry tracing.
#[cfg(feature = "otel")]
pub fn init_otel_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_sdk::trace::TracerProvider;
    use opentelemetry_sdk::Resource;
    use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
    
    let resource = Resource::new(vec![SERVICE_NAME.string(service_name.to_string())]);
    
    let provider = TracerProvider::builder()
        .with_resource(resource)
        .build();
    
    global::set_tracer_provider(provider);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "otel")]
    fn test_extract_context() {
        // This would require an active OTel span
        // For now, just verify the function exists
        let _ctx = super::extract_context_from_otel();
    }
}

