//! Salvo framework integration for content negotiation.
//!
//! Provides [`Extractible`] implementation for [`Negotiate<T>`], and
//! [`Writer`] implementations for [`NegotiateResponse<T>`] and
//! [`NegotiationError`].

use std::sync::Arc;

use salvo::extract::{Extractible, Metadata};
use salvo::http::header;
use salvo::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{deserialize_body, deserialize_unit, get_negotiated_format, serialize_body};
use crate::core::{Negotiate, NegotiateResponse, NegotiationError};

impl<'ex, T> Extractible<'ex> for Negotiate<T>
where
    T: DeserializeOwned + Send,
{
    fn metadata() -> &'static Metadata {
        static METADATA: Metadata = Metadata::new("Negotiate");
        &METADATA
    }

    async fn extract(
        req: &'ex mut Request,
    ) -> Result<Self, impl Writer + Send + std::fmt::Debug + 'static> {
        let negotiated = get_negotiated_format(req.extensions())?.clone();
        let response_format = Arc::clone(negotiated.response_format());

        let value = match negotiated.request_format() {
            Some(fmt) => {
                let bytes = req
                    .payload()
                    .await
                    .map_err(NegotiationError::body_collection)?;
                deserialize_body(bytes, fmt.as_ref())?
            }
            None => deserialize_unit()?,
        };

        Ok::<_, NegotiationError>(Negotiate::new(value, response_format))
    }
}

#[async_trait]
impl<T: Serialize + Send + Sync> Writer for NegotiateResponse<T> {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let format = Arc::clone(self.format());
        let content_type = format.content_type_header();
        let value = self.into_inner();

        match serialize_body(&value, format.as_ref()) {
            Ok(bytes) => {
                res.headers_mut().insert(header::CONTENT_TYPE, content_type);
                res.body(bytes);
            }
            Err(e) => e.write(_req, _depot, res).await,
        }
    }
}

#[async_trait]
impl Writer for NegotiationError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code(self.status_code());
        res.body(self.to_string());
    }
}
