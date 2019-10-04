use crate::edi_parse_error::EdiParseError;
use crate::functional_group::FunctionalGroup;
use std::borrow::Cow;

/// Represents the ISA/IEA header information commonly known as the "envelope" in X12 EDI.
#[derive(PartialEq, Debug)]
struct InterchangeControl<'a> {
    // I chose to use `Cow`s here because I don't know how the crate will be used --
    // given enough documents of sufficient size and a restrictive enough environment,
    // the space complexity could undesirably grow. This allows for some mitigation
    // and while it isn't zero-copy is at least less-copy.
    authorization_qualifier: Cow<'a, str>,
    authorization_information: Cow<'a, str>,
    security_qualifier: Cow<'a, str>,
    security_information: Cow<'a, str>,
    sender_qualifier: Cow<'a, str>,
    sender_id: Cow<'a, str>,
    receiver_qualifier: Cow<'a, str>,
    receiver_id: Cow<'a, str>,
    date: Cow<'a, str>, // chrono::Date?
    time: Cow<'a, str>, // chrono::Time?
    standards_id: Cow<'a, str>,
    version: Cow<'a, str>,                    // u64?
    interchange_control_number: Cow<'a, str>, // u64?
    acknowledgement_requested: Cow<'a, str>,  // bool?  0 for false, 1 for true
    test_indicator: Cow<'a, str>,             // P for production, T for test
    functional_groups: Vec<FunctionalGroup<'a>>,
}

impl<'a> InterchangeControl<'a> {
    pub fn parse_from_str(input: Vec<&'a str>) -> Result<InterchangeControl<'a>, EdiParseError> {
        let elements: Vec<&str> = input.iter().map(|x| x.trim()).collect();
        // I always inject invariants wherever I can to ensure debugging is quick and painless,
        // and to check my assumptions.
        edi_assert!(
            elements[0] == "ISA",
            "attempted to parse ISA from non-ISA segment"
        );
        edi_assert!(
            elements.len() >= 16,
            "ISA segment does not contain enough elements",
            elements.len()
        );
        let (
            authorization_qualifier,
            authorization_information,
            security_qualifier,
            security_information,
            sender_qualifier,
            sender_id,
            receiver_qualifier,
            receiver_id,
            date,
            time,
            standards_id,
            version,
            interchange_control_number,
            acknowledgement_requested,
            test_indicator,
        ) = (
            Cow::from(elements[1]),
            Cow::from(elements[2]),
            Cow::from(elements[3]),
            Cow::from(elements[4]),
            Cow::from(elements[5]),
            Cow::from(elements[6]),
            Cow::from(elements[7]),
            Cow::from(elements[8]),
            Cow::from(elements[9]),
            Cow::from(elements[10]),
            Cow::from(elements[11]),
            Cow::from(elements[12]),
            Cow::from(elements[13]),
            Cow::from(elements[14]),
            Cow::from(elements[15]),
        );
        Ok(InterchangeControl {
            authorization_qualifier,
            authorization_information,
            security_qualifier,
            security_information,
            sender_qualifier,
            sender_id,
            receiver_qualifier,
            receiver_id,
            date,
            time,
            standards_id,
            version,
            interchange_control_number,
            acknowledgement_requested,
            test_indicator,
            functional_groups: Vec::new(), // TODO
        })
    }
}

// For tests that check behavior of private fields or structs, I put the tests in
// the same file. This avoids unnecessary `pub` at the cost of messier test organization.
// I think the trade-off is worth it, and the organizational loss is not that bad.
// Of course, this is subjective.
#[test]
fn construct_interchange_control() {
    let expected_result = InterchangeControl {
        authorization_qualifier: Cow::from("00"),
        authorization_information: Cow::from(""),
        security_qualifier: Cow::from("00"),
        security_information: Cow::from(""),
        sender_qualifier: Cow::from("ZZ"),
        sender_id: Cow::from("SENDERISA"),
        receiver_qualifier: Cow::from("14"),
        receiver_id: Cow::from("0073268795005"),
        date: Cow::from("020226"),
        time: Cow::from("1534"),
        standards_id: Cow::from("U"),
        version: Cow::from("00401"),
        interchange_control_number: Cow::from("000000001"),
        acknowledgement_requested: Cow::from("0"),
        test_indicator: Cow::from("T"),
        functional_groups: Vec::new(),
    };

    let test_input = vec![
        "ISA",
        "00",
        "",
        "00",
        "",
        "ZZ",
        "SENDERISA",
        "14",
        "0073268795005",
        "020226",
        "1534",
        "U",
        "00401",
        "000000001",
        "0",
        "T",
    ];
    assert_eq!(
        InterchangeControl::parse_from_str(test_input,).unwrap(),
        expected_result
    );
}
