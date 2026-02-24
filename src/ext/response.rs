//! Response extension trait.

use std::sync::Arc;

use bytes::Bytes;
use http::{Response, header};
use http_body::Body;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;

use crate::content_type::parse_content_type_erased;
use crate::error::NegotiationError;
use crate::format::ErasedFormat;

/// Extension trait for parsing responses with content negotiation.
pub trait NegotiateResponseExt<B> {
    /// Finds the format matching the response's `Content-Type` header.
    ///
    /// # Errors
    ///
    /// Returns `NegotiationError::UnsupportedMediaType` if no format matches.
    fn negotiated_format(
        &self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<Arc<dyn ErasedFormat>, NegotiationError>;

    /// Collects the body and deserializes using the matching format.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No format matches the `Content-Type` header
    /// - Body collection fails
    /// - Deserialization fails
    fn deserialize<T: DeserializeOwned>(
        self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> impl std::future::Future<Output = Result<T, NegotiationError>> + Send
    where
        B: Body<Data = Bytes> + Send,
        B::Error: std::fmt::Display;
}

impl<B> NegotiateResponseExt<B> for Response<B> {
    fn negotiated_format(
        &self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<Arc<dyn ErasedFormat>, NegotiationError> {
        let content_type = self
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());

        let supported: Arc<[String]> = formats
            .iter()
            .map(|f| f.primary_media_type().to_string())
            .collect();

        match content_type {
            Some(ct) => parse_content_type_erased(ct, formats).ok_or_else(|| {
                NegotiationError::UnsupportedMediaType {
                    provided: Some(ct.to_owned()),
                    supported,
                }
            }),
            None => Err(NegotiationError::UnsupportedMediaType {
                provided: None,
                supported,
            }),
        }
    }

    async fn deserialize<T: DeserializeOwned>(
        self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<T, NegotiationError>
    where
        B: Body<Data = Bytes> + Send,
        B::Error: std::fmt::Display,
    {
        let format = self.negotiated_format(formats)?;
        let body = self.into_body();

        let bytes = body
            .collect()
            .await
            .map_err(|e| NegotiationError::BodyCollection {
                source: e.to_string(),
            })?
            .to_bytes();

        let mut result: Option<T> = None;
        format
            .deserialize(&bytes, &mut |deserializer| {
                result = Some(erased_serde::deserialize(deserializer)?);
                Ok(())
            })
            .map_err(|e| NegotiationError::Deserialization {
                source: e.to_string(),
            })?;

        result.ok_or_else(|| NegotiationError::Deserialization {
            source: "deserializer callback never ran".to_string(),
        })
    }
}
