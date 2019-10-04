use crate::edi_parse_error::EdiParseError;
/// The type that represents a 2d vec of tokens representing EDI segments and their elements.
pub type DocumentTokens<'a> = Vec<SegmentTokens<'a>>;
/// The type that represents an individual segment's tokens.
pub type SegmentTokens<'a> = Vec<&'a str>;

/// The input is the entire EDI document string, and the output is a 2d array of edi segments and their elements.
/// If an element has subelements, they are not separated into separate tokens. It also performs some basic
/// sanity checks to see if the input is of the format we are expecting and validates that all ISA/GS openers
/// are closed.
pub fn tokenize(input: &str) -> Result<DocumentTokens, EdiParseError> {
    edi_assert!(
        input.len() >= 106,
        "input not long enough to contain ISA header delimiters"
    );
    let delimiters_str: Vec<char> = input[103..106].chars().collect();
    let (element_delimiter, sub_element_delimiter, segment_delimiter) =
        (delimiters_str[0], delimiters_str[1], delimiters_str[2]);
    edi_assert!(
        element_delimiter != sub_element_delimiter,
        "element and subelement delimiters cannot be the same"
    );
    edi_assert!(
        sub_element_delimiter != segment_delimiter,
        "subelement and segment delimiters cannot be the same"
    );
    edi_assert!(
        element_delimiter != segment_delimiter,
        "element and segment delimiters cannot be the same"
    );
    // Filter out any empty segments caused by newlines.
    let segments: SegmentTokens = input
        .split(segment_delimiter)
        .map(|x| x.trim())
        .filter(|x| *x != "")
        .collect();
    let tokens: DocumentTokens = segments
        .iter()
        .map(|x| x.split(element_delimiter).collect::<Vec<&str>>())
        .collect();

    Ok(tokens)
}

// I tend to put individual unit tests inside the file they belong to, and E2E/integration tests in the tests directory.
#[test]
fn basic_segment_tokenize() {
    let test_input = "ISA*00*          *00*          *ZZ*SENDERISA      *14*0073268795005  *020226*1534*U*00401*000000001*0*T*>~
GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010~
ST*850*000000001~
BEG*00*SA*A99999-01**19970214~
REF*VR*54321~
ITD*01*3*1**15**16~
DTM*002*19971219~
DTM*002*19971219~
SE*35*000000001~
GE*1*1~
IEA*1*000000001~";

    let tokens = tokenize(test_input).unwrap();
    assert_eq!(tokens.len(), 11);
    assert_eq!(tokens[0].len(), 17)
}

#[test]
fn fail_to_tokenize_no_header() {
    let test_input =
        "00*          *ZZ*SENDERISA      *14*0073268795005  *020226*1534*U*00401*000000001*0*T";
    assert!(tokenize(test_input).is_err());
}

#[test]
fn fail_same_delimiters() {
    let test_input = "ISA*00*          *00*          *ZZ*SENDERISA      *14*0073268795005  *020226*1534*U*00401*000000001*0*T~~~
GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010~
ST*850*000000001~";
    assert!(tokenize(test_input).is_err());
}
