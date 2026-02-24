//! Tests for the built-in CborFormat.

#![cfg(feature = "cbor")]

use std::sync::Arc;

use erased_serde::Serialize as _;
use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};
use serde::{Deserialize, Serialize};
use tower_conneg::{
    CborFormat, ErasedFormat, Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer,
    match_specificity,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    field: String,
    number: i32,
}

#[test]
fn cbor_format_media_types() {
    let format = CborFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 1);
    assert_eq!(
        types[0],
        MediaType::new(APPLICATION, Name::new_unchecked("cbor"))
    );
}

#[test]
fn cbor_format_content_type_header() {
    let format = CborFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(header, HeaderValue::from_static("application/cbor"));
}

#[test]
fn cbor_format_match_specificity_exact() {
    let format = CborFormat;
    let media_type = MediaType::new(APPLICATION, Name::new_unchecked("cbor"));

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn cbor_format_match_specificity_wildcard() {
    let format = CborFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn cbor_format_match_specificity_type_only() {
    let format = CborFormat;
    let media_type = MediaType::parse("application/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn cbor_format_match_specificity_none() {
    let format = CborFormat;
    let media_type = mediatype::media_type!(TEXT / PLAIN);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn cbor_format_roundtrip_serialization() {
    let format = CborFormat;
    let data = TestStruct {
        field: "hello".to_string(),
        number: 42,
    };

    let mut bytes = Vec::new();
    format
        .serializer(&mut bytes)
        .unwrap()
        .with_erased(&mut |ser| data.erased_serialize(ser))
        .unwrap();

    let deserializer = format.deserializer(&bytes).unwrap();
    let result = TestStruct::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(result, data);
}

#[test]
fn cbor_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(CborFormat);
    let data = TestStruct {
        field: "test".to_string(),
        number: 123,
    };

    let mut bytes = Vec::new();
    format
        .serialize(&mut bytes, &mut |serializer| {
            use erased_serde::Serialize;
            data.erased_serialize(serializer)
        })
        .unwrap();

    let mut result: Option<TestStruct> = None;
    format
        .deserialize(&bytes, &mut |deserializer| {
            result = Some(erased_serde::deserialize(deserializer)?);
            Ok(())
        })
        .unwrap();

    assert_eq!(result, Some(data));
}

#[test]
fn cbor_format_binary_compactness() {
    let format = CborFormat;
    let data = TestStruct {
        field: "hello".to_string(),
        number: 42,
    };

    let mut bytes = Vec::new();
    format
        .serializer(&mut bytes)
        .unwrap()
        .with_erased(&mut |ser| data.erased_serialize(ser))
        .unwrap();

    let json_bytes = serde_json::to_vec(&data).unwrap();
    assert!(
        bytes.len() < json_bytes.len(),
        "CBOR ({} bytes) should be more compact than JSON ({} bytes)",
        bytes.len(),
        json_bytes.len()
    );
}

#[test]
fn cbor_format_is_default() {
    let format = CborFormat::default();
    assert_eq!(
        Format::content_type_header(&format),
        HeaderValue::from_static("application/cbor")
    );
}

#[test]
fn cbor_format_is_debug() {
    let format = CborFormat;
    let debug_str = format!("{:?}", format);
    assert_eq!(debug_str, "CborFormat");
}

#[test]
fn cbor_format_is_clone() {
    let format = CborFormat;
    let cloned = format;
    assert_eq!(
        Format::content_type_header(&cloned),
        Format::content_type_header(&format)
    );
}
