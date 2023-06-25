use crate::edi_parse_error::{try_option, EdiParseError};
use crate::interchange_control::InterchangeControl;
use crate::tokenizer::tokenize;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Represents an entire parsed EDI document with both the envelope (i.e. metadata) and
/// the data segments.
#[derive(Serialize, Deserialize)]
pub struct EdiDocument<'a> {
    // Here I chose a VecDeque because when I output an EDI document, I want to pull from
    // it in a queue style.
    /// Represents the interchanges (ISA/IEA) held within this document.
    #[serde(borrow = "'a")]
    pub interchanges: VecDeque<InterchangeControl<'a>>,
    /// Represents the separator between segments in the EDI document.
    pub segment_delimiter: char,
    /// Represents the separator between sub elements in the EDI document.
    pub sub_element_delimiter: char,
    /// Represents the separator between elements in the EDI document.
    pub element_delimiter: char,
}

impl EdiDocument<'_> {
    /// Turns this [EdiDocument] into an ANSI x12 string.
    pub fn to_x12_string(&self) -> String {
        let mut buffer = String::new();
        let mut idx = 0;
        for interchange in self.interchanges.iter() {
            if idx > 0 {
                buffer.push(self.segment_delimiter);
            }
            buffer.push_str(&interchange.to_x12_string(
                self.segment_delimiter,
                self.element_delimiter,
                self.sub_element_delimiter,
            ));
            idx += 1;
        }

        buffer
    }
}

/// This is the main entry point to the crate. Parse an input str and output either
/// an [EdiParseError] or a resulting [EdiDocument].
pub fn parse(input: &str) -> Result<EdiDocument, EdiParseError> {
    parse_inner(input, false)
}

/// This is an alternate parser which does not perform closing tag validation. If you are receiving
/// EDI documents which have had less rigor applied to their construction, this may help. The number
/// of documents in the confirmation and the IDs on the closing tags don't need to match.
pub fn loose_parse(input: &str) -> Result<EdiDocument, EdiParseError> {
    parse_inner(input, true)
}

/// An internal function which is the root of the parsing. It is accessed publicly via [parse] and [loose_parse].
fn parse_inner(input: &str, loose: bool) -> Result<EdiDocument, EdiParseError> {
    let tokenize_result = tokenize(input)?;
    let document_tokens = tokenize_result.tokens;

    // Go through all the segments and parse them either into an interchange control header,
    // functional group header, transaction header, or generic segment. Also verify that
    // the nesting order is correct.
    let mut interchanges: VecDeque<InterchangeControl> = VecDeque::new();

    for segment in document_tokens {
        match segment[0] {
            "ISA" => {
                interchanges.push_back(InterchangeControl::parse_from_tokens(segment)?);
            }
            "GS" => {
                try_option(interchanges.back_mut(), &segment)?.add_functional_group(segment)?;
            }
            "ST" => {
                try_option(interchanges.back_mut(), &segment)?.add_transaction(segment)?;
            }
            "IEA" => {
                if !loose {
                    try_option(interchanges.back(), &segment)?
                        .validate_interchange_control(segment)?;
                };
            }
            "GE" => {
                if !loose {
                    try_option(interchanges.back(), &segment)?
                        .validate_functional_group(segment)?;
                };
            }
            "SE" => {
                if !loose {
                    try_option(interchanges.back(), &segment)?.validate_transaction(segment)?;
                };
            }
            _ => {
                try_option(interchanges.back_mut(), &segment)?.add_generic_segment(segment)?;
            }
        }
    }

    return Ok(EdiDocument {
        interchanges,
        element_delimiter: tokenize_result.element_delimiter,
        sub_element_delimiter: tokenize_result.sub_element_delimiter,
        segment_delimiter: tokenize_result.segment_delimiter,
    });
}
