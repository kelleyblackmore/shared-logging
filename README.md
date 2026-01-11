# shared-logging

A Rust library for structured logging with context propagation, redaction, and HTTP middleware support.

## Features

- **Structured Logger**: JSON-formatted logs with consistent field names
- **Context Propagation**: Automatic propagation of trace_id, span_id, request_id, user_id, and tenant_id
- **Redaction Helpers**: Automatic redaction of PII, secrets, and tokens
- **Standard Event Schema**: Consistent field names, levels, and error formatting
- **HTTP Middleware**: Request ID injection and request lifecycle logging
- **OpenTelemetry Support**: Optional log correlation with OpenTelemetry traces

## Installation

### From crates.io (Recommended)

```toml
[dependencies]
shared-logging = "0.1.0"

# Optional: Enable HTTP middleware support
# shared-logging = { version = "0.1.0", features = ["http"] }

# Optional: Enable OpenTelemetry support
# shared-logging = { version = "0.1.0", features = ["otel"] }

# Both optional features
# shared-logging = { version = "0.1.0", features = ["http", "otel"] }
```

### Version Constraints

```toml
# Exact version
shared-logging = "0.1.0"

# Compatible versions (semver)
shared-logging = "~0.1.0"  # >=0.1.0, <0.2.0
shared-logging = "^0.1.0"  # >=0.1.0, <0.2.0 (default)
shared-logging = ">=0.1.0, <0.2.0"
```

### Local Path Dependency (Development)

**Best for:** Local development, testing changes, developing alongside your app

If your project structure looks like this:
```
projects/
├── shared-logging/     # The logging library
└── my-service/         # Your application
```

Add to `my-service/Cargo.toml`:
```toml
[dependencies]
shared-logging = { path = "../shared-logging" }

# With optional features
shared-logging = { 
    path = "../shared-logging",
    features = ["http", "otel"]
}
```

**Advantages:**
- ✅ Instant feedback when making changes
- ✅ No need to publish or push changes
- ✅ Easy to debug and develop

### Git Dependency

**Best for:** Using a specific version from a Git repository

```toml
[dependencies]
shared-logging = { git = "https://github.com/kelleyblackmore/shared-logging" }

# Use a specific branch
# shared-logging = { git = "https://github.com/kelleyblackmore/shared-logging", branch = "main" }

# Use a specific tag/version
# shared-logging = { git = "https://github.com/kelleyblackmore/shared-logging", tag = "v0.1.0" }

# Use a specific commit
# shared-logging = { git = "https://github.com/kelleyblackmore/shared-logging", rev = "abc123def456" }
```

### Cargo Workspace (Multiple Projects)

**Best for:** Managing multiple projects that share the logging library

Create a workspace structure:
```
my-workspace/
├── Cargo.toml          # Workspace root
├── shared-logging/      # The logging library
│   └── Cargo.toml
├── service-a/           # Your first service
│   └── Cargo.toml
└── service-b/           # Your second service
    └── Cargo.toml
```

**Workspace root `Cargo.toml`:**
```toml
[workspace]
members = [
    "shared-logging",
    "service-a",
    "service-b",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
```

**Service `Cargo.toml` (e.g., `service-a/Cargo.toml`):**
```toml
[package]
name = "service-a"
version.workspace = true
edition.workspace = true

[dependencies]
shared-logging = { path = "../shared-logging" }
```

### Feature Flags

The crate supports optional features:

- **`http`**: HTTP middleware support (tower-http)
- **`otel`**: OpenTelemetry integration

**Enabling Features:**
```toml
# Single feature
shared-logging = { version = "0.1.0", features = ["http"] }

# Multiple features
shared-logging = { version = "0.1.0", features = ["http", "otel"] }

# Disable default features (if any)
shared-logging = { version = "0.1.0", default-features = false }
```

## Quick Start

### Basic Usage

