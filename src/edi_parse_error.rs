use crate::tokenizer::SegmentTokens;
use std::{error, fmt};
/// Represents an error that occurred at any point in parsing a document.
/// Contains a reason the error occurred and the segment in which the error occurred.
#[derive(Debug, Clone)]
pub struct EdiParseError {
    /// The reason for the error.
    reason: String,
    /// The segment in which the error occurred.
    #[allow(dead_code)]
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
    /// Construct a new [EdiParseError].
    pub fn new(reason: &str, error_segment: Option<SegmentTokens>) -> EdiParseError {
        let error_segment = error_segment
            .map(|error_segment| error_segment.iter().map(|x| x.to_string()).collect());
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
    if let Some(segment) = maybe_segment {
        Ok(segment)
    } else {
        Err(EdiParseError{
            reason: "EDI file out of order: from out to in, the file must have ISA, GS, ST, and then generic segments".to_string(),
            error_segment: Some(error_segment.iter().map(|x| x.to_string()).collect())
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = EdiParseError::new("Test error reason", None);
        let display_string = format!("{}", error);
        assert!(display_string.contains("Error parsing input into EDI document"));
        assert!(display_string.contains("Test error reason"));
    }

    #[test]
    fn test_error_construction_with_segment() {
        let segment = vec!["ISA", "00", "test"];
        let error = EdiParseError::new("Invalid segment", Some(segment.clone()));
        assert!(format!("{}", error).contains("Invalid segment"));
    }

    #[test]
    fn test_error_construction_without_segment() {
        let error = EdiParseError::new("Missing required field", None);
        assert!(format!("{}", error).contains("Missing required field"));
    }

    #[test]
    fn test_try_option_with_some() {
        let segment = vec!["TEST"];
        let result = try_option(Some(42), &segment);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_try_option_with_none() {
        let segment = vec!["TEST"];
        let result: Result<i32, EdiParseError> = try_option(None, &segment);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(format!("{}", error).contains("EDI file out of order"));
    }

    #[test]
    fn test_edi_assert_macro_simple() {
        fn test_assert_simple() -> Result<(), EdiParseError> {
            edi_assert!(true, "This should not fail");
            edi_assert!(1 + 1 == 2, "Math still works");
            Ok(())
        }
        assert!(test_assert_simple().is_ok());
    }

    #[test]
    fn test_edi_assert_macro_failure() {
        fn test_assert_failure() -> Result<(), EdiParseError> {
            edi_assert!(false, "This should fail");
            Ok(())
        }
        let result = test_assert_failure();
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("This should fail"));
    }

    #[test]
    fn test_edi_assert_with_expected_result() {
        fn test_assert_with_values() -> Result<(), EdiParseError> {
            let expected = "850";
            let result = "810";
            edi_assert!(
                expected == result,
                "Transaction code mismatch",
                expected,
                result
            );
            Ok(())
        }
        let result = test_assert_with_values();
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Transaction code mismatch"));
        assert!(error_msg.contains("expected: 850"));
        assert!(error_msg.contains("received: 810"));
    }
}
