//! OpenTelemetry log correlation support.

use crate::context::Context;
use opentelemetry::trace::TraceContextExt;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Extract context from OpenTelemetry span.
pub fn extract_context_from_otel() -> Context {
    let span = tracing::Span::current();
    let otel_ctx = span.context();
    let span_ref = otel_ctx.span();
    let span_context = span_ref.span_context();

    let mut ctx = Context::new();

    if span_context.is_valid() {
        ctx = ctx.with_trace_id(span_context.trace_id().to_string());
        ctx = ctx.with_span_id(span_context.span_id().to_string());
    }

    ctx
}

/// Set OpenTelemetry context from a Context.
///
/// Currently a no-op placeholder: the actual implementation depends on your
/// OTel setup and would typically be done when creating a new span.
pub fn set_otel_context(context: &Context) {
    let _ = context;
}

/// Initialize OpenTelemetry tracing.
pub fn init_otel_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use opentelemetry_sdk::Resource;

    let resource = Resource::builder()
        .with_service_name(service_name.to_string())
        .build();

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_context() {
        // Without an active OTel span the context should be empty
        let ctx = super::extract_context_from_otel();
        assert!(ctx.trace_id.is_none());
        assert!(ctx.span_id.is_none());
    }
}
