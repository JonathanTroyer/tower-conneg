//! Tests for the built-in HtmlFormat.

#![cfg(feature = "plain")]

use std::sync::Arc;

use erased_serde::Serialize as _;
use http::HeaderValue;
use mediatype::MediaType;
use serde::Deserialize;
use tower_conneg::{
    ErasedFormat, Format, HtmlFormat, MatchSpecificity, OwnedDeserializer, OwnedSerializer,
    match_specificity,
};

#[test]
fn html_format_media_types() {
    let format = HtmlFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 1);
    assert_eq!(types[0], mediatype::media_type!(TEXT / HTML));
}

#[test]
fn html_format_content_type_header() {
    let format = HtmlFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(header, HeaderValue::from_static("text/html; charset=utf-8"));
}

#[test]
fn html_format_match_specificity_exact() {
    let format = HtmlFormat;
    let media_type = mediatype::media_type!(TEXT / HTML);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn html_format_match_specificity_wildcard() {
    let format = HtmlFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn html_format_match_specificity_type_only() {
    let format = HtmlFormat;
    let media_type = MediaType::parse("text/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn html_format_match_specificity_none() {
    let format = HtmlFormat;
    let media_type = mediatype::media_type!(APPLICATION / JSON);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn html_format_serialize_string() {
    let format = HtmlFormat;
    let data = "<html><body>Hello</body></html>".to_string();

    let mut bytes = Vec::new();
    format
        .serializer(&mut bytes)
        .unwrap()
        .with_erased(&mut |ser| data.erased_serialize(ser))
        .unwrap();

    assert_eq!(bytes, b"<html><body>Hello</body></html>");
}

#[test]
fn html_format_deserialize_string() {
    let format = HtmlFormat;
    let bytes = b"<html><body>Hello</body></html>";

    let deserializer = format.deserializer(bytes).unwrap();
    let result = String::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(result, "<html><body>Hello</body></html>");
}

#[test]
fn html_format_roundtrip() {
    let format = HtmlFormat;
    let data = "<html><body>&amp; test</body></html>".to_string();

    let mut bytes = Vec::new();
    format
        .serializer(&mut bytes)
        .unwrap()
        .with_erased(&mut |ser| data.erased_serialize(ser))
        .unwrap();

    let deserializer = format.deserializer(&bytes).unwrap();
    let result = String::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(result, data);
}

#[test]
fn html_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(HtmlFormat);
    let data = "<p>test</p>".to_string();

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
