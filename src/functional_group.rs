use crate::edi_parse_error::EdiParseError;

use crate::transaction::Transaction;

use crate::tokenizer::SegmentTokens;
use std::borrow::Cow;
use std::collections::VecDeque;

/// Represents a GS/GE segment which wraps a functional group.
#[derive(PartialEq, Debug)]
pub struct FunctionalGroup<'a> {
    functional_identifier_code: Cow<'a, str>,
    application_sender_code: Cow<'a, str>,
    application_receiver_code: Cow<'a, str>,
    date: Cow<'a, str>,
    time: Cow<'a, str>,
    group_control_number: Cow<'a, str>,
    responsible_agency_code: Cow<'a, str>,
    version: Cow<'a, str>,
    transactions: VecDeque<Transaction<'a>>,
}

impl<'a> FunctionalGroup<'a> {
    pub fn parse_from_tokens(
        input: SegmentTokens<'a>,
    ) -> Result<FunctionalGroup<'a>, EdiParseError> {
        let elements: Vec<&str> = input.iter().map(|x| x.trim()).collect();
        // I always inject invariants wherever I can to ensure debugging is quick and painless,
        // and to check my assumptions.
        edi_assert!(
            elements[0] == "GS",
            "attempted to parse GS from non-GS segment"
        );
        edi_assert!(
            elements.len() >= 9,
            "GS segment does not contain enough elements",
            elements.len()
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
    pub fn add_transaction(&mut self, tokens: SegmentTokens<'a>) {
        self.transactions.push_back(
            Transaction::parse_from_tokens(tokens).expect("failed to parse transaction header"),
        );
    }

    /// Enqueue a [GenericSegment] into the most recently enqueued [Transaction].
    pub fn add_generic_segment(&mut self, tokens: SegmentTokens<'a>) {
        self.transactions
            .back_mut()
            .expect("unable to enqueue generic segment when no transactions have been enqueued")
            .add_generic_segment(tokens);
    }

    /// Verify this [FunctionalGroup] with a GE segment.
    pub fn validate_functional_group(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        edi_assert!(
            tokens[0] == "GE",
            "attempted to call GE verification on non-GE segment"
        );
        edi_assert!(
            self.transactions.len() == str::parse::<usize>(tokens[1]).unwrap(),
            "functional group validation failed: incorrect number of transactions"
        );
        edi_assert!(
            self.group_control_number == tokens[2],
            "functional group validation failed: mismatched ID"
        );
        Ok(())
    }

    /// Validate the latest [Transaction] within this functional group with an SE segment.
    pub fn validate_transaction(&self, tokens: SegmentTokens<'a>) -> Result<(), EdiParseError> {
        self.transactions
            .back()
            .expect("unable to validate nonexistent transaction")
            .validate_transaction(tokens)
    }
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
