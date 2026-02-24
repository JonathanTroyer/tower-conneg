//! Basic server example demonstrating content negotiation with tower-conneg.
//!
//! This example shows how to:
//! - Configure multiple serialization formats (JSON and MessagePack)
//! - Use `Negotiate<()>` for GET endpoints (no request body)
//! - Use `Negotiate<T>` for POST endpoints (with request body)
//! - Handle errors using `Result<NegotiateResponse<T>, NegotiateResponse<E>>`
//!
//! Run with: cargo run --example server --features "axum,json,msgpack"
//! Test with:
//!   curl -H "Accept: application/json" http://localhost:3000/users/1
//!   curl -H "Accept: application/msgpack" http://localhost:3000/users/1
//!   curl -X POST -H "Content-Type: application/json" -H "Accept: application/json" \
//!        -d '{"name":"Alice","email":"alice@example.com"}' http://localhost:3000/users

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stdout)]

use std::sync::Arc;

use axum::Router;
use axum::extract::Path;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_conneg::{
    ErasedFormat, JsonFormat, MsgPackFormat, Negotiate, NegotiateLayer, NegotiateResponse,
    ServerConfig,
};

// Domain types - these are your application's data models

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize)]
struct ApiError {
    code: String,
    message: String,
}

// Configuration - set up supported formats with a fallback

fn build_config() -> ServerConfig {
    // Create format instances as Arc<dyn ErasedFormat> for type erasure
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let msgpack: Arc<dyn ErasedFormat> = Arc::new(MsgPackFormat);

    // Build server config with multiple formats
    // The order matters: first format is used when client sends Accept: */*
    ServerConfig::builder()
        .formats(vec![json.clone(), msgpack])
        .fallback_format(json) // Used when no Accept header or no match
        .build()
}

// GET endpoint with path parameter
// Negotiate<()> must come LAST since it implements FromRequest (consumes body)

async fn get_user(
    Path(id): Path<u64>,
    neg: Negotiate<()>,
) -> Result<NegotiateResponse<User>, NegotiateResponse<ApiError>> {
    // Simulate database lookup
    if id == 0 {
        // Error case: use the same format negotiation for error responses
        return Err(neg.respond(ApiError {
            code: "NOT_FOUND".to_string(),
            message: format!("User with id {} not found", id),
        }));
    }

    // Success: respond with the negotiated format
    Ok(neg.respond(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    }))
}

// POST endpoint - uses Negotiate<T> to deserialize the request body
// The format for deserialization comes from Content-Type header
// The format for response comes from Accept header

async fn create_user(
    req: Negotiate<CreateUserRequest>,
) -> Result<NegotiateResponse<User>, NegotiateResponse<ApiError>> {
    // Access the deserialized body via Deref
    if req.name.is_empty() {
        return Err(req.respond(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "Name cannot be empty".to_string(),
        }));
    }

    // Create user (in real app, this would go to a database)
    let user = User {
        id: 42, // Simulated auto-generated ID
        name: req.name.clone(),
        email: req.email.clone(),
    };

    // Use .respond() to create response with the negotiated format
    Ok(req.respond(user))
}

// Alternative: GET endpoint returning a list

async fn list_users(neg: Negotiate<()>) -> NegotiateResponse<Vec<User>> {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    neg.respond(users)
}

#[tokio::main]
async fn main() {
    // Build the content negotiation configuration
    let config = build_config();

    // Create router with NegotiateLayer applied
    // The layer must wrap routes that use Negotiate<T> extractors
    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user))
        .layer(NegotiateLayer::new(config));

    // Start the server
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    println!();
    println!("Try these requests:");
    println!("  curl -H 'Accept: application/json' http://localhost:3000/users");
    println!("  curl -H 'Accept: application/msgpack' http://localhost:3000/users/1");
    println!("  curl -X POST -H 'Content-Type: application/json' -H 'Accept: application/json' \\");
    println!(
        "       -d '{{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}' http://localhost:3000/users"
    );

    axum::serve(listener, app).await.unwrap();
}
