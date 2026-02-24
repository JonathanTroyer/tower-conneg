//! Content-Type header parsing.

use std::sync::Arc;

use mediatype::MediaType;

use crate::accept::FormatMatcher;
use crate::format::{ErasedFormat, Format, MatchSpecificity};

fn parse_content_type_from_slice<F: FormatMatcher>(header: &str, formats: &[F]) -> Option<F> {
    let media_type = MediaType::parse(header).ok()?;

    formats.iter().find_map(|format| {
        matches!(format.try_match(&media_type), Some(MatchSpecificity::Exact))
            .then(|| format.clone())
    })
}

/// Parse a Content-Type header and find the matching format.
///
/// Unlike Accept parsing, Content-Type headers contain a single media type
/// without quality values or wildcards - we simply match against the
/// configured formats.
pub fn parse_content_type<'a, F: Format + 'a>(header: &str, formats: &[&'a F]) -> Option<&'a F> {
    parse_content_type_from_slice(header, formats)
}

/// Parse a Content-Type header and find the matching format from erased formats.
///
/// This version works with `Arc<dyn ErasedFormat>` for dynamic dispatch.
pub fn parse_content_type_erased(
    header: &str,
    formats: &[Arc<dyn ErasedFormat>],
) -> Option<Arc<dyn ErasedFormat>> {
    parse_content_type_from_slice(header, formats)
}
