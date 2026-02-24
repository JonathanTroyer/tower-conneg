//! Request builder extension trait.

use std::sync::Arc;

use bytes::Bytes;
use http::{Request, header};
use http_body_util::Full;
use serde::Serialize;

use crate::error::NegotiationError;
use crate::format::ErasedFormat;

/// Extension trait for building requests with content negotiation.
pub trait NegotiateRequestBuilderExt {
    /// Sets the `Accept` header with quality values for the given formats.
    ///
    /// Formats are listed in priority order (highest priority first), with
    /// decreasing quality values (q=1.0, q=0.9, q=0.8, ...).
    #[must_use]
    fn accept_formats(self, formats: &[Arc<dyn ErasedFormat>]) -> Self;

    /// Serializes the value using the given format and builds the request.
    ///
    /// Sets the `Content-Type` header and request body.
    ///
    /// # Errors
    ///
    /// Returns `NegotiationError::Serialization` if serialization fails.
    fn body_with_format<T: Serialize>(
        self,
        value: &T,
        format: &dyn ErasedFormat,
    ) -> Result<Request<Full<Bytes>>, NegotiationError>;
}

impl NegotiateRequestBuilderExt for http::request::Builder {
    fn accept_formats(mut self, formats: &[Arc<dyn ErasedFormat>]) -> Self {
        if formats.is_empty() {
            return self;
        }

        let mut accept = String::new();
        for (i, format) in formats.iter().enumerate() {
            if !accept.is_empty() {
                accept.push_str(", ");
            }
            accept.push_str(&format.primary_media_type().to_string());

            #[allow(clippy::cast_precision_loss)]
            let q = 1.0 - (i as f32 * 0.1);
            if q < 1.0 {
                use std::fmt::Write;
                let _ = write!(accept, ";q={q:.1}");
            }
        }

        if let Ok(value) = accept.parse::<http::HeaderValue>() {
            self = self.header(header::ACCEPT, value);
        }
        self
    }

    fn body_with_format<T: Serialize>(
        self,
        value: &T,
        format: &dyn ErasedFormat,
    ) -> Result<Request<Full<Bytes>>, NegotiationError> {
        let mut bytes = Vec::new();
        format
            .serialize(&mut bytes, &mut |serializer| {
                use erased_serde::Serialize;
                value.erased_serialize(serializer)
            })
            .map_err(|e| NegotiationError::Serialization {
                source: e.to_string(),
            })?;

        self.header(header::CONTENT_TYPE, format.content_type_header())
            .body(Full::new(bytes.into()))
            .map_err(|e| NegotiationError::Serialization {
                source: e.to_string(),
            })
    }
}
