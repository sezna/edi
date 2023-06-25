use crate::edi_parse_error::EdiParseError;
use crate::generic_segment::GenericSegment;
use crate::tokenizer::SegmentTokens;
use csv::ReaderBuilder;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

/// Represents a transaction in an EDI document. A transaction is initialized with an ST segment
/// and ended with an SE segment.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Transaction<'a> {
    /// The numeric code which represents the type of transaction.
    #[serde(borrow)]
    pub transaction_code: Cow<'a, str>,
    /// The name of the transaction type in human-readable form.
    #[serde(borrow)]
    pub transaction_name: Cow<'a, str>,
    /// Each transaction within a functional group also has a control number.
    /// Typically, trading partners use a number relative to the functional group in which they are contained.
    #[serde(borrow)]
    pub transaction_set_control_number: Cow<'a, str>,
    /// Identifier of the implementation convention reference. Valid value is up to 35 standard characters. Optional.
    #[serde(borrow)]
    pub implementation_convention_reference: Option<Cow<'a, str>>,
    /// The [GenericSegment]s contained within this transaction.
    #[serde(borrow)]
    pub segments: VecDeque<GenericSegment<'a>>,
}

// Load the potential transaction schema names from a csv
// source: scraped from https://www.arcesb.com/edi/standards/x12/
lazy_static! {
    static ref SCHEMAS: HashMap<String, String> = {
        let mut map = HashMap::new();
        let schemas_path = format!("{}/resources/schemas.csv", env!("CARGO_MANIFEST_DIR"));
        let mut schemas_csv = ReaderBuilder::new()
            .has_headers(false)
            .from_path(schemas_path)
            .expect("Failed to open schemas.csv. Does edi/resources/schemas.csv exist?");
        for record in schemas_csv.records() {
            let record = record.unwrap();
            map.insert(record[0].to_string(), record[1].to_string());
        }
        map
    };
}

impl<'a> Transaction<'a> {
    /// Given [SegmentTokens] (where the first token is "ST"), construct a [Transaction].
    pub(crate) fn parse_from_tokens(
        input: SegmentTokens<'a>,
    ) -> Result<Transaction<'a>, EdiParseError> {
        let elements: Vec<&str> = input.iter().map(|x| x.trim()).collect();
        // I always inject invariants wherever I can to ensure debugging is quick and painless,
        // and to check my assumptions.
        edi_assert!(
            elements[0] == "ST",
            "attempted to parse ST from non-ST segment",
            input
        );
        edi_assert!(
            elements.len() >= 3,
            "ST segment does not contain enough elements. At least 3 required",
            input
        );

        let (transaction_code, transaction_set_control_number) =
            (Cow::from(elements[1]), Cow::from(elements[2]));
        let implementation_convention_reference = if elements.len() >= 4 {
            Some(Cow::from(elements[3]))
        } else {
            None
        };
        let transaction_name = if let Some(name) = SCHEMAS.get(&transaction_code.to_string()) {
            name
        } else {
            "unidentified"
        };

        Ok(Transaction {
            transaction_code,
            transaction_name: Cow::from(transaction_name),
            transaction_set_control_number,
            implementation_convention_reference,
            segments: VecDeque::new(),
        })
    }

    /// Enqueue a [GenericSegment](struct.GenericSegment.html) into the transaction.
    pub(crate) fn add_generic_segment(
        &mut self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        self.segments
            .push_back(GenericSegment::parse_from_tokens(tokens)?);
        Ok(())
    }

    /// Validate this transaction with an SE segment.
    pub(crate) fn validate_transaction(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        edi_assert!(
            tokens[0] == "SE",
            "attempted to validate transaction with non-SE segment",
            tokens
        );
        // we have to add two here because transaction counts include ST and SE
        edi_assert!(
            str::parse::<usize>(tokens[1]).unwrap() == self.segments.len() + 2,
            "transaction validation failed: incorrect number of segments",
            tokens[1],
            self.segments.len() + 2,
            tokens
        );
        edi_assert!(
            tokens[2] == self.transaction_set_control_number,
            "transaction validation failed: incorrect transaction ID",
            tokens[2],
            self.transaction_set_control_number,
            tokens
        );
        Ok(())
    }

    /// Converts this [Transaction] into an ANSI x12 string to be used in an EDI document.
    pub fn to_x12_string(&self, segment_delimiter: char, element_delimiter: char) -> String {
        let mut header = "ST".to_string();
        header.push(element_delimiter);
        header.push_str(&self.transaction_code);
        header.push(element_delimiter);
        header.push_str(&self.transaction_set_control_number);
        header.push(element_delimiter);
        header.push_str(
            &self
                .implementation_convention_reference
                .clone()
                .unwrap_or(Cow::Borrowed("")),
        );

        let mut final_string = self.segments.iter().fold(header, |mut acc, segment| {
            acc.push(segment_delimiter);
            acc.push_str(&segment.to_x12_string(element_delimiter));
            acc
        });

        let mut closer = "SE".to_string();
        closer.push(element_delimiter);
        closer.push_str(&(self.segments.len() + 2).to_string()); // +2 because the count includes the ST and SE segments
        closer.push(element_delimiter);
        closer.push_str(&self.transaction_set_control_number.clone());

        final_string.push(segment_delimiter);
        final_string.push_str(&closer);

        final_string
    }
}

#[test]
fn transaction_to_string() {
    use std::iter::FromIterator;
    let segments = VecDeque::from_iter(
        vec![
            GenericSegment {
                segment_abbreviation: Cow::from("BGN"),
                elements: vec!["20", "TEST_ID", "200615", "0000"]
                    .iter()
                    .map(|x| Cow::from(*x))
                    .collect::<VecDeque<Cow<str>>>(),
            },
            GenericSegment {
                segment_abbreviation: Cow::from("BGN"),
                elements: vec!["15", "OTHER_TEST_ID", "", "", "END"]
                    .iter()
                    .map(|x| Cow::from(*x))
                    .collect::<VecDeque<Cow<str>>>(),
            },
        ]
        .into_iter(),
    );
    let transaction = Transaction {
        transaction_code: Cow::from("140"),
        transaction_name: Cow::from(""),
        transaction_set_control_number: Cow::from("100000001"),
        implementation_convention_reference: None,
        segments: segments,
    };

    assert_eq!(
        transaction.to_x12_string('~', '*'),
        "ST*140*100000001*~BGN*20*TEST_ID*200615*0000~BGN*15*OTHER_TEST_ID***END~SE*4*100000001"
    );
}

#[test]
fn construct_transaction() {
    let expected_result = Transaction {
        transaction_code: Cow::from("850"),
        transaction_name: Cow::from(SCHEMAS.get(&"850".to_string()).unwrap()), // should be "Purchase Order"
        transaction_set_control_number: Cow::from("000000001"),
        implementation_convention_reference: None,
        segments: VecDeque::new(),
    };
    let test_input = vec!["ST", "850", "000000001"];

    assert_eq!(
        Transaction::parse_from_tokens(test_input).unwrap(),
        expected_result
    );
}

#[test]
fn spot_check_schemas() {
    assert_eq!(SCHEMAS.get(&"850".to_string()).unwrap(), "Purchase Order");
    assert_eq!(
        SCHEMAS.get(&"100".to_string()).unwrap(),
        "Insurance Plan Description"
    );
    assert_eq!(
        SCHEMAS.get(&"999".to_string()).unwrap(),
        "Implementation Acknowledgment"
    );
}
