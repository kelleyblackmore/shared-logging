//! HTTP middleware for request ID injection and request lifecycle logging.

#[cfg(feature = "http")]
use crate::context::{Context, ContextBuilder};
#[cfg(feature = "http")]
use http::{HeaderMap, HeaderValue, Request, Response};
#[cfg(feature = "http")]
use std::time::Instant;
#[cfg(feature = "http")]
use tower::Service;
#[cfg(feature = "http")]
use tracing::{info_span, Instrument};

/// Header name for request ID.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Header name for trace ID (OpenTelemetry).
pub const TRACE_ID_HEADER: &str = "x-trace-id";

/// Header name for span ID (OpenTelemetry).
pub const SPAN_ID_HEADER: &str = "x-span-id";

/// HTTP middleware for logging requests and injecting request IDs.
#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
    service_name: String,
}

impl<S> LoggingMiddleware<S> {
    /// Create a new logging middleware.
    pub fn new(inner: S, service_name: impl Into<String>) -> Self {
        Self {
            inner,
            service_name: service_name.into(),
        }
    }
}

#[cfg(feature = "http")]
impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let service_name = self.service_name.clone();
        
        // Extract or generate request ID
        let request_id = extract_or_generate_request_id(req.headers());
        
        // Extract trace and span IDs if present
        let trace_id = req.headers()
            .get(TRACE_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let span_id = req.headers()
            .get(SPAN_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        // Inject request ID into headers
        req.headers_mut().insert(
            REQUEST_ID_HEADER,
            HeaderValue::from_str(&request_id).unwrap(),
        );
        
        // Build context
        let mut ctx_builder = ContextBuilder::new()
            .request_id(&request_id);
        
        if let Some(ref trace_id) = trace_id {
            ctx_builder = ctx_builder.trace_id(trace_id);
        }
        if let Some(ref span_id) = span_id {
            ctx_builder = ctx_builder.span_id(span_id);
        }
        
        let context = ctx_builder.build();
        
        // Extract request details
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let client_ip = extract_client_ip(&req);
        let user_agent = req.headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        // Create span for request
        let span = info_span!(
            "http_request",
            service = %service_name,
            request_id = %request_id,
            method = %method,
            path = %path,
            client_ip = ?client_ip,
            user_agent = ?user_agent,
        );
        
        // Add context fields to span
        for (key, value) in context.to_fields() {
            span.record(key, value.as_str());
        }
        
        let mut inner = self.inner.clone();
        
        Box::pin(async move {
            // Log request start
            tracing::info!(
                request_id = %request_id,
                method = %method,
                path = %path,
                "Request started"
            );
            
            // Process request
            let mut result = inner.call(req).instrument(span.clone()).await;
            
            // Log request completion and inject request ID
            let duration = start.elapsed();
            let duration_ms = duration.as_millis() as u64;
            
            match &mut result {
                Ok(response) => {
                    let status = response.status();
                    
                    // Inject request ID into response headers
                    response.headers_mut().insert(
                        REQUEST_ID_HEADER,
                        HeaderValue::from_str(&request_id).unwrap(),
                    );
                    
                    tracing::info!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        status = %status.as_u16(),
                        duration_ms = duration_ms,
                        "Request completed"
                    );
                }
                Err(_e) => {
                    // Convert error to string for logging
                    let error_msg = format!("{:?}", _e);
                    tracing::error!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        error = %error_msg,
                        duration_ms = duration_ms,
                        "Request failed"
                    );
                }
            }
            
            result
        })
    }
}

/// Extract request ID from headers or generate a new one.
fn extract_or_generate_request_id(headers: &HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(Context::generate_request_id)
}

/// Extract client IP from request headers.
fn extract_client_ip<B>(req: &Request<B>) -> Option<String> {
    // Check common proxy headers
    let headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip", // Cloudflare
    ];
    
    for header_name in &headers {
        if let Some(header) = req.headers().get(*header_name) {
            if let Ok(value) = header.to_str() {
                // Take first IP if comma-separated list
                let ip = value.split(',').next().unwrap_or(value).trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }
    
    // Fallback to remote address if available
    req.extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip().to_string())
}

// Note: For framework-specific middleware integration:
// - Axum: Use `axum::middleware::from_fn` with a closure that calls LoggingMiddleware
// - Actix-web: Implement as Actix middleware
// - Tower: Use LoggingMiddleware directly as a Service

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(
            REQUEST_ID_HEADER,
            HeaderValue::from_str("test-request-id").unwrap(),
        );
        
        let request_id = extract_or_generate_request_id(&headers);
        assert_eq!(request_id, "test-request-id");
    }

    #[test]
    fn test_generate_request_id() {
        let headers = HeaderMap::new();
        let request_id = extract_or_generate_request_id(&headers);
        assert!(!request_id.is_empty());
    }
}

