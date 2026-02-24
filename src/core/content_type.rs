//! Content-Type header parsing.

use std::sync::Arc;

use mediatype::MediaType;

use super::accept::FormatMatcher;
use crate::format::{ErasedFormat, Format, MatchSpecificity};

fn parse_content_type_from_slice<F: FormatMatcher>(header: &str, formats: &[F]) -> Option<F> {
    let media_type = MediaType::parse(header).ok()?;

    formats.iter().find_map(|format| {
        matches!(format.try_match(&media_type), Some(MatchSpecificity::Exact))
            .then(|| format.clone())
    })
}

/// Parses a Content-Type header and returns the matching format.
pub fn parse_content_type<'a, F: Format + 'a>(header: &str, formats: &[&'a F]) -> Option<&'a F> {
    parse_content_type_from_slice(header, formats)
}

/// Parses a Content-Type header and returns the matching erased format.
pub fn parse_content_type_erased(
    header: &str,
    formats: &[Arc<dyn ErasedFormat>],
) -> Option<Arc<dyn ErasedFormat>> {
    parse_content_type_from_slice(header, formats)
}
