//! Configuration types.

use std::sync::Arc;

use bon::Builder;
use http::HeaderValue;

use crate::format::ErasedFormat;

fn build_supported_media_types(formats: &[Arc<dyn ErasedFormat>]) -> Vec<String> {
    formats
        .iter()
        .filter_map(|f| f.content_type_header().to_str().ok().map(str::to_owned))
        .collect()
}

fn build_accept_header_value(formats: &[Arc<dyn ErasedFormat>]) -> Option<HeaderValue> {
    let media_types: Vec<_> = formats.iter().map(|f| f.content_type_header()).collect();
    let value = media_types
        .iter()
        .filter_map(|h| h.to_str().ok())
        .collect::<Vec<_>>()
        .join(", ");
    HeaderValue::from_str(&value).ok()
}

#[allow(clippy::cast_precision_loss)]
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

/// Server-side content negotiation configuration.
///
/// Deserializes request bodies via `Content-Type` and serializes responses via `Accept`.
/// Formats are in priority order; the first is the default for missing/wildcard Accept headers.
#[derive(Debug, Clone, Builder)]
#[builder(finish_fn(vis = "", name = __build))]
pub struct ServerConfig {
    /// Supported formats in priority order.
    #[builder(with = |formats: impl IntoIterator<Item = Arc<dyn ErasedFormat>>| formats.into_iter().collect())]
    pub(crate) formats: Vec<Arc<dyn ErasedFormat>>,
    /// Default format when negotiation fails or formats list is empty.
    pub(crate) fallback_format: Arc<dyn ErasedFormat>,
    /// If true, return 406 when Accept doesn't match. If false, use fallback.
    #[builder(default)]
    pub(crate) strict: bool,
    #[builder(skip)]
    pub(crate) accept_header_value: Option<HeaderValue>,
    #[builder(skip)]
    pub(crate) supported_media_types: Arc<[String]>,
}

impl<S: server_config_builder::IsComplete> ServerConfigBuilder<S> {
    /// Builds the configuration, ensuring formats list is not empty.
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

/// Client-side content negotiation configuration.
///
/// Serializes request bodies and deserializes responses via `Content-Type`.
/// Formats are in priority order for constructing the `Accept` header.
#[derive(Debug, Clone, Builder)]
#[builder(finish_fn(vis = "", name = __build))]
pub struct ClientConfig {
    /// Supported formats in priority order.
    #[builder(with = |formats: impl IntoIterator<Item = Arc<dyn ErasedFormat>>| formats.into_iter().collect())]
    pub(crate) formats: Vec<Arc<dyn ErasedFormat>>,
    /// Default format when 415 response lacks Accept-Post/Accept-Patch header.
    pub(crate) fallback_format: Arc<dyn ErasedFormat>,
    #[builder(skip)]
    pub(crate) accept_header_value: Option<HeaderValue>,
}

impl<S: client_config_builder::IsComplete> ClientConfigBuilder<S> {
    /// Builds the configuration, ensuring formats list is not empty.
    pub fn build(self) -> ClientConfig {
        let mut config = self.__build();
        if config.formats.is_empty() {
            config.formats.push(config.fallback_format.clone());
        }
        config.accept_header_value = build_client_accept_header_value(&config.formats);
        config
    }
}
