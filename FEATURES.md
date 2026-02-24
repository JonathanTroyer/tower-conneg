# Planned Features for tower-conneg

This document outlines feature-gated convenience features planned for tower-conneg.

## Feature Flag Structure

```toml
[features]
default = ["json"]

# Framework integrations
axum = ["dep:axum"]
poem = ["dep:poem"]
salvo = ["dep:salvo"]
viz = ["dep:viz"]

# Client helpers
hyper-client = ["dep:hyper", "dep:hyper-util"]
reqwest = ["dep:reqwest"]

# Serialization formats - Primary
json = ["dep:serde_json"]
msgpack = ["dep:rmp-serde"]
cbor = ["dep:ciborium"]
xml = ["dep:quick-xml"]

# Serialization formats - Secondary
bincode = ["dep:bincode"]
postcard = ["dep:postcard"]
bson = ["dep:bson"]
toml = ["dep:toml"]
form = ["dep:serde_urlencoded"]
csv = ["dep:csv"]

# Convenience bundles
binary = ["msgpack", "cbor", "bincode"]
web = ["json", "xml", "form"]
all-formats = ["json", "msgpack", "cbor", "xml", "bincode", "postcard", "bson", "toml", "form", "csv"]
```

---

## Server Framework Helpers

### 1. Axum Integration (`axum` feature) - PRIORITY: HIGH

Native Tower support makes Axum the natural fit.

**Traits to Implement:**

| Trait | Type | Purpose |
|-------|------|---------|
| `FromRequestParts<S>` | `Negotiate<()>` | Extract negotiation context without body |
| `FromRequest<S>` | `Negotiate<T>` | Extract + deserialize request body |
| `IntoResponse` | `NegotiateResponse<T>` | Serialize response with negotiated format |
| `IntoResponse` | `NegotiationError` | Return 406/415 with proper headers |

**Example API:**

```rust
use axum::{Router, routing::post};
use tower_conneg::{Negotiate, NegotiateResponse};

async fn create_user(req: Negotiate<CreateUser>) -> NegotiateResponse<User> {
    let user = db.create(&*req).await;
    req.respond(user)
}

async fn get_user(neg: Negotiate<()>, Path(id): Path<u64>) -> NegotiateResponse<User> {
    neg.respond(db.get(id).await)
}

let app = Router::new()
    .route("/users", post(create_user))
    .layer(NegotiateLayer::new(config));
```

**Dependencies:** `axum`, `async-trait`

---

### 2. Poem Integration (`poem` feature) - PRIORITY: MEDIUM

Poem has Tower compatibility via `TowerLayerCompat`.

**Traits to Implement:**

| Trait | Type | Purpose |
|-------|------|---------|
| `FromRequest<'a>` | `Negotiate<T>` | Extract + deserialize |
| `IntoResponse` | `NegotiateResponse<T>` | Serialize response |

**Example API:**

```rust
use poem::{Route, post};
use poem::middleware::TowerLayerCompat;
use tower_conneg::{Negotiate, NegotiateResponse};

async fn create_user(req: Negotiate<CreateUser>) -> NegotiateResponse<User> {
    req.respond(db.create(&*req).await)
}

let app = Route::new()
    .at("/users", post(create_user))
    .with(TowerLayerCompat::new(NegotiateLayer::new(config)));
```

**Dependencies:** `poem`, `async-trait`

---

### 3. Salvo Integration (`salvo` feature) - PRIORITY: MEDIUM

Native Tower Layer support with 9.2M downloads. Uses Handler/Writer pattern.

**Traits to Implement:**

| Trait | Type | Purpose |
|-------|------|---------|
| `Writer` | `NegotiateResponse<T>` | Write serialized response |
| Custom extractor | `Negotiate<T>` | Extract from Depot/extensions |

**Example API:**

```rust
use salvo::prelude::*;
use tower_conneg::{Negotiate, NegotiateResponse};

#[handler]
async fn create_user(depot: &mut Depot) -> NegotiateResponse<User> {
    let negotiate = depot.obtain::<Negotiate<CreateUser>>().unwrap();
    negotiate.respond(db.create(&*negotiate).await)
}

let router = Router::new()
    .push(Router::with_path("/users").post(create_user))
    .hoop(TowerLayerCompat::new(NegotiateLayer::new(config)));
```

