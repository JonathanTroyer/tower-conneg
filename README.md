# tower-conneg

Tower middleware for HTTP content negotiation on both client and server.

## Features

- **Server-side**: Deserialize request bodies via `Content-Type`, serialize responses via `Accept`
- **Client-side**: Serialize request bodies, deserialize responses via `Content-Type`
- **Multiple formats**: JSON, MessagePack, CBOR, XML, TOML, Form, Postcard, BSON, plain text
- **Framework integrations**: Axum, Poem, Salvo, Viz
- **Type-safe**: Explicit format threading through handler signatures
- **Serde-based**: Uses `erased-serde` for type erasure at the format level

## Quick Start (Server with Axum)

```rust
use axum::{Router, routing::post};
use tower_conneg::{Negotiate, NegotiateResponse, NegotiateLayer, ServerConfig, JsonFormat};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

async fn create_user(req: Negotiate<CreateUser>) -> NegotiateResponse<User> {
    let user = User { id: 1, name: req.name.clone() };
    req.respond(user)
}

let config = ServerConfig::builder()
    .formats([JsonFormat])
    .build();

let app = Router::new()
    .route("/users", post(create_user))
    .layer(NegotiateLayer::new(config));
```

For endpoints without request bodies (GET, DELETE), use `Negotiate<()>`:

```rust
async fn get_user(neg: Negotiate<()>, Path(id): Path<u64>) -> NegotiateResponse<User> {
    let user = db.get(id).await;
    neg.respond(user)
}
```

## Quick Start (Client)

```rust
use tower_conneg::{ClientNegotiateLayer, ClientConfig, JsonFormat, MsgPackFormat};

let config = ClientConfig::builder()
    .formats([MsgPackFormat, JsonFormat])  // Priority order
    .fallback_format(JsonFormat)
    .build();

let client = ServiceBuilder::new()
    .layer(ClientNegotiateLayer::new(config))
    .service(hyper_client);
```

## Available Formats

| Format | Feature Flag | Media Type |
|--------|--------------|------------|
| JSON | `json` (default) | `application/json` |
| MessagePack | `msgpack` | `application/msgpack` |
| CBOR | `cbor` | `application/cbor` |
| XML | `xml` | `application/xml` |
| TOML | `toml` | `application/toml` |
| Form | `form` | `application/x-www-form-urlencoded` |
| Postcard | `postcard` | `application/x-postcard` |
| BSON | `bson` | `application/bson` |
| Plain Text | `plain` | `text/plain` |
| HTML | `plain` | `text/html` |

## Framework Integrations

| Framework | Feature Flag | Notes |
|-----------|--------------|-------|
| Axum | `axum` | `Negotiate<T>` implements `FromRequest`/`FromRequestParts`, `NegotiateResponse` implements `IntoResponse` |
| Poem | `poem` | Native extractor/responder support |
| Salvo | `salvo` | Native `Extractible` support |
| Viz | `viz` | Native extractor support |
| Hyper Client | `hyper-client` | Client-side negotiation |

## Design

The library uses an extractor/responder pattern where the negotiated format flows explicitly through handler signatures:

1. Middleware parses `Accept` and `Content-Type` headers
2. `Negotiate<T>` extractor deserializes the request body and captures the format
3. Handler calls `.respond(value)` to create a `NegotiateResponse<T>`
4. `IntoResponse` serializes using the captured format

This approach avoids heap allocation for response values and makes data flow visible in type signatures.

## License

MIT
