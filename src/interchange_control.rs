use crate::edi_parse_error::EdiParseError;
use crate::functional_group::FunctionalGroup;

use crate::tokenizer::SegmentTokens;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::VecDeque;

/// Represents the ISA/IEA header information commonly known as the "envelope" in X12 EDI.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct InterchangeControl<'a, 'b> {
    // I chose to use `Cow`s here because I don't know how the crate will be used --
    // given enough documents of sufficient size and a restrictive enough environment,
    // the space complexity could undesirably grow. This allows for some mitigation
    // and while it isn't zero-copy is at least less-copy.
    /// Code to identify the type of information in the Authorization Information.
    ///
    /// Qualifiers are two-digit prefixes which categorize the following element.
    #[serde(borrow)]
    pub authorization_qualifier: Cow<'a, str>,
    /// Information used for additional identification or authorization of the
    /// interchange sender or the data in the interchange; the type of information is set by the
    /// Authorization Information Qualifier.
    #[serde(borrow)]
    pub authorization_information: Cow<'a, str>,
    /// Code to identify the type of information in the Security Information
    ///
    /// Qualifiers are two-digit prefixes which categorize the following element.
    #[serde(borrow)]
    pub security_qualifier: Cow<'a, str>,
    /// This is used for identifying the security information about the interchange
    /// sender or the data in the interchange; the type of information is set by the Security
    /// Information Qualifier
    #[serde(borrow)]
    pub security_information: Cow<'a, str>,
    /// Qualifier to designate the system/method of code structure used to designate
    /// the sender ID.
    ///
    /// Qualifiers are two-digit prefixes which categorize the following element.
    #[serde(borrow)]
    pub sender_qualifier: Cow<'a, str>,
    /// Identification code published by the sender for other parties to use as the
    /// receiver ID to route data to them; the sender always codes this value in the sender ID
    /// element
    #[serde(borrow)]
    pub sender_id: Cow<'a, str>,
    /// Qualifier to designate the system/method of code structure used to designate
    /// the receiver ID.
    ///
    /// Qualifiers are two-digit prefixes which categorize the following element.
    #[serde(borrow)]
    pub receiver_qualifier: Cow<'a, str>,
    /// Identification code published by the receiver of the data; When sending, it is
    /// used by the sender as their sending ID, thus other parties sending to them will use this as a
    /// receiving ID to route data to them
    #[serde(borrow)]
    pub receiver_id: Cow<'a, str>,
    /// Date of the interchange
    #[serde(borrow)]
    pub date: Cow<'a, str>, // chrono::Date?
    /// Time of the interchange
    #[serde(borrow)]
    pub time: Cow<'a, str>, // chrono::Time?
    /// Code to identify the agency responsible for the control standard used by the
    /// message that is enclosed by the interchange header and trailer
    #[serde(borrow)]
    pub standards_id: Cow<'a, str>,
    /// Code specifying the version number of the interchange control segments
    #[serde(borrow)]
    pub version: Cow<'a, str>, // u64?
    /// A control number assigned by the interchange sender
    #[serde(borrow)]
    pub interchange_control_number: Cow<'a, str>, // u64?
    /// Either a 0 or a 1 denoting that acknowledgment is not requested (0) or it is
    /// requested (1).
    #[serde(borrow)]
    pub acknowledgement_requested: Cow<'a, str>, // bool?  0 for false, 1 for true
    /// Code to indicate whether data enclosed by this interchange envelope is test ("T"),
    /// production ("P"), or information ("I").
    #[serde(borrow)]
    pub test_indicator: Cow<'a, str>, // P for production, T for test
    /// The [FunctionalGroups](struct.FunctionalGroup.html) contained in this interchange.
    #[serde(borrow = "'a + 'b")]
    pub functional_groups: VecDeque<FunctionalGroup<'a, 'b>>,
}

