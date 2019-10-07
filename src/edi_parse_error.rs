use crate::tokenizer::SegmentTokens;
use std::{error, fmt};

/// Represents an error that occured at any point in parsing a document.
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
