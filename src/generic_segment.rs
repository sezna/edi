use crate::edi_parse_error::EdiParseError;
use crate::tokenizer::SegmentTokens;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::VecDeque;

/// A generic segment.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct GenericSegment<'a> {
    #[serde(borrow)]
    pub segment_abbreviation: Cow<'a, str>,
    pub segment_name: String,
    #[serde(borrow)]
    pub elements: VecDeque<Cow<'a, str>>,
}

impl<'a> GenericSegment<'a> {
    pub fn parse_from_tokens(tokens: SegmentTokens<'a>) -> Result<GenericSegment, EdiParseError> {
        let elements: Vec<&str> = tokens.iter().map(|x| x.trim()).collect();
        edi_assert!(
            elements.len() >= 2,
            "at least two elements are required in a segment"
        );
        let segment_abbreviation = Cow::from(elements[0]);

        let segment_name: String = "unidentified".to_string(); // TODO

        let elements = elements[1..]
            .to_vec()
            .iter()
            .map(|x| Cow::from(*x))
            .collect::<VecDeque<Cow<str>>>();

        Ok(GenericSegment {
            segment_abbreviation,
            segment_name,
            elements,
        })
    }
}

#[test]
fn construct_generic_segment() {
    let test_input = vec![
        "GS",
        "PO",
        "SENDERGS",
        "007326879",
        "20020226",
        "1534",
        "1",
        "X",
        "004010",
    ];

    let expected_result = GenericSegment {
        segment_abbreviation: Cow::from("GS"),
        segment_name: String::from("unidentified"),
        elements: vec![
            "PO",
            "SENDERGS",
            "007326879",
            "20020226",
            "1534",
            "1",
            "X",
            "004010",
        ]
        .iter()
        .map(|x| Cow::from(*x))
        .collect::<VecDeque<Cow<str>>>(),
    };

    assert_eq!(
        GenericSegment::parse_from_tokens(test_input).unwrap(),
        expected_result
    );
}
