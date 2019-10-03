use crate::edi_parse_error::EdiParseError;
use crate::segment::Segment;
use crate::transaction::Transaction;

use std::borrow::Cow;

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
    transactions: Vec<Transaction<'a>>,
}

impl<'a> FunctionalGroup<'a> {
    pub fn parse_from_str(
        input: &'a str,
        element_delimiter: char,
    ) -> Result<FunctionalGroup<'a>, EdiParseError> {
        let elements: Vec<&str> = input.split(element_delimiter).map(|x| x.trim()).collect();
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
            transactions: Vec::new(), // TODO
        })
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
        transactions: Vec::new(),
    };

    let test_input = "GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010";

    assert_eq!(
        FunctionalGroup::parse_from_str(test_input, '*').unwrap(),
        expected_result
    );
}