```rust
use shared_logging::{init_logger, Logger};

fn main() {
    // Initialize the logger (call once at startup)
    init_logger("my-service", "info").unwrap();
    
    // Create a logger instance
    let logger = Logger::new(Some("auth".to_string()));
    
    // Simple logging
    logger.info("User authenticated");
    logger.warn("Rate limit approaching");
    logger.error("Authentication failed");
    
    // Logging with fields
    logger.info_with("User logged in", |e| {
        e.field("user_id", "user123");
        e.field("ip_address", "192.168.1.1");
    });
    
    // Logging errors
    let result: Result<(), &str> = Err("Database connection failed");
    if let Err(e) = result {
        logger.log_error("Failed to connect to database", &e);
    }
}
```

### Context Propagation

```rust
use shared_logging::{Logger, ContextBuilder};

// Create context with correlation IDs
let context = ContextBuilder::new()
    .trace_id("abc123")
    .span_id("def456")
    .generate_request_id()
    .user_id("user789")
    .tenant_id("tenant001")
    .build();

// Create logger with context
let logger = Logger::with_context(Some("api".to_string()), context);

// All logs will include the context fields
logger.info("Processing request");
```

### Redaction

The library automatically redacts sensitive data:

```rust
use shared_logging::Logger;

let logger = Logger::new(Some("api".to_string()));

// These will be automatically redacted:
logger.info_with("User data", |e| {
    e.field("email", "user@example.com");        // Redacted as partial
    e.field("password", "secret123");            // Fully redacted
    e.field("api_key", "sk_live_abc123xyz");     // Fully redacted
    e.field("token", "Bearer eyJhbGc...");      // Fully redacted
});
```

## Step-by-Step Guide

### Step 1: Initialize the Logger

The logger must be initialized once at the start of your application. This sets up:
- JSON formatter
- Log level filtering
- Output destination (stdout by default)

```rust
use shared_logging::init_logger;

fn main() {
    // Initialize with service name and default log level
    init_logger("my-service", "info")
        .expect("Failed to initialize logger");
    
    // Your application code here
}
```

**Parameters:**
- `service_name`: Name of your service (appears in all log events)
- `default_level`: Default log level (`"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`)

**Note:** The log level can be overridden with the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run
RUST_LOG=my_service=debug,other_crate=info cargo run
```

### Step 2: Create Logger Instances

Create logger instances for different modules/components:

```rust
use shared_logging::Logger;

// Create logger for a specific module
let logger = Logger::new(Some("auth".to_string()));
let api_logger = Logger::new(Some("api".to_string()));
let db_logger = Logger::new(Some("database".to_string()));
```

### Step 3: Basic Logging

```rust
let logger = Logger::new(Some("my-module".to_string()));

logger.trace("Very detailed information");
logger.debug("Debug information");
logger.info("Informational message");
logger.warn("Warning message");
logger.error("Error message");
```

**Output:**
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "info",
  "message": "Informational message",
  "service": "my-service",
  "module": "my-module"
}
```

### Step 4: Structured Logging with Fields

Add structured fields to your logs:

```rust
logger.info_with("User logged in", |e| {
    e.field("user_id", "user123");
    e.field("ip_address", "192.168.1.100");
    e.field("login_method", "oauth");
    e.field("success", true);
});
```

**Output:**
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "info",
  "message": "User logged in",
  "service": "my-service",
  "module": "auth",
  "fields": "{\"user_id\":\"user123\",\"ip_address\":\"192.168.1.100\",\"login_method\":\"oauth\",\"success\":true}"
}
```

### Step 5: Context Propagation

Context fields (`trace_id`, `span_id`, `request_id`, `user_id`, `tenant_id`) are automatically included in all logs.

**Creating Context:**
```rust
use shared_logging::{ContextBuilder, Logger};

// Build context
let context = ContextBuilder::new()
    .trace_id("abc123def456")           // OpenTelemetry trace ID
    .span_id("span789")                 // OpenTelemetry span ID
    .generate_request_id()               // Auto-generate request ID
    .user_id("user123")                  // User identifier
    .tenant_id("tenant001")              // Tenant identifier
    .build();

// Create logger with context
let logger = Logger::with_context(Some("api".to_string()), context);
logger.info("Processing request");  // All logs include context fields
```

**Updating Context:**
```rust
let mut logger = Logger::new(Some("api".to_string()));

// Set context
let context = ContextBuilder::new()
    .generate_request_id()
    .user_id("user123")
    .build();
