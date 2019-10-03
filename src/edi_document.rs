use crate::edi_parse_error::EdiParseError;

/// Represents an entire parsed EDI document with both the envelope (i.e. metadata) and
/// the data segments.
pub struct EdiDocument;

/// This is the main entry point to the crate. Parse an input str and output either
/// an [EdiParseError] or a resulting [EdiDocument].
pub fn parse(input: &str) -> Result<EdiDocument, EdiParseError> {
    edi_assert!(
        input.len() > 106,
        "Input document is not long enough to be an EDI document"
    );
    // I found documentation that the ISA line is standardized such that the
    // 103rd to 106th bytes are the delimiters.
    let delimiters: Vec<char> = input[103..106].chars().collect();
    let (element_delimiter, subelement_delimiter, segment_delimiter) =
        (delimiters[0], delimiters[1], delimiters[2]);

    return Ok(EdiDocument {});
}
