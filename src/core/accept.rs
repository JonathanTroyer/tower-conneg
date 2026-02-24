//! Accept header parsing.

use std::sync::Arc;

use mediatype::{MediaType, MediaTypeList, ReadParams, names};

use crate::format::{ErasedFormat, Format, MatchSpecificity, match_specificity};

/// Result of Accept header parsing.
#[derive(Debug, Clone, Copy)]
pub struct AcceptMatch<F> {
    /// Matched format.
    pub format: F,
    /// Quality value (0.0-1.0).
    pub quality: f32,
    /// Match specificity level.
    pub specificity: MatchSpecificity,
}

pub(crate) trait FormatMatcher: Clone {
    fn try_match(&self, media_type: &MediaType<'_>) -> Option<MatchSpecificity>;
}

impl<F: Format> FormatMatcher for &F {
    fn try_match(&self, media_type: &MediaType<'_>) -> Option<MatchSpecificity> {
        match_specificity(*self, media_type)
    }
}

impl FormatMatcher for Arc<dyn ErasedFormat> {
    fn try_match(&self, media_type: &MediaType<'_>) -> Option<MatchSpecificity> {
        self.match_specificity(media_type)
    }
}

fn parse_quality_value(s: &str) -> Option<f32> {
    let value: f32 = s.parse().ok()?;
    (0.0..=1.0).contains(&value).then_some(value)
}

fn parse_accept_from_slice<F: FormatMatcher>(
    header: &str,
    formats: &[F],
) -> Option<AcceptMatch<F>> {
    let mut best: Option<AcceptMatch<F>> = None;

    for item in MediaTypeList::new(header) {
        let Ok(media_type) = item else { continue };

        let quality = media_type
            .get_param(names::Q)
            .and_then(|q| parse_quality_value(q.as_str()))
            .unwrap_or(1.0);

        for format in formats {
            if let Some(specificity) = format.try_match(&media_type) {
                let dominated = best
                    .as_ref()
                    .is_some_and(|b| (quality, specificity) <= (b.quality, b.specificity));
                if !dominated {
                    best = Some(AcceptMatch {
                        format: format.clone(),
                        quality,
                        specificity,
                    });
                }
            }
        }
    }

    best
}

/// Parses an Accept header and returns the best matching format.
pub fn parse_accept<'a, F: Format + 'a>(
    header: &str,
    formats: &[&'a F],
) -> Option<AcceptMatch<&'a F>> {
    parse_accept_from_slice(header, formats)
}

pub(crate) fn parse_accept_erased(
    header: &str,
    formats: &[Arc<dyn ErasedFormat>],
) -> Option<AcceptMatch<Arc<dyn ErasedFormat>>> {
    parse_accept_from_slice(header, formats)
}
