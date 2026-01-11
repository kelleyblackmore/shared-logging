//! Error handling example

use shared_logging::{init_logger, Logger};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct DatabaseError {
    message: String,
    code: u32,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database error {}: {}", self.code, self.message)
    }
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
struct NetworkError {
    message: String,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Network error: {}", self.message)
    }
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
struct ServiceError {
    message: String,
    source: Box<dyn Error>,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service error: {}", self.message)
    }
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

fn main() {
    init_logger("error-example", "info").expect("Failed to initialize logger");
    let logger = Logger::new(Some("service".to_string()));

    // Example 1: Simple error logging
    println!("\n=== Simple Error ===");
    let db_error = DatabaseError {
        message: "Connection timeout".to_string(),
        code: 1001,
    };
    logger.log_error("Database operation failed", &db_error);

    // Example 2: Error with context
    println!("\n=== Error with Context ===");
    logger.error_with("Failed to process request", |e| {
        e.field("operation", "user_creation");
        e.field("retry_count", 3);
        e.error(&db_error);
    });

    // Example 3: Error chain
    println!("\n=== Error Chain ===");
    let network_error = NetworkError {
        message: "Connection refused".to_string(),
    };
    let service_error = ServiceError {
        message: "Failed to fetch user data".to_string(),
        source: Box::new(network_error),
    };
    logger.log_error("User service failed", &service_error);

    // Example 4: Error with full context
    println!("\n=== Error with Full Context ===");
    logger.error_with("Critical operation failed", |e| {
        e.field("user_id", "user123");
        e.field("operation", "payment_processing");
        e.field("amount", 99.99);
        e.error(&db_error);
    });
}

