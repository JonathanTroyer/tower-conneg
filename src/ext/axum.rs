//! Axum framework integration for content negotiation.
//!
//! Provides [`FromRequest`] implementation for [`Negotiate<T>`], and
//! [`IntoResponse`] implementations for [`NegotiateResponse<T>`] and
//! [`NegotiationError`].

use std::sync::Arc;

use axum::body::Body;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use http::{Request, header};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{deserialize_body, deserialize_unit, get_negotiated_format, serialize_body};
use crate::core::{Negotiate, NegotiateResponse, NegotiationError};

impl<S, T> FromRequest<S> for Negotiate<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = NegotiationError;

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let negotiated = get_negotiated_format(&parts.extensions)?;
        let response_format = Arc::clone(negotiated.response_format());

        let value = match negotiated.request_format() {
            Some(request_format) => {
                let bytes = axum::body::to_bytes(body, usize::MAX)
                    .await
                    .map_err(NegotiationError::body_collection)?;
                deserialize_body(&bytes, request_format.as_ref())?
            }
            None => deserialize_unit()?,
        };

        Ok(Negotiate::new(value, response_format))
    }
}

impl<T: Serialize> IntoResponse for NegotiateResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let format = Arc::clone(self.format());
        let content_type = format.content_type_header();
        let value = self.into_inner();

        match serialize_body(&value, format.as_ref()) {
            Ok(bytes) => ([(header::CONTENT_TYPE, content_type)], bytes).into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for NegotiationError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