**Dependencies:** `salvo`, `async-trait`

---

### 4. Viz Integration (`viz` feature) - PRIORITY: MEDIUM

Lightweight framework with explicit Tower Service support. Safety-focused (`#![forbid(unsafe_code)]`).

**Traits to Implement:**

| Trait | Type | Purpose |
|-------|------|---------|
| `FromRequest` | `Negotiate<T>` | Extract + deserialize |
| `IntoResponse` | `NegotiateResponse<T>` | Serialize response |

**Example API:**

```rust
use viz::{Router, Request, Result};
use tower_conneg::{Negotiate, NegotiateResponse};

async fn create_user(req: Request) -> Result<NegotiateResponse<User>> {
    let negotiate: Negotiate<CreateUser> = req.extract().await?;
    Ok(negotiate.respond(db.create(&*negotiate).await))
}

let app = Router::new()
    .post("/users", create_user)
    .with(NegotiateLayer::new(config));
```

**Dependencies:** `viz`

---

### 5. Other Frameworks

#### Tower-Native (Already Compatible)

| Framework | Status | Notes |
|-----------|--------|-------|
| **Loco** | Compatible | Built on Axum - works via `axum` feature |
| **Warp** | Skip | Maintenance mode, filter combinator pattern awkward fit |

#### Non-Tower (Would Require Custom Implementation)

| Framework | Downloads | Recommendation |
|-----------|-----------|----------------|
| **Rocket** | 10.1M | Consider if demand - popular but high effort |
| **Dropshot** | 539K | Consider - API-focused with OpenAPI generation |
| **Trillium** | 197K | Consider - multi-runtime (tokio/async-std/smol) |
| **Actix-web** | 56.7M | Skip - own middleware system, violates Tower-only principle |
| **Tide** | 2.7M | Skip - maintenance mode, async-std declining |
| **Gotham** | Low | Skip - seeking new maintainers |

#### Monitor for Future

| Framework | Notes |
|-----------|-------|
| **Pavex** | Compile-time framework in beta (v0.2.x) - interesting once stable |

---

## Client Library Helpers

### 1. Generic Request Builder Extensions - PRIORITY: HIGH

Works with any `http::Request` builder - completely client-agnostic.

**Traits to Implement:**

```rust
pub trait NegotiateRequestBuilderExt {
    fn body_with_format<T: Serialize>(
        self,
        value: &T,
        format: Arc<dyn ErasedFormat>,
    ) -> Result<Request<Full<Bytes>>, Error>;

    fn accept_formats(self, formats: &[Arc<dyn ErasedFormat>]) -> Self;
}
```

**Example API:**

```rust
use tower_conneg::NegotiateRequestBuilderExt;

// Works with raw http::Request
let request = http::Request::builder()
    .uri("https://api.example.com/users")
    .method(Method::POST)
    .accept_formats(&[json_format.clone(), xml_format.clone()])
    .body_with_format(&user, json_format)?;

// Same API works with hyper, reqwest, or any HTTP client
```

---

### 2. Response Parsing Helpers - PRIORITY: HIGH

Works with any `http::Response` - completely client-agnostic.

**Traits to Implement:**

```rust
pub trait NegotiateResponseExt<B> {
    async fn deserialize<T: DeserializeOwned>(
        self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<T, NegotiationError>;

    fn negotiated_format(
        &self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<Arc<dyn ErasedFormat>, NegotiationError>;
}
```

**Example API:**

```rust
use tower_conneg::NegotiateResponseExt;

// Works with any http::Response<B> where B: Body
let user: User = response
    .deserialize(&[json_format, xml_format])
    .await?;

// Get format without consuming response
let format = response.negotiated_format(&[json_format, xml_format])?;
```

---

### 3. Hyper Client Integration (`hyper-client` feature) - PRIORITY: HIGH

Convenience wrapper for hyper clients. Uses the generic request/response helpers under the hood.

**Types to Implement:**

