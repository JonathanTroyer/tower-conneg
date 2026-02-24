//! Tests for the built-in JsonFormat.

#![cfg(feature = "json")]

use std::sync::Arc;

use http::HeaderValue;
use mediatype::MediaType;
use serde::{Deserialize, Serialize};
use tower_conneg::{
    ErasedFormat, Format, JsonFormat, MatchSpecificity, OwnedDeserializer, OwnedSerializer,
    match_specificity,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    field: String,
    number: i32,
}

#[test]
fn json_format_media_types() {
    let format = JsonFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 1);
    assert_eq!(types[0], mediatype::media_type!(APPLICATION / JSON));
}

#[test]
fn json_format_content_type_header() {
    let format = JsonFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(header, HeaderValue::from_static("application/json"));
}

#[test]
fn json_format_match_specificity_exact() {
    let format = JsonFormat;
    let media_type = mediatype::media_type!(APPLICATION / JSON);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn json_format_match_specificity_wildcard() {
    let format = JsonFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn json_format_match_specificity_type_only() {
    let format = JsonFormat;
    let media_type = MediaType::parse("application/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn json_format_match_specificity_none() {
    let format = JsonFormat;
    let media_type = mediatype::media_type!(TEXT / PLAIN);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn json_format_roundtrip_serialization() {
    let format = JsonFormat;
    let data = TestStruct {
        field: "hello".to_string(),
        number: 42,
    };

    let mut bytes = Vec::new();
    {
        let mut serializer = format.serializer(&mut bytes).unwrap();
        data.serialize(serializer.as_serializer()).unwrap();
    }

    let deserializer = format.deserializer(&bytes).unwrap();
    let result = TestStruct::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(result, data);
}

#[test]
fn json_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
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
fn json_format_is_default() {
    let format = JsonFormat::default();
    assert_eq!(
        Format::content_type_header(&format),
        HeaderValue::from_static("application/json")
    );
}

#[test]
fn json_format_is_debug() {
    let format = JsonFormat;
    let debug_str = format!("{:?}", format);
    assert_eq!(debug_str, "JsonFormat");
}

#[test]
fn json_format_is_clone() {
    let format = JsonFormat;
    let cloned = format;
    assert_eq!(
        Format::content_type_header(&cloned),
        Format::content_type_header(&format)
    );
}
