use crate::edi_parse_error::EdiParseError;

use crate::transaction::Transaction;

use crate::tokenizer::SegmentTokens;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::VecDeque;

/// Represents a GS/GE segment which wraps a functional group.
/// Documentation here gleaned mostly from [here](http://u.sezna.dev/b)
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct FunctionalGroup<'a> {
    /// Identifies the function of this group.
    /// See http://ecomgx17.ecomtoday.com/edi/EDI_4010/el479.htm for a list of
    /// functional identifier codes.
    #[serde(borrow)]
    pub functional_identifier_code: Cow<'a, str>,
    /// Identifies the sender of this group.
    #[serde(borrow)]
    pub application_sender_code: Cow<'a, str>,
    /// Identifies the receiver of this group.
    #[serde(borrow)]
    pub application_receiver_code: Cow<'a, str>,
    /// Identifies the date of the function performed.
    #[serde(borrow)]
    pub date: Cow<'a, str>,
    /// Identifies the time of the function performed.
    ///  Expressed in 24-hour clock time as follows: HHMM, or HHMMSS, or
    /// HHMMSSD, or HHMMSSDD, where H = hours (00-23), M = minutes (00-59), S = integer
    /// seconds (00-59) and DD = decimal seconds; decimal seconds are expressed as follows: D
    /// = tenths (0-9) and DD = hundredths (00-99)
    #[serde(borrow)]
    pub time: Cow<'a, str>,
    /// An ID code for this specific control group. Should
    /// be the same in the GE (group end) segment.
    #[serde(borrow)]
    pub group_control_number: Cow<'a, str>,
    /// Code identifying the issuer of the standard
    #[serde(borrow)]
    pub responsible_agency_code: Cow<'a, str>,
    ///  Code indicating the version, release, subrelease, and industry identifier of the
    ///  EDI standard being used, including the GS and GE segments; If code DE455 in GS
    /// segment is X, then in DE 480 positions 1-3 are the version number; positions 4-6 are the
    /// release and subrelease, level of the version; and positions 7-12 are the industry or trade
    /// association identifiers (optionally assigned by user); if code in DE455 in GS segment is T,
    /// then other formats are allowed
    #[serde(borrow)]
    pub version: Cow<'a, str>,
    /// The transactions that this functional group contains.
    #[serde(borrow = "'a")]
    pub transactions: VecDeque<Transaction<'a>>,
}

