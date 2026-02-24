//! Configuration types for content negotiation middleware.

use std::sync::Arc;

use bon::Builder;
use http::HeaderValue;

use crate::ErasedFormat;

/// Builds a list of supported media types as strings for error messages.
fn build_supported_media_types(formats: &[Arc<dyn ErasedFormat>]) -> Vec<String> {
    formats
        .iter()
        .filter_map(|f| f.content_type_header().to_str().ok().map(str::to_owned))
        .collect()
}

/// Builds an Accept header value from a list of formats.
/// Used for Accept-Post/Accept-Patch headers in 415 responses.
fn build_accept_header_value(formats: &[Arc<dyn ErasedFormat>]) -> Option<HeaderValue> {
    let media_types: Vec<_> = formats.iter().map(|f| f.content_type_header()).collect();
    let value = media_types
        .iter()
        .filter_map(|h| h.to_str().ok())
        .collect::<Vec<_>>()
        .join(", ");
    HeaderValue::from_str(&value).ok()
}

/// Builds an Accept header value with quality values for client requests.
/// First format gets implicit q=1.0, subsequent formats get descending q-values
/// evenly distributed from 0.9 to 0.1, formatted to 3 decimal places per RFC 7231.
#[allow(clippy::cast_precision_loss)] // RFC 7231 limits q-values to 3 decimal places
fn build_client_accept_header_value(formats: &[Arc<dyn ErasedFormat>]) -> Option<HeaderValue> {
    let count = formats.len();
    let decrement = if count > 1 { 0.9 / count as f64 } else { 0.0 };

    let parts: Vec<String> = formats
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let media_type = f.content_type_header().to_str().ok()?.to_string();
            if i == 0 {
                Some(media_type)
            } else {
                let q = 1.0 - (i as f64 * decrement);
                Some(format!("{media_type};q={q:.3}"))
            }
        })
        .collect();
    HeaderValue::from_str(&parts.join(", ")).ok()
}

/// Configuration for server-side content negotiation.
///
/// The server uses this configuration to:
/// - Deserialize request bodies based on `Content-Type` header
/// - Serialize responses based on `Accept` header matching
///
/// Formats are stored in priority order. The first format is used as the default
/// when the `Accept` header is missing or contains only wildcards.
///
/// Use [`ServerConfig::builder()`] to construct instances.
#[derive(Debug, Clone, Builder)]
#[builder(finish_fn(vis = "", name = __build))]
pub struct ServerConfig {
    /// Supported formats in priority order.
    #[builder(with = |formats: impl IntoIterator<Item = Arc<dyn ErasedFormat>>| formats.into_iter().collect())]
    pub(crate) formats: Vec<Arc<dyn ErasedFormat>>,
    /// Fallback format used when the formats list is empty or when negotiation
    /// requires a default. This format is automatically added to the formats list
    /// if not already present.
    pub(crate) fallback_format: Arc<dyn ErasedFormat>,
    /// If true, return 406 Not Acceptable when Accept doesn't match any format.
    /// If false, fall back to the fallback format.
    #[builder(default)]
    pub(crate) strict: bool,
    /// Pre-computed Accept header value for 415 responses (Accept-Post/Accept-Patch).
    #[builder(skip)]
    pub(crate) accept_header_value: Option<HeaderValue>,
    /// Pre-computed list of supported media types for error messages.
    #[builder(skip)]
    pub(crate) supported_media_types: Arc<[String]>,
}

impl<S: server_config_builder::IsComplete> ServerConfigBuilder<S> {
    /// Builds the server configuration, ensuring formats list is not empty.
    pub fn build(self) -> ServerConfig {
        let mut config = self.__build();
        if config.formats.is_empty() {
            config.formats.push(config.fallback_format.clone());
        }
        config.accept_header_value = build_accept_header_value(&config.formats);
        config.supported_media_types = build_supported_media_types(&config.formats).into();
        config
    }
}

/// Configuration for client-side content negotiation.
///
/// The client uses this configuration to:
/// - Serialize request bodies
/// - Deserialize responses based on `Content-Type` header
///
/// Formats are stored in priority order for constructing the `Accept` header.
///
/// Use [`ClientConfig::builder()`] to construct instances.
#[derive(Debug, Clone, Builder)]
#[builder(finish_fn(vis = "", name = __build))]
pub struct ClientConfig {
    /// Formats in priority order for requests.
    #[builder(with = |formats: impl IntoIterator<Item = Arc<dyn ErasedFormat>>| formats.into_iter().collect())]
    pub(crate) formats: Vec<Arc<dyn ErasedFormat>>,
    /// Fallback format used when the server returns 415 Unsupported Media Type
    /// without an `Accept-Post` or `Accept-Patch` header indicating supported types.
    /// Also used as the default format if formats list is empty.
    pub(crate) fallback_format: Arc<dyn ErasedFormat>,
    /// Pre-computed Accept header value for requests.
    #[builder(skip)]
    pub(crate) accept_header_value: Option<HeaderValue>,
}

impl<S: client_config_builder::IsComplete> ClientConfigBuilder<S> {
    /// Builds the client configuration, ensuring formats list is not empty.
    pub fn build(self) -> ClientConfig {
        let mut config = self.__build();
        if config.formats.is_empty() {
            config.formats.push(config.fallback_format.clone());
        }
        config.accept_header_value = build_client_accept_header_value(&config.formats);
        config
    }
}
