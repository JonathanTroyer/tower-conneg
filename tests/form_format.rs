//! Tests for the built-in FormFormat.

#![cfg(feature = "form")]

use std::sync::Arc;

use erased_serde::Serialize as _;
use http::HeaderValue;
use mediatype::MediaType;
use serde::{Deserialize, Serialize};
use tower_conneg::{
    ErasedFormat, FormFormat, Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer,
    match_specificity,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    field: String,
    number: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember: bool,
}

#[test]
fn form_format_media_types() {
    let format = FormFormat;
    let types = format.media_types();

    assert_eq!(types.len(), 1);
    assert_eq!(
        types[0],
        MediaType::new(
            mediatype::names::APPLICATION,
            mediatype::Name::new_unchecked("x-www-form-urlencoded")
        )
    );
}

#[test]
fn form_format_content_type_header() {
    let format = FormFormat;
    let header = Format::content_type_header(&format);

    assert_eq!(
        header,
        HeaderValue::from_static("application/x-www-form-urlencoded")
    );
}

#[test]
fn form_format_match_specificity_exact() {
    let format = FormFormat;
    let media_type = MediaType::new(
        mediatype::names::APPLICATION,
        mediatype::Name::new_unchecked("x-www-form-urlencoded"),
    );

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Exact));
}

#[test]
fn form_format_match_specificity_wildcard() {
    let format = FormFormat;
    let media_type = MediaType::parse("*/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::Wildcard));
}

#[test]
fn form_format_match_specificity_type_only() {
    let format = FormFormat;
    let media_type = MediaType::parse("application/*").unwrap();

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, Some(MatchSpecificity::TypeOnly));
}

#[test]
fn form_format_match_specificity_none() {
    let format = FormFormat;
    let media_type = mediatype::media_type!(TEXT / PLAIN);

    let result = match_specificity(&format, &media_type);
    assert_eq!(result, None);
}

#[test]
fn form_format_serialize_struct() {
    let format = FormFormat;
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

    assert_eq!(String::from_utf8(bytes).unwrap(), "field=hello&number=42");
}

#[test]
fn form_format_deserialize_struct() {
    let format = FormFormat;
    let bytes = b"field=hello&number=42";

    let deserializer = format.deserializer(bytes).unwrap();
    let result = TestStruct::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(
        result,
        TestStruct {
            field: "hello".to_string(),
            number: 42
        }
    );
}

#[test]
fn form_format_roundtrip_struct() {
    let format = FormFormat;
    let data = TestStruct {
        field: "test".to_string(),
        number: 123,
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
fn form_format_url_encoding() {
    let format = FormFormat;
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

    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("hello+world") || output.contains("hello%20world"));
}

#[test]
fn form_format_deserialize_url_encoded() {
    let format = FormFormat;
    let bytes = b"field=hello%20world&number=42";

    let deserializer = format.deserializer(bytes).unwrap();
    let result = TestStruct::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(
        result,
        TestStruct {
            field: "hello world".to_string(),
            number: 42
        }
    );
}

#[test]
fn form_format_login_form() {
    let format = FormFormat;
    let data = LoginForm {
        username: "user@example.com".to_string(),
        password: "secret123".to_string(),
        remember: true,
    };

    let mut bytes = Vec::new();
    format
        .serializer(&mut bytes)
        .unwrap()
        .with_erased(&mut |ser| data.erased_serialize(ser))
        .unwrap();

    let deserializer = format.deserializer(&bytes).unwrap();
    let result = LoginForm::deserialize(deserializer.into_deserializer()).unwrap();

    assert_eq!(result, data);
}

#[test]
fn form_format_erased_roundtrip() {
    let format: Arc<dyn ErasedFormat> = Arc::new(FormFormat);
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