logger.set_context(context);

// Or merge additional context
let additional_context = ContextBuilder::new()
    .tenant_id("tenant001")
    .build();
logger.merge_context(additional_context);
```

### Step 6: Error Logging

**Simple Error Logging:**
```rust
let result: Result<(), String> = Err("Something went wrong".to_string());

if let Err(e) = result {
    logger.log_error("Operation failed", &e);
}
```

**Error with Context:**
```rust
use std::error::Error;

fn process_user(user_id: &str) -> Result<(), Box<dyn Error>> {
    // ... your code ...
    Err("Database error".into())
}

// Log error with context
match process_user("user123") {
    Ok(_) => logger.info("User processed successfully"),
    Err(e) => {
        logger.error_with("Failed to process user", |e| {
            e.field("user_id", "user123");
            e.field("operation", "user_processing");
            e.error(e.as_ref());
        });
    }
}
```

**Output:**
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "error",
  "message": "Failed to process user",
  "service": "my-service",
  "module": "api",
  "fields": "{\"user_id\":\"user123\",\"operation\":\"user_processing\",\"error\":{\"error_type\":\"...\",\"error_message\":\"Database error\",\"error_stack\":[...]}}"
}
```

## HTTP Middleware

### Using with Axum

```rust
use axum::{Router, routing::get};
use shared_logging::init_logger;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    init_logger("http-server", "info").unwrap();
    
    let app = Router::new()
        .route("/", get(handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        );
    
    // ... start server
}
```

### Manual Request ID Handling

```rust
use axum::extract::Request;
use shared_logging::{Logger, ContextBuilder};

async fn handler(request: Request) -> Response {
    // Extract or generate request ID
    let request_id = request.headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            ContextBuilder::new()
                .generate_request_id()
                .build()
                .request_id
                .unwrap()
        });
    
    // Create context with request ID
    let context = ContextBuilder::new()
        .request_id(&request_id)
        .build();
    
    let logger = Logger::with_context(Some("handler".to_string()), context);
    logger.info("Processing request");
    
    // Your handler logic
}
```

The middleware will:
- Generate or extract request IDs from headers
- Inject request IDs into request and response headers
- Log request start and completion with timing
- Extract trace/span IDs from headers (if present)

## OpenTelemetry Integration

### Setup

```rust
use shared_logging::{init_logger, otel};

#[cfg(feature = "otel")]
fn setup_otel() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry
    otel::init_otel_tracing("my-service")?;
    
    // Initialize logger
    init_logger("my-service", "info")?;
    
    Ok(())
}
```

### Extract Context from OTel Span

```rust
use shared_logging::{Logger, otel};

#[cfg(feature = "otel")]
fn log_with_otel_context() {
    // Extract context from current OTel span
    let context = otel::extract_context_from_otel();
    
    let logger = Logger::with_context(Some("api".to_string()), context);
    logger.info("Logging with OTel context");
}
```

## Configuration

### Environment Variables

```bash
# Set log level
export RUST_LOG=info

# Set log level for specific modules
export RUST_LOG=my_service=debug,shared_logging=info

# Set log level for all dependencies
export RUST_LOG=debug
```

### Programmatic Configuration

The logger is configured at initialization:

```rust
// Set default level to "debug"
init_logger("my-service", "debug")?;

// Can be overridden by RUST_LOG environment variable
```

### JSON Output

