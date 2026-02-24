//! Viz framework integration for content negotiation.

use std::sync::Arc;

use http::header;
use http_body_util::Full;
use serde::Serialize;
use serde::de::DeserializeOwned;
use viz::{Bytes, FromRequest, IntoResponse, Request, RequestExt, Response};

use super::{deserialize_body, deserialize_unit, get_negotiated_format, serialize_body};
use crate::core::{Negotiate, NegotiateResponse, NegotiationError};

impl<T: DeserializeOwned> FromRequest for Negotiate<T> {
    type Error = NegotiationError;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        let negotiated = get_negotiated_format(req.extensions())?.clone();
        let response_format = Arc::clone(negotiated.response_format());

        let value = match negotiated.request_format() {
            Some(fmt) => {
                let bytes = req
                    .bytes()
                    .await
                    .map_err(NegotiationError::body_collection)?;
                deserialize_body(&bytes, fmt.as_ref())?
            }
            None => deserialize_unit()?,
        };

        Ok(Negotiate::new(value, response_format))
    }
}

impl<T: Serialize> IntoResponse for NegotiateResponse<T> {
    fn into_response(self) -> Response {
        let format = Arc::clone(self.format());
        let content_type = format.content_type_header();
        let value = self.into_inner();

        match serialize_body(&value, format.as_ref()) {
            Ok(bytes) => {
                let mut response = Response::new(Full::new(Bytes::from(bytes)).into());
                response
                    .headers_mut()
                    .insert(header::CONTENT_TYPE, content_type);
                response
            }
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for NegotiationError {
    fn into_response(self) -> Response {
        let mut response = Response::new(Full::new(Bytes::from(self.to_string())).into());
        *response.status_mut() = self.status_code();
        response
    }
}
