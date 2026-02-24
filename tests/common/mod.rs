#![allow(elided_lifetimes_in_paths)]
#![allow(unused_imports)]

#[cfg(feature = "json")]
pub(crate) use tower_conneg::JsonFormat;
#[cfg(feature = "xml")]
pub(crate) use tower_conneg::XmlFormat;
