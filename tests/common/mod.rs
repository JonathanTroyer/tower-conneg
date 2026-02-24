#![allow(dead_code)]

use std::convert::Infallible;

use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
pub use tower_conneg::JsonFormat;
#[cfg(feature = "xml")]
pub use tower_conneg::XmlFormat;

/// Shared test data structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestStruct {
    pub field: String,
    pub number: i32,
}

/// Generic test error for service implementations.
#[derive(Debug)]
pub struct TestError(pub String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl From<Infallible> for TestError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl<T> From<std::sync::PoisonError<T>> for TestError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        TestError(e.to_string())
    }
}

#[cfg(all(feature = "json", feature = "xml"))]
impl From<tower_conneg::RetryError<Infallible>> for TestError {
    fn from(e: tower_conneg::RetryError<Infallible>) -> Self {
        TestError(e.to_string())
    }
}

pub type TestResult = Result<(), TestError>;
