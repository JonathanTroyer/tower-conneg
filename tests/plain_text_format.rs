//! Tests for the built-in PlainTextFormat.

#![cfg(feature = "plain")]

use std::sync::Arc;

use http::HeaderValue;
use mediatype::MediaType;
use serde::{Deserialize, Serialize};
use tower_conneg::{
    ErasedFormat, Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer, PlainTextFormat,
    match_specificity,
};

#[test]
fn plain_text_format_media_types() {
    let format = PlainTextFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 1);
    assert_eq!(types[0], mediatype::media_type!(TEXT / PLAIN));
}

#[test]
fn plain_text_format_content_type_header() {
    let format = PlainTextFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(header, HeaderValue::from_static("text/plain; charset=utf-8"));
}

#[test]
fn plain_text_format_match_specificity_exact() {
    let format = PlainTextFormat;
    let media_type = mediatype::media_type!(TEXT / PLAIN);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn plain_text_format_match_specificity_wildcard() {
    let format = PlainTextFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn plain_text_format_match_specificity_type_only() {
    let format = PlainTextFormat;
    let media_type = MediaType::parse("text/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn plain_text_format_match_specificity_none() {
    let format = PlainTextFormat;
    let media_type = mediatype::media_type!(APPLICATION / JSON);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn plain_text_format_serialize_string() {
    let format = PlainTextFormat;
    let data = "Hello, world!".to_string();

    let mut bytes = Vec::new();
    {
        let mut serializer = format.serializer(&mut bytes).unwrap();
        data.serialize(serializer.as_serializer()).unwrap();
    }

    assert_eq!(bytes, b"Hello, world!");
}

#[test]
fn plain_text_format_deserialize_string() {
    let format = PlainTextFormat;
    let bytes = b"Hello, world!";

    let mut deserializer = format.deserializer(bytes).unwrap();
    let result = String::deserialize(deserializer.as_deserializer()).unwrap();

    assert_eq!(result, "Hello, world!");
}

#[test]
fn plain_text_format_roundtrip() {
    let format = PlainTextFormat;
    let data = "Test string with special chars: <>&\"'".to_string();

    let mut bytes = Vec::new();
    {
        let mut serializer = format.serializer(&mut bytes).unwrap();
        data.serialize(serializer.as_serializer()).unwrap();
    }

    let mut deserializer = format.deserializer(&bytes).unwrap();
    let result = String::deserialize(deserializer.as_deserializer()).unwrap();

    assert_eq!(result, data);
}

#[test]
fn plain_text_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(PlainTextFormat);
    let data = "test string".to_string();

    let mut bytes = Vec::new();
    format
        .serialize(&mut bytes, &mut |serializer| {
            use erased_serde::Serialize;
            data.erased_serialize(serializer)
        })
        .unwrap();

    let mut result: Option<String> = None;
    format
        .deserialize(&bytes, &mut |deserializer| {
            result = Some(erased_serde::deserialize(deserializer)?);
            Ok(())
        })
        .unwrap();

    assert_eq!(result, Some(data));
}

#[test]
fn plain_text_format_serialize_integer_fails() {
    let format = PlainTextFormat;

    let mut bytes = Vec::new();
    let mut serializer = format.serializer(&mut bytes).unwrap();
    let result = 42i32.serialize(serializer.as_serializer());

    assert!(result.is_err());
}

#[test]
fn plain_text_format_deserialize_integer_fails() {
    let format = PlainTextFormat;
    let bytes = b"42";

    let mut deserializer = format.deserializer(bytes).unwrap();
    let result = i32::deserialize(deserializer.as_deserializer());

    assert!(result.is_err());
}

#[test]
fn plain_text_format_is_default() {
    let format = PlainTextFormat::default();
    assert_eq!(
        Format::content_type_header(&format),
        HeaderValue::from_static("text/plain; charset=utf-8")
    );
}

#[test]
fn plain_text_format_is_debug() {
    let format = PlainTextFormat;
    let debug_str = format!("{:?}", format);
    assert_eq!(debug_str, "PlainTextFormat");
}

#[test]
fn plain_text_format_is_clone() {
    let format = PlainTextFormat;
    let cloned = format;
    assert_eq!(
        Format::content_type_header(&cloned),
        Format::content_type_header(&format)
    );
}