```rust
pub trait HyperClientExt {
    fn with_content_negotiation(self, config: ClientConfig) -> NegotiatedClient<Self>;
}

pub struct NegotiatedClient<S> {
    inner: S,
    config: ClientConfig,
}
```

**Example API:**

```rust
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tower_conneg::HyperClientExt;

// Wrap a hyper client with content negotiation
let client = Client::builder(TokioExecutor::new())
    .build_http()
    .with_content_negotiation(config);

// Build requests using generic helpers
let request = http::Request::builder()
    .uri("https://api.example.com/users")
    .method(Method::POST)
    .body_with_format(&new_user, json_format)?;

// Response parsing uses generic helpers
let response = client.request(request).await?;
let user: User = response.deserialize(&config.formats).await?;
```

**Dependencies:** `hyper = "1"`, `hyper-util = "0.1"`

---

### 4. Reqwest Integration (`reqwest` feature) - PRIORITY: HIGH

Higher-level convenience wrapper for reqwest. Provides a more ergonomic builder API on top of the generic helpers.

**Types to Implement:**

```rust
pub trait ReqwestNegotiateExt {
    fn with_content_negotiation(self, config: ClientConfig) -> NegotiatedClient<Self>;
}

pub struct NegotiatedClient<S> {
    inner: S,
    config: ClientConfig,
}

impl NegotiatedClient<reqwest::Client> {
    pub fn post<T>(&self, url: impl IntoUrl) -> NegotiatedRequestBuilder<'_, T>;
    pub fn get<T>(&self, url: impl IntoUrl) -> NegotiatedRequestBuilder<'_, T>;
    pub fn put<T>(&self, url: impl IntoUrl) -> NegotiatedRequestBuilder<'_, T>;
    // ... other HTTP methods
}

pub struct NegotiatedRequestBuilder<'a, T> {
    // Wraps reqwest::RequestBuilder with content negotiation
}

impl<'a, T: Serialize> NegotiatedRequestBuilder<'a, T> {
    pub fn body(self, value: &T) -> Self;
    pub fn send(self) -> impl Future<Output = Result<NegotiatedResponse>>;
}

pub struct NegotiatedResponse {
    // Wraps reqwest::Response with negotiation helpers
}

impl NegotiatedResponse {
    pub async fn deserialize<T: DeserializeOwned>(self) -> Result<T, NegotiationError>;
}
```

**Example API:**

```rust
use tower_conneg::ReqwestNegotiateExt;

let client = reqwest::Client::new().with_content_negotiation(config);

// High-level builder API
let user: User = client
    .post("https://api.example.com/users")
    .body(&new_user)
    .send()
    .await?
    .deserialize()
    .await?;

// Or use generic helpers directly with reqwest
let request = client
    .post("https://api.example.com/users")
    .build_http_request(|builder| {
        builder.body_with_format(&new_user, json_format)
    })?;
```

**Dependencies:** `reqwest = "0.13"`

---

### 5. Retry Helper - PRIORITY: HIGH

Composable utility for 415 retry logic. Works with any Tower service - not client-specific.

**Types to Implement:**

```rust
pub struct Retry415Helper {
    config: ClientConfig,
    max_attempts: usize,
}

impl Retry415Helper {
    pub fn new(config: ClientConfig, max_attempts: usize) -> Self;

    pub async fn call<S, ReqBody, ResBody>(
        &self,
        service: S,
        request_fn: impl Fn(Arc<dyn ErasedFormat>) -> Request<ReqBody>,
    ) -> Result<Response<ResBody>, Error>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>;
}
```

**Example API:**

```rust
// Generic pattern - works with any Tower service
let helper = Retry415Helper::new(config, 3);

let response = helper.call(service, |format| {
    http::Request::builder()
        .uri("/users")
        .method(Method::POST)
        .body_with_format(&user, format)
        .unwrap()
}).await?;

// With hyper client
use hyper_util::client::legacy::Client;

let hyper_client = Client::builder(TokioExecutor::new()).build_http();

let response = helper.call(hyper_client, |format| {
    http::Request::builder()
        .uri("https://api.example.com/users")
        .body_with_format(&user, format)
        .unwrap()
}).await?;

let user: User = response.deserialize(&config.formats).await?;

// With reqwest (using tower-reqwest or similar adapter)
let reqwest_service = tower_reqwest::ReqwestService::new(reqwest_client);

let response = helper.call(reqwest_service, |format| {
    http::Request::builder()
        .uri("https://api.example.com/users")
        .body_with_format(&user, format)
        .unwrap()
}).await?;
```

