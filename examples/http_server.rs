//! HTTP server example with middleware

use axum::{
    extract::Request,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use shared_logging::{init_logger, Logger, ContextBuilder};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Initialize logger
    init_logger("http-server", "info").expect("Failed to initialize logger");

    // Create logger for the server module
    let logger = Logger::new(Some("server".to_string()));
    logger.info("Starting HTTP server");

    // Build the application
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/users/:id", get(user_handler))
        .route("/health", get(health_handler))
        .layer(
            ServiceBuilder::new()
                // Add tracing layer for request/response logging
                .layer(TraceLayer::new_for_http())
        );

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    logger.info_with("Server listening", |e| {
        e.field("address", &addr.to_string());
        e.field("port", 3000);
    });

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler() -> Json<Value> {
    let logger = Logger::new(Some("handlers".to_string()));
    logger.info("Root endpoint accessed");
    Json(json!({ "message": "Hello, World!" }))
}

async fn user_handler(axum::extract::Path(user_id): axum::extract::Path<String>) -> Result<Json<Value>, StatusCode> {
    // Extract request ID from headers (in real app, use middleware)
    let context = ContextBuilder::new()
        .generate_request_id()
        .user_id(&user_id)
        .build();
    
    let logger = Logger::with_context(Some("handlers".to_string()), context);
    
    logger.info_with("Fetching user", |e| {
        e.field("user_id", &user_id);
    });

    // Simulate some processing
    if user_id == "error" {
        logger.error("User not found");
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(json!({
        "user_id": user_id,
        "name": "John Doe",
        "email": "john@example.com"
    })))
}

async fn health_handler() -> Json<Value> {
    let logger = Logger::new(Some("handlers".to_string()));
    logger.debug("Health check");
    Json(json!({ "status": "healthy" }))
}

