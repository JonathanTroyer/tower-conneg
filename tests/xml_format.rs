//! Tests for the built-in XmlFormat.

#![cfg(feature = "xml")]

use std::sync::Arc;

use erased_serde::Serialize as _;
use http::HeaderValue;
use mediatype::MediaType;
use serde::{Deserialize, Serialize};
use tower_conneg::{
    ErasedFormat, Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer, XmlFormat,
    match_specificity,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    field: String,
    number: i32,
}

#[test]
fn xml_format_media_types() {
    let format = XmlFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 2);
    assert_eq!(types[0], mediatype::media_type!(APPLICATION / XML));
    assert_eq!(types[1], mediatype::media_type!(TEXT / XML));
}

#[test]
fn xml_format_content_type_header() {
    let format = XmlFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(header, HeaderValue::from_static("application/xml"));
}

#[test]
fn xml_format_match_specificity_exact_application() {
    let format = XmlFormat;
    let media_type = mediatype::media_type!(APPLICATION / XML);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn xml_format_match_specificity_exact_text() {
    let format = XmlFormat;
    let media_type = mediatype::media_type!(TEXT / XML);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn xml_format_match_specificity_wildcard() {
    let format = XmlFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn xml_format_match_specificity_type_only_application() {
    let format = XmlFormat;
    let media_type = MediaType::parse("application/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn xml_format_match_specificity_type_only_text() {
    let format = XmlFormat;
    let media_type = MediaType::parse("text/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn xml_format_match_specificity_none() {
    let format = XmlFormat;
    let media_type = mediatype::media_type!(IMAGE / PNG);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn xml_format_roundtrip_serialization() {
    let format = XmlFormat;
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
fn xml_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(XmlFormat);
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
fn xml_format_utf8_handling() {
    let format = XmlFormat;
    let data = TestStruct {
        field: "hello world".to_string(),
        number: 42,
    };

    let mut bytes = Vec::new();
    format
        .serializer(&mut bytes)
        .unwrap()
        .with_erased(&mut |ser| data.erased_serialize(ser))
        .unwrap();

    let output = String::from_utf8(bytes.clone()).unwrap();
    assert!(
        output.contains("hello world"),
        "XML output should be valid UTF-8: {}",
        output
    );
}

#[test]
fn xml_format_invalid_utf8_fails() {
    let format = XmlFormat;
    let invalid_utf8: &[u8] = &[0xFF, 0xFE];

    let result = format.deserializer(invalid_utf8);
    assert!(result.is_err(), "Invalid UTF-8 should cause an error");
}

#[test]
fn xml_format_is_default() {
    let format = XmlFormat::default();
    assert_eq!(
        Format::content_type_header(&format),
        HeaderValue::from_static("application/xml")
    );
}

#[test]
fn xml_format_is_debug() {
    let format = XmlFormat;
    let debug_str = format!("{:?}", format);
    assert_eq!(debug_str, "XmlFormat");
}

#[test]
fn xml_format_is_clone() {
    let format = XmlFormat;
    let cloned = format;
    assert_eq!(
        Format::content_type_header(&cloned),
        Format::content_type_header(&format)
    );
}
