//! Tests for the built-in BsonFormat.

#![cfg(feature = "bson")]

use std::sync::Arc;

use erased_serde::Serialize as _;
use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};
use serde::{Deserialize, Serialize};
use tower_conneg::{
    BsonFormat, ErasedFormat, Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer,
    match_specificity,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    field: String,
    number: i32,
}

#[test]
fn bson_format_media_types() {
    let format = BsonFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 1);
    assert_eq!(
        types[0],
        MediaType::new(APPLICATION, Name::new_unchecked("bson"))
    );
}

#[test]
fn bson_format_content_type_header() {
    let format = BsonFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(header, HeaderValue::from_static("application/bson"));
}

#[test]
fn bson_format_match_specificity_exact() {
    let format = BsonFormat;
    let media_type = MediaType::new(APPLICATION, Name::new_unchecked("bson"));

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn bson_format_match_specificity_wildcard() {
    let format = BsonFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn bson_format_match_specificity_type_only() {
    let format = BsonFormat;
    let media_type = MediaType::parse("application/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn bson_format_match_specificity_none() {
    let format = BsonFormat;
    let media_type = mediatype::media_type!(TEXT / PLAIN);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn bson_format_roundtrip_serialization() {
    let format = BsonFormat;
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
fn bson_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(BsonFormat);
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
