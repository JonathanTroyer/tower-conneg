//! Poem framework integration for content negotiation.

use std::sync::Arc;

use poem::error::ResponseError;
use poem::http::header;
use poem::{FromRequest, IntoResponse, Request, RequestBody, Response, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{deserialize_body, deserialize_unit, get_negotiated_format, serialize_body};
use crate::core::{Negotiate, NegotiateResponse, NegotiationError};

impl ResponseError for NegotiationError {
    fn status(&self) -> poem::http::StatusCode {
        self.status_code()
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for Negotiate<T> {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let negotiated = get_negotiated_format(req.extensions())?;
        let response_format = Arc::clone(negotiated.response_format());

        let value = match negotiated.request_format() {
            Some(fmt) => {
                let bytes = body
                    .take()?
                    .into_bytes()
                    .await
                    .map_err(NegotiationError::body_collection)?;
                deserialize_body(&bytes, fmt.as_ref())?
            }
            None => deserialize_unit()?,
        };

        Ok(Negotiate::new(value, response_format))
    }
}

impl<T: Serialize + Send> IntoResponse for NegotiateResponse<T> {
    fn into_response(self) -> Response {
        let format = Arc::clone(self.format());
        let content_type = format.content_type_header();
        let value = self.into_inner();

        match serialize_body(&value, format.as_ref()) {
            Ok(bytes) => Response::builder()
                .header(header::CONTENT_TYPE, content_type)
                .body(bytes),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for NegotiationError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.status_code())
            .body(self.to_string())
    }
}
