//! Extension traits for request builders and responses.

#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
use serde::Serialize;
#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
use serde::de::DeserializeOwned;

#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
use crate::core::{NegotiatedFormat, NegotiationError};
#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
use crate::format::ErasedFormat;

#[cfg(feature = "axum")]
mod axum;
#[cfg(feature = "hyper-client")]
mod hyper;
#[cfg(feature = "poem")]
mod poem;
mod request;
mod response;
#[cfg(feature = "salvo")]
mod salvo;
#[cfg(feature = "viz")]
mod viz;

pub use request::NegotiateRequestBuilderExt;
pub use response::NegotiateResponseExt;

#[cfg(feature = "hyper-client")]
pub use self::hyper::HyperClientExt;

#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
pub(crate) fn get_negotiated_format(
    extensions: &http::Extensions,
) -> Result<&NegotiatedFormat, NegotiationError> {
    extensions
        .get::<NegotiatedFormat>()
        .ok_or_else(|| NegotiationError::not_acceptable(None, &[]))
}

#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
pub(crate) fn deserialize_body<T: DeserializeOwned>(
    bytes: &[u8],
    format: &dyn ErasedFormat,
) -> Result<T, NegotiationError> {
    let mut value: Option<T> = None;
    format
        .deserialize(bytes, &mut |deserializer| {
            value = Some(erased_serde::deserialize(deserializer)?);
            Ok(())
        })
        .map_err(NegotiationError::deserialization)?;

    value.ok_or_else(NegotiationError::deserialization_produced_no_value)
}

#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
pub(crate) fn deserialize_unit<T: DeserializeOwned>() -> Result<T, NegotiationError> {
    serde::de::Deserialize::deserialize(serde::de::value::UnitDeserializer::new())
        .map_err(|e: serde::de::value::Error| NegotiationError::deserialization(e))
}

#[cfg(any(feature = "axum", feature = "poem", feature = "salvo", feature = "viz"))]
pub(crate) fn serialize_body<T: Serialize>(
    value: &T,
    format: &dyn ErasedFormat,
) -> Result<Vec<u8>, NegotiationError> {
    let mut bytes = Vec::new();
    format
        .serialize(&mut bytes, &mut |serializer| {
            use erased_serde::Serialize;
            value.erased_serialize(serializer)
        })
        .map_err(NegotiationError::serialization)?;
    Ok(bytes)
}
