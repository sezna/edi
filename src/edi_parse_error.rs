use std::{error, fmt};

/// Represents an error that occured at any point in parsing a document.
#[derive(Debug, Clone)]
pub struct EdiParseError {
    reason: String,
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
    pub fn new(reason: &str) -> EdiParseError {
        EdiParseError {
            reason: String::from(reason),
        }
    }
}

/// returns an EDI error with a custom error message if the given condition is false.
macro_rules! edi_assert {
    ($condition:expr, $reason:expr) => {{
        if !$condition {
            return Err(EdiParseError::new($reason));
        }
    }};
    ($condition:expr, $reason:expr, $additional_info:expr) => {{
        if !$condition {
            return Err(EdiParseError::new(
                format!("{} {:?}", $reason, $additional_info).as_str(),
            ));
        }
    }};
}
