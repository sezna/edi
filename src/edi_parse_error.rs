use crate::tokenizer::SegmentTokens;
use std::{error, fmt};
/// Represents an error that occurred at any point in parsing a document.
/// Contains a reason the error occurred and the segment in which the error occurred.
#[derive(Debug, Clone)]
pub struct EdiParseError {
    /// The reason for the error.
    reason: String,
    /// The segment in which the error occurred.
    error_segment: Option<Vec<String>>,
}

impl fmt::Display for EdiParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing input into EDI document {}", self.reason)
    }
}

impl error::Error for EdiParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl EdiParseError {
    #[doc(skip)]
    /// Construct a new [EdiParseError].
    pub fn new(reason: &str, error_segment: Option<SegmentTokens>) -> EdiParseError {
        let error_segment = if let Some(error_segment) = error_segment {
            Some(error_segment.iter().map(|x| x.to_string()).collect())
        } else {
            None
        };
        EdiParseError {
            reason: String::from(reason),
            error_segment,
        }
    }
}

/// Since implementing `From<NoneError>` is unstable right now, this is a temporary way to emulate
/// coercing the `?` (try trait)'s behavior on an `Option` into an [EdiParseError]
pub fn try_option<T>(
    maybe_segment: Option<T>,
    error_segment: &SegmentTokens,
) -> Result<T, EdiParseError> {
    if maybe_segment.is_some() {
        return Ok(maybe_segment.unwrap());
    } else {
        return Err(EdiParseError{
            reason: "EDI file out of order: from out to in, the file must have ISA, GS, ST, and then generic segments".to_string(),
            error_segment: Some(error_segment.iter().map(|x| x.to_string()).collect())
        });
    }
}

/// returns an EDI error with a custom error message if the given condition is false.
/// Supports three use cases:
///    `(condition, reason)` - if not condition, display reason
///    `(condition, reason, error_segment)` - if not condition, display reason with the segment it occurred in
///    `(condition, reason, expected, result)` - if not condition, display reason with what was expected and what occurred.
///                                              similar to `assert_eq!`.
///    `(condition, reason, expected, result, error_segment)` - if not condition, display reason with what was expected and what occurred,
///                                                             and the segment the error occurred in.
///                                                             similar to `assert_eq!`.
// perhaps someday this can become edi_assert_eq, edi_assert_neq, and edi_assert
macro_rules! edi_assert {
    ($condition:expr, $reason:expr) => {{
        if !$condition {
            return Err(EdiParseError::new($reason, None));
        }
    }};
    ($condition:expr, $reason:expr, $error_segment:expr) => {{
        if !$condition {
            return Err(EdiParseError::new($reason, Some($error_segment)));
        }
    }};
    ($condition:expr, $reason:expr, $expected:expr, $result:expr) => {{
        if !$condition {
            return Err(EdiParseError::new(
                format!(
                    "{}  --  expected: {}  received: {}",
                    $reason, $expected, $result
                )
                .as_str(),
                None,
            ));
        }
    }};
    ($condition:expr, $reason:expr, $expected:expr, $result:expr, $error_segment:expr) => {{
        if !$condition {
            return Err(EdiParseError::new(
                format!(
                    "{}  --  expected: {}  received: {}",
                    $reason, $expected, $result
                )
                .as_str(),
                Some($error_segment),
            ));
        }
    }};
}
