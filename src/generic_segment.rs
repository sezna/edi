use crate::edi_parse_error::EdiParseError;
use crate::tokenizer::SegmentTokens;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::VecDeque;

/// A generic segment.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct GenericSegment<'a> {
    /// The first element in the segment which denotes the segment type.
    #[serde(borrow)]
    pub segment_abbreviation: Cow<'a, str>,
    /// The ordered list of elements in the segment.
    #[serde(borrow)]
    pub elements: VecDeque<Cow<'a, str>>,
}

impl<'a> GenericSegment<'a> {
    #[doc(skip)]
    /// Given [SegmentTokens](struct.SegmentTokens.html), construct a [GenericSegment].
    pub(crate) fn parse_from_tokens(
        tokens: SegmentTokens<'a>,
    ) -> Result<GenericSegment, EdiParseError> {
        let elements: Vec<&str> = tokens.iter().map(|x| x.trim()).collect();
        edi_assert!(
            elements.len() >= 2,
            "at least two elements are required in a segment",
            tokens
        );
        let segment_abbreviation = Cow::from(elements[0]);

        let elements = elements[1..]
            .to_vec()
            .iter()
            .map(|x| Cow::from(*x))
            .collect::<VecDeque<Cow<str>>>();

        Ok(GenericSegment {
            segment_abbreviation,
            elements,
        })
    }

    /// Converts a single generic segment into an ANSI x12 compliant string to be used in an EDI
    /// document.
    pub fn to_x12_string(&self, element_delimiter: char) -> String {
        self.elements
            .iter()
            .fold(self.segment_abbreviation.to_string(), |mut acc, s| {
                acc.push(element_delimiter);
                acc.push_str(s);
                acc
            })
    }
}

#[test]
fn convert_generic_segment_to_string() {
    let segment = GenericSegment {
        segment_abbreviation: Cow::from("BGN"),
        elements: vec!["20", "TEST_ID", "200615", "0000"]
            .iter()
            .map(|x| Cow::from(*x))
            .collect::<VecDeque<Cow<str>>>(),
    };

    assert_eq!(segment.to_x12_string('*'), "BGN*20*TEST_ID*200615*0000");
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