impl<'a, 'b> InterchangeControl<'a, 'b> {
    /// Given [SegmentTokens](struct.SegmentTokens.html) (where the first token is "ISA"), construct an [InterchangeControl].
    pub(crate) fn parse_from_tokens(
        input: SegmentTokens<'a>,
    ) -> Result<InterchangeControl<'a, 'b>, EdiParseError> {
        let elements: Vec<&str> = input.iter().map(|x| x.trim()).collect();
        // I always inject invariants wherever I can to ensure debugging is quick and painless,
        // and to check my assumptions.
        edi_assert!(
            elements[0] == "ISA",
            "attempted to parse ISA from non-ISA segment",
            input
        );
        edi_assert!(
            elements.len() >= 16,
            "ISA segment does not contain enough elements. At least 16 required",
            input
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
            functional_groups: VecDeque::new(),
        })
    }

    /// Enqueue a [FunctionalGroup] into the interchange. Subsequent [Transaction]s will be inserted into this functional group,
    /// until a new one is enqueued.
    pub(crate) fn add_functional_group(
        &mut self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        self.functional_groups
            .push_back(FunctionalGroup::parse_from_tokens(tokens)?);
        Ok(())
    }

    /// Enqueue a [Transaction] into the most recently enqueued [FunctionalGroup] in this interchange.
    pub(crate) fn add_transaction(
        &mut self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        if let Some(functional_group) = self.functional_groups.back_mut() {
            functional_group.add_transaction(tokens)
        } else {
            Err(EdiParseError::new(
                "unable to enqueue transaction when no functional groups have been added",
                Some(tokens),
            ))
        }
    }

    /// Enqueue a [GenericSegment](struct.GenericSegment.html) into the most recently enqueued [FunctionalGroup]'s most recently enqueued [Transaction](struct.Transaction.html).
    pub(crate) fn add_generic_segment(
        &mut self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        if let Some(functional_group) = self.functional_groups.back_mut() {
            functional_group.add_generic_segment(tokens)
        } else {
            Err(EdiParseError::new(
                "unable to enqueue generic segment when no functional groups have been added",
                Some(tokens),
            ))
        }
    }

    /// Given the tokens of an IEA segment, or Interchange Control closer, verify that the correct
    /// number of control groups have been given.
    pub(crate) fn validate_interchange_control(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        edi_assert!(
            tokens[0] == "IEA",
            "attempted to verify IEA on non-IEA segment",
            tokens
        );
        edi_assert!(
            str::parse::<usize>(&tokens[1].to_string()).unwrap() == self.functional_groups.len(),
            "interchange validation failed: incorrect number of functional groups",
            tokens[1].to_string(),
            self.functional_groups.len(),
            tokens
        );
        edi_assert!(
            tokens[2] == self.interchange_control_number,
            "interchange validation failed: mismatched ID",
            tokens[2],
            self.interchange_control_number.clone(),
            tokens
        );

        Ok(())
    }

    /// Verify the latest [FunctionalGroup] with a GE segment.
    pub(crate) fn validate_functional_group(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        if let Some(functional_group) = self.functional_groups.back() {
            functional_group.validate_functional_group(tokens)
        } else {
            return Err(EdiParseError::new(
                "unable to verify nonexistent functional group",
                Some(tokens),
            ));
        }
    }

    /// Verify the latest [Transaction](struct.Transaction.html) within the latest [FunctionalGroup]
    pub(crate) fn validate_transaction(
        &self,
        tokens: SegmentTokens<'a>,
    ) -> Result<(), EdiParseError> {
        if let Some(functional_group) = self.functional_groups.back() {
            functional_group.validate_transaction(tokens)
        } else {
            return Err(EdiParseError::new(
                "unable to verify transaction within nonexistent functional group",
                Some(tokens),
            ));
        }
    }
}

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
        functional_groups: VecDeque::new(),
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
        InterchangeControl::parse_from_tokens(test_input,).unwrap(),
        expected_result
    );
}