impl<'a, 'b> FunctionalGroup<'a> {
    /// Given [SegmentTokens](struct.SegmentTokens.html) (where the first token is "GS"), construct a [FunctionalGroup].
    pub(crate) fn parse_from_tokens(
        input: SegmentTokens<'a>,
    ) -> Result<FunctionalGroup<'a>, EdiParseError> {
        let elements: Vec<&str> = input.iter().map(|x| x.trim()).collect();
        // I always inject invariants wherever I can to ensure debugging is quick and painless,
        // and to check my assumptions.
        edi_assert!(
            elements[0] == "GS",
            "attempted to parse GS from non-GS segment",
            input
        );
        edi_assert!(
            elements.len() >= 9,
            "GS segment does not contain enough elements. At least 9 required",
            input
        );
        let (
            functional_identifier_code,
            application_sender_code,
            application_receiver_code,
            date,
            time,
            group_control_number,
            responsible_agency_code,
            version,
        ) = (
            Cow::from(elements[1]),
            Cow::from(elements[2]),
            Cow::from(elements[3]),
            Cow::from(elements[4]),
            Cow::from(elements[5]),
            Cow::from(elements[6]),
            Cow::from(elements[7]),
            Cow::from(elements[8]),
        );

        Ok(FunctionalGroup {
            functional_identifier_code,
            application_sender_code,
            application_receiver_code,
            date,
            time,
            group_control_number,
            responsible_agency_code,
            version,
            transactions: VecDeque::new(),
        })
    }

    /// Enqueue a [Transaction] into the group. Subsequent segments will be enqueued into this transaction.
    pub(crate) fn add_transaction(
        &mut self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        self.transactions
            .push_back(Transaction::parse_from_tokens(tokens)?);
        Ok(())
    }

    /// Enqueue a [GenericSegment](struct.GenericSegment.html) into the most recently enqueued [Transaction].
    pub(crate) fn add_generic_segment(
        &mut self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        if let Some(transaction) = self.transactions.back_mut() {
            transaction.add_generic_segment(tokens)
        } else {
            Err(EdiParseError::new(
                "unable to enqueue generic segment when no transactions have been enqueued",
                Some(tokens),
            ))
        }
    }

    /// Verify this [FunctionalGroup] with a GE segment.
    pub(crate) fn validate_functional_group(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        edi_assert!(
            tokens[0] == "GE",
            "attempted to call GE verification on non-GE segment",
            tokens
        );
        edi_assert!(
            self.transactions.len() == str::parse::<usize>(tokens[1]).unwrap(),
            "functional group validation failed: incorrect number of transactions",
            self.transactions.len(),
            str::parse::<usize>(tokens[1]).unwrap(),
            tokens
        );
        edi_assert!(
            self.group_control_number == tokens[2],
            "functional group validation failed: mismatched ID",
            self.group_control_number,
            tokens[2],
            tokens
        );
        Ok(())
    }

    /// Validate the latest [Transaction] within this functional group with an SE segment.
    pub(crate) fn validate_transaction(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        if let Some(transaction) = self.transactions.back() {
            transaction.validate_transaction(tokens)
        } else {
            Err(EdiParseError::new(
                "unable to validate nonexistent transaction",
                Some(tokens),
            ))
        }
    }

    /// Converts this functional group into an ANSI x12 string for use in an EDI document.
    pub fn to_x12_string(&self, segment_delimiter: char, element_delimiter: char) -> String {
        let header = String::from("GS");
        let elements_of_gs = vec![
            self.functional_identifier_code.clone(),
            self.application_sender_code.clone(),
            self.application_receiver_code.clone(),
            self.date.clone(),
            self.time.clone(),
            self.group_control_number.clone(),
            self.responsible_agency_code.clone(),
            self.version.clone(),
        ];

        let mut buffer = elements_of_gs.iter().fold(header, |mut acc, elem| {
            acc.push(element_delimiter);
            acc.push_str(&elem);
            acc
        });
        let transactions = self
            .transactions
            .iter()
            .fold(String::new(), |mut acc, transaction| {
                acc.push(segment_delimiter);
                acc.push_str(&transaction.to_x12_string(segment_delimiter, element_delimiter));
                acc
            });

        buffer.push_str(&transactions);

        let mut closer = String::from("GE");
        closer.push(element_delimiter);
        closer.push_str(&self.transactions.len().to_string());
        closer.push(element_delimiter);
        closer.push_str(&self.group_control_number);

        buffer.push(segment_delimiter);
        buffer.push_str(&closer);
        buffer
    }
}

#[test]
fn functional_group_to_string() {
    use crate::GenericSegment;
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

    let functional_group = FunctionalGroup {
        functional_identifier_code: Cow::from("PO"),
        application_sender_code: Cow::from("SENDERGS"),
        application_receiver_code: Cow::from("007326879"),
        date: Cow::from("20020226"),
        time: Cow::from("1534"),
        group_control_number: Cow::from("1"),
        responsible_agency_code: Cow::from("X"),
        version: Cow::from("004010"),
        transactions: VecDeque::from_iter(vec![transaction].into_iter()),
    };
    assert_eq!(functional_group.to_x12_string('\n', '*'), "GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010\nST*140*100000001*\nBGN*20*TEST_ID*200615*0000\nBGN*15*OTHER_TEST_ID***END\nSE*4*100000001\nGE*1*1");
}

#[test]
fn construct_functional_group() {
    let expected_result = FunctionalGroup {
        functional_identifier_code: Cow::from("PO"),
        application_sender_code: Cow::from("SENDERGS"),
        application_receiver_code: Cow::from("007326879"),
        date: Cow::from("20020226"),
        time: Cow::from("1534"),
        group_control_number: Cow::from("1"),
        responsible_agency_code: Cow::from("X"),
        version: Cow::from("004010"),
        transactions: VecDeque::new(),
    };

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

    assert_eq!(
        FunctionalGroup::parse_from_tokens(test_input).unwrap(),
        expected_result
    );
}
