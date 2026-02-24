//! Response extension trait.

use std::sync::Arc;

use bytes::Bytes;
use http::{Response, header};
use http_body::Body;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;

use crate::core::{NegotiationError, parse_content_type_erased};
use crate::format::ErasedFormat;

/// Extension trait for deserializing responses with content negotiation.
pub trait NegotiateResponseExt<B> {
    /// Returns the format matching the response's `Content-Type` header.
    ///
    /// # Errors
    /// Returns an error if no format matches the Content-Type.
    fn negotiated_format(
        &self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<Arc<dyn ErasedFormat>, NegotiationError>;

    /// Collects the body and deserializes using the matching format.
    fn deserialize<T: DeserializeOwned>(
        self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> impl std::future::Future<Output = Result<T, NegotiationError>> + Send
    where
        B: Body<Data = Bytes> + Send,
        B::Error: std::error::Error + Send + Sync + 'static;
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

        let supported: Vec<String> = formats
            .iter()
            .map(|f| f.primary_media_type().to_string())
            .collect();

        match content_type {
            Some(ct) => parse_content_type_erased(ct, formats)
                .ok_or_else(|| NegotiationError::unsupported_media_type(Some(ct), &supported)),
            None => Err(NegotiationError::unsupported_media_type(None, &supported)),
        }
    }

    async fn deserialize<T: DeserializeOwned>(
        self,
        formats: &[Arc<dyn ErasedFormat>],
    ) -> Result<T, NegotiationError>
    where
        B: Body<Data = Bytes> + Send,
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        let format = self.negotiated_format(formats)?;
        let body = self.into_body();

        let bytes = body
            .collect()
            .await
            .map_err(NegotiationError::body_collection)?
            .to_bytes();

        let mut result: Option<T> = None;
        format
            .deserialize(&bytes, &mut |deserializer| {
                result = Some(erased_serde::deserialize(deserializer)?);
                Ok(())
            })
            .map_err(NegotiationError::deserialization)?;

        result.ok_or_else(NegotiationError::deserialization_produced_no_value)
    }
}