The logger outputs structured JSON by default. Example output:

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "info",
  "message": "User authenticated",
  "service": "my-service",
  "module": "auth",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "user123",
  "fields": {
    "ip_address": "192.168.1.1"
  }
}
```

## Standard Fields

The library uses consistent field names:

- `service`: Service name
- `module`: Module/component name
- `level`: Log level (trace, debug, info, warn, error)
- `message`: Log message
- `timestamp`: ISO 8601 timestamp
- `trace_id`: OpenTelemetry trace ID
- `span_id`: OpenTelemetry span ID
- `request_id`: HTTP request identifier
- `user_id`: Authenticated user identifier
- `tenant_id`: Multi-tenant organization identifier
- `error`: Error details (when logging errors)
- `http.method`: HTTP method
- `http.path`: HTTP path
- `http.status`: HTTP status code
- `http.duration_ms`: Request duration in milliseconds

## Redaction Patterns

The library automatically detects and redacts:

- **PII**: Email addresses, credit cards, SSNs, phone numbers
- **Secrets**: API keys, bearer tokens, JWTs, AWS keys, private keys

Redaction modes:
- **Full**: `[REDACTED]`
- **Partial**: `****1234` (shows last 4 characters)
- **Hash**: `hash:1234567890` (hashed value)

## Error Formatting

Errors are formatted with:
- Error type (from `std::any::type_name`)
- Error message
- Error chain (if multiple sources)

```rust
logger.log_error("Operation failed", &my_error);
// Output includes:
// {
//   "error": {
//     "error_type": "MyError",
//     "error_message": "Operation failed",
//     "error_stack": ["Error 1", "Error 2"]
//   }
// }
```

## Complete Example

Here's a complete example putting it all together:

```rust
use shared_logging::{init_logger, Logger, ContextBuilder};
use std::error::Error;

fn main() {
    // 1. Initialize logger once at startup
    init_logger("my-service", "info").unwrap();
    
    // 2. Create module loggers
    let logger = Logger::new(Some("main".to_string()));
    logger.info("Application starting");
    
    // 3. Simulate processing a request with context
    let context = ContextBuilder::new()
        .generate_request_id()
        .user_id("user123")
        .build();
    
    let api_logger = Logger::with_context(Some("api".to_string()), context);
    
    // 4. Log request processing
    api_logger.info_with("Processing request", |e| {
        e.field("endpoint", "/api/users");
        e.field("method", "GET");
    });
    
    // 5. Handle errors
    match process_request() {
        Ok(_) => api_logger.info("Request processed successfully"),
        Err(e) => {
            api_logger.error_with("Request failed", |e| {
                e.field("retry_count", 3);
                e.error(&e);
            });
        }
    }
}

fn process_request() -> Result<(), Box<dyn Error>> {
    // Simulate some work
    Ok(())
}
```

## Running Examples

The crate includes runnable examples:

```bash
# Basic usage
cargo run --example basic_usage

# HTTP server example (requires tokio)
cargo run --example http_server

# Error handling example
cargo run --example error_handling
```

## Best Practices

1. **Initialize once**: Call `init_logger()` only once at application startup
2. **Use module names**: Create logger instances with descriptive module names
3. **Propagate context**: Use context for request-scoped logging
4. **Structured fields**: Use `*_with()` methods to add structured fields
5. **Error logging**: Use `log_error()` for proper error formatting
6. **Redaction**: Trust the library to redact sensitive data automatically
7. **Environment control**: Use `RUST_LOG` to control log levels

## Architecture

### What This Library Owns

- Structured logger wrapper
- Context propagation (trace_id, span_id, request_id, user_id, tenant_id)
- Redaction helpers (PII, secrets, tokens)
- Standard event schema (field names, levels, error formatting)
- HTTP middleware for request IDs and lifecycle logging
- Optional OpenTelemetry log correlation

### What This Library Does NOT Own

- Auth token parsing/verification (services handle this)
- Business-specific log fields (services add them via `field()`)

## Troubleshooting

### "Could not find `shared-logging`"

- Check the path is correct relative to your project
- Verify the `shared-logging` directory exists
- Ensure `Cargo.toml` in `shared-logging` has the correct `[package]` name

### "Package `shared-logging` not found in registry"

- If using Git, check the URL is correct
- If using crates.io, ensure the version exists
- Try `cargo update` to refresh the registry

### Feature not available

- Ensure you've enabled the feature in `Cargo.toml`
- Check that the feature exists in the crate's `Cargo.toml`
- Verify you're using a version that supports the feature

### Logs not appearing

- Check `RUST_LOG` environment variable
- Ensure `init_logger()` was called
- Verify log level is appropriate

### Context not propagating

- Ensure context is set on the logger instance
- Check that you're using the same logger instance within a request

### Redaction not working

- Verify field names match redaction patterns (password, token, etc.)
- Check that values match PII/secret patterns

## License

MIT