---

### 6. Error Handling Improvements - PRIORITY: MEDIUM

Enhanced error types with recovery suggestions.

```rust
impl NegotiationError {
    pub fn is_recoverable(&self) -> bool;
    pub fn suggested_format(
        &self,
        available: &[Arc<dyn ErasedFormat>],
    ) -> Option<Arc<dyn ErasedFormat>>;
}
```

---

## Built-in Serialization Formats

### Tier 1: Essential

| Feature | Crate | Media Types | Notes |
|---------|-------|-------------|-------|
| `json` | `serde_json` | `application/json` | Default, universal baseline |
| `form` | `serde_urlencoded` | `application/x-www-form-urlencoded` | HTML forms |

### Tier 2: High Priority

| Feature | Crate | Media Types | Notes |
|---------|-------|-------------|-------|
| `msgpack` | `rmp-serde` | `application/msgpack`, `application/x-msgpack` | Compact binary, widely used |
| `cbor` | `ciborium` | `application/cbor` | IETF standard (RFC 8949), IoT/embedded |
| `xml` | `quick-xml` | `application/xml`, `text/xml` | Enterprise/legacy APIs, 10x faster than serde-xml-rs |

### Tier 3: Medium Priority

| Feature | Crate | Media Types | Notes |
|---------|-------|-------------|-------|
| `bincode` | `bincode` | `application/x-bincode` | Very fast, Rust-specific, schema-sensitive |
| `toml` | `toml` | `application/toml` | Configuration files |
| `postcard` | `postcard` | `application/x-postcard` | no_std, embedded-friendly |
| `bson` | `bson` | `application/bson` | MongoDB compatibility |
| `csv` | `csv` | `text/csv` | Tabular data export |

### Tier 4: Future Consideration

| Feature | Crate | Media Types | Notes |
|---------|-------|-------------|-------|
| `ndjson` | `serde_json` wrapper | `application/x-ndjson` | Streaming JSON |
| `json5` | `serde_json5` | `application/json5` | Relaxed JSON syntax |
| `ron` | `ron` | `application/x-ron` | Rust-specific readable format |

### Excluded Formats

| Format | Reason |
|--------|--------|
| YAML (`serde_yaml`) | **Deprecated** since 2024-03-25, unmaintained |
| Protocol Buffers | Schema-based, doesn't fit serde model |
| FlatBuffers | Schema-based, complex, poor Rust support |
| Cap'n Proto | No serde support |
| Avro | Schema-dependent |
| rkyv | Own trait system, not serde-compatible |
| FlexBuffers | Poor performance |
| Multipart | Doesn't fit Format model (streaming/files) |

---

## Implementation Priority

### Phase 1: Core Conveniences

1. Generic request builder extensions
2. Response parsing helpers
3. JSON format (`json` feature)
4. URL-encoded forms (`form` feature)

### Phase 2: Popular Clients

5. Hyper client integration
6. Reqwest integration
7. Retry helper

### Phase 3: Primary Formats

8. MessagePack (`msgpack`)
9. CBOR (`cbor`)
10. XML (`xml`)

### Phase 4: Framework Integrations

11. Axum integration (`axum`)
12. Poem integration (`poem`)
13. Salvo integration (`salvo`)
14. Viz integration (`viz`)

### Phase 5: Secondary Formats

15. Bincode, TOML, Postcard, BSON, CSV

---

## Notes

- All formats must work with `erased-serde` for type erasure
- Use IANA-registered media types where available
- Use `application/x-*` prefix for non-standard formats
- Feature flags use Cargo's `dep:` syntax to avoid exposing dependencies
- `resolver = "2"` ensures unused optional deps aren't compiled
