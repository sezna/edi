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
        for (idx, interchange) in self.interchanges.iter().enumerate() {
            if idx > 0 {
                buffer.push(self.segment_delimiter);
            }
            buffer.push_str(&interchange.to_x12_string(
                self.segment_delimiter,
                self.element_delimiter,
                self.sub_element_delimiter,
            ));
        }

        buffer
    }
}

/// This is the main entry point to the crate. Parse an input str and output either
/// an [EdiParseError] or a resulting [EdiDocument].
pub fn parse(input: &str) -> Result<EdiDocument<'_>, EdiParseError> {
    parse_inner(input, false)
}

/// This is an alternate parser which does not perform closing tag validation. If you are receiving
/// EDI documents which have had less rigor applied to their construction, this may help. The number
/// of documents in the confirmation and the IDs on the closing tags don't need to match.
pub fn loose_parse(input: &str) -> Result<EdiDocument<'_>, EdiParseError> {
    parse_inner(input, true)
}

/// An internal function which is the root of the parsing. It is accessed publicly via [parse] and [loose_parse].
fn parse_inner(input: &str, loose: bool) -> Result<EdiDocument<'_>, EdiParseError> {
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

    Ok(EdiDocument {
        interchanges,
        element_delimiter: tokenize_result.element_delimiter,
        sub_element_delimiter: tokenize_result.sub_element_delimiter,
        segment_delimiter: tokenize_result.segment_delimiter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edi_document_to_x12_string() {
        let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
SE*3*0001~
GE*1*1~
IEA*1*000000001~";
        
        let doc = parse(input).unwrap();
        let output = doc.to_x12_string();
        
        // The output should contain all the essential segments
        assert!(output.contains("ISA"));
        assert!(output.contains("GS"));
        assert!(output.contains("ST"));
        assert!(output.contains("BEG"));
        assert!(output.contains("SE"));
        assert!(output.contains("GE"));
        assert!(output.contains("IEA"));
    }

    #[test]
    fn test_round_trip_parsing() {
        let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
REF*DP*123456~
SE*4*0001~
GE*1*1~
IEA*1*000000001~";
        
        let doc1 = parse(input).unwrap();
        let output = doc1.to_x12_string();
        let doc2 = parse(&output).unwrap();
        
        // Verify key properties are preserved
        assert_eq!(doc1.interchanges.len(), doc2.interchanges.len());
        assert_eq!(doc1.element_delimiter, doc2.element_delimiter);
        assert_eq!(doc1.segment_delimiter, doc2.segment_delimiter);
        assert_eq!(doc1.sub_element_delimiter, doc2.sub_element_delimiter);
        
        // Verify the document structure is preserved
        assert_eq!(
            doc1.interchanges[0].functional_groups.len(),
            doc2.interchanges[0].functional_groups.len()
        );
        assert_eq!(
            doc1.interchanges[0].functional_groups[0].transactions.len(),
            doc2.interchanges[0].functional_groups[0].transactions.len()
        );
    }

    #[test]
    fn test_multiple_interchanges() {
        let input = "ISA*00*          *00*          *ZZ*SENDER1        *ZZ*RECEIVER1      *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER1*RECEIVER1*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
SE*3*0001~
GE*1*1~
IEA*1*000000001~ISA*00*          *00*          *ZZ*SENDER2        *ZZ*RECEIVER2      *200301*1254*U*00401*000000002*0*T*>~
GS*PO*SENDER2*RECEIVER2*20200301*1254*2*X*004010~
ST*850*0002~
BEG*00*SA*PO456**20200301~
SE*3*0002~
GE*1*2~
IEA*1*000000002~";

        let doc = parse(input).unwrap();
        
        // Should have two interchanges
        assert_eq!(doc.interchanges.len(), 2);
        assert_eq!(doc.interchanges[0].sender_id, "SENDER1");
        assert_eq!(doc.interchanges[1].sender_id, "SENDER2");
        assert_eq!(doc.interchanges[0].receiver_id, "RECEIVER1");
        assert_eq!(doc.interchanges[1].receiver_id, "RECEIVER2");
    }

    #[test]
    fn test_custom_delimiters() {
        // Test with different element delimiter (: instead of *)
        let input = "ISA:00:          :00:          :ZZ:SENDER         :ZZ:RECEIVER       :200301:1253:U:00401:000000001:0:T:>~
GS:PO:SENDER:RECEIVER:20200301:1253:1:X:004010~
ST:850:0001~
BEG:00:SA:PO123::20200301~
SE:3:0001~
GE:1:1~
IEA:1:000000001~";
        
        let doc = parse(input).unwrap();
        assert_eq!(doc.element_delimiter, ':');
        assert_eq!(doc.segment_delimiter, '~');
        
        // Verify the document was parsed correctly
        assert_eq!(doc.interchanges.len(), 1);
        assert_eq!(doc.interchanges[0].sender_id, "SENDER");
        assert_eq!(doc.interchanges[0].receiver_id, "RECEIVER");
    }

    #[test]
    fn test_loose_parse_with_mismatched_counts() {
        // SE has wrong segment count (says 5 but only has 3)
        let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
SE*5*0001~
GE*1*1~
IEA*1*000000001~";
        
        // Should fail with strict parse
        assert!(parse(input).is_err());
        
        // Should succeed with loose parse
        let doc = loose_parse(input).unwrap();
        assert_eq!(doc.interchanges.len(), 1);
    }

    #[test]
    fn test_document_with_sub_elements() {
        let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*:~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123:SUB1:SUB2**20200301~
SE*3*0001~
GE*1*1~
IEA*1*000000001~";
        
        let doc = parse(input).unwrap();
        assert_eq!(doc.sub_element_delimiter, ':');
        
        // Verify sub-elements are preserved
        let beg_segment = &doc.interchanges[0].functional_groups[0].transactions[0].segments[0];
        assert_eq!(beg_segment.segment_abbreviation, "BEG");
    }
}
