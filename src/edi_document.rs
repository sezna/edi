use crate::edi_parse_error::EdiParseError;
use crate::interchange_control::InterchangeControl;
use crate::tokenizer::tokenize;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Represents an entire parsed EDI document with both the envelope (i.e. metadata) and
/// the data segments.
#[derive(Serialize, Deserialize)]
pub struct EdiDocument<'a, 'b> {
    // Here I chose a VecDeque because when I output an EDI document, I want to pull from
    // it in a queue style.
    /// Represents the interchanges (ISA/IEA) held within this document.
    #[serde(borrow = "'a + 'b")]
    pub interchanges: VecDeque<InterchangeControl<'a, 'b>>,
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
    let document_tokens = tokenize(input).expect("unsupported EDI format");

    // Go through all the segments and parse them either into an interchange control header,
    // functional group header, transaction header, or generic segment. Also verify that
    // the nesting order is correct.
    let mut interchanges: VecDeque<InterchangeControl> = VecDeque::new();

    for segment in document_tokens {
        match segment[0] {
            "ISA" => {
                interchanges.push_back(
                    InterchangeControl::parse_from_tokens(segment)
                        .expect("failed to parse interchange header"),
                );
            }
            "GS" => {
                interchanges.back_mut().expect("unable to enqueue functional group when no interchanges have been enqueued").add_functional_group(segment);
            }
            "ST" => {
                interchanges
                    .back_mut()
                    .expect("unable to enqueue transaction when no interchanges have been enqueued")
                    .add_transaction(segment);
            }
            "IEA" => {
                if !loose {
                    interchanges
                        .back()
                        .expect("unable to validate IEA without initial ISA")
                        .validate_interchange_control(segment)
                        .expect("interchange control validation failed");
                }
            }
            "GE" => {
                if !loose {
                    interchanges
                        .back()
                        .expect("unable to validate GE without interchange")
                        .validate_functional_group(segment)
                        .expect("functional group validation failed");
                }
            }
            "SE" => {
                if !loose {
                    interchanges
                        .back()
                        .expect("unable to validate SE without interchange")
                        .validate_transaction(segment)
                        .expect("transaction validation failed")
                }
            }
            _ => {
                interchanges
                    .back_mut()
                    .expect(
                        "unable to enqueue generic segment when no interchanges have been enqueued",
                    )
                    .add_generic_segment(segment);
            }
        }
    }

    return Ok(EdiDocument { interchanges });
}
