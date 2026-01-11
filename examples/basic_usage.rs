//! Basic usage example for shared-logging

use shared_logging::{init_logger, Logger, ContextBuilder};

fn main() {
    // Step 1: Initialize the logger once at application startup
    // This sets up the JSON formatter and log level filtering
    init_logger("my-service", "info").expect("Failed to initialize logger");

    // Step 2: Create a logger instance for your module/component
    // Module name can be Some(String) or None
    let logger = Logger::new(Some("auth".to_string()));

    // Step 3: Basic logging
    println!("\n=== Basic Logging ===");
    logger.info("Application started");
    logger.debug("Debug information");
    logger.warn("This is a warning");
    logger.error("This is an error");

    // Step 4: Logging with custom fields
    println!("\n=== Logging with Fields ===");
    logger.info_with("User logged in", |e| {
        e.field("user_id", "user123");
        e.field("ip_address", "192.168.1.100");
        e.field("login_method", "oauth");
    });

    // Step 5: Logging errors
    println!("\n=== Error Logging ===");
    #[derive(Debug)]
    struct DatabaseError(String);
    impl std::fmt::Display for DatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for DatabaseError {}
    
    let result: Result<(), DatabaseError> = Err(DatabaseError("Database connection failed".to_string()));
    if let Err(e) = result {
        logger.log_error("Failed to connect to database", &e);
    }

    // Step 6: Using context propagation
    println!("\n=== Context Propagation ===");
    let context = ContextBuilder::new()
        .trace_id("abc123def456")
        .span_id("span789")
        .generate_request_id()
        .user_id("user123")
        .tenant_id("tenant001")
        .build();

    let logger_with_context = Logger::with_context(Some("api".to_string()), context);
    logger_with_context.info("Processing request with context");

    // Step 7: Redaction demonstration
    println!("\n=== Redaction (sensitive data will be redacted) ===");
    logger.info_with("User registration", |e| {
        e.field("email", "user@example.com");        // Will be partially redacted
        e.field("password", "secret123");            // Will be fully redacted
        e.field("api_key", "sk_live_abc123xyz456");  // Will be fully redacted
        e.field("phone", "555-123-4567");            // Will be partially redacted
    });
}

