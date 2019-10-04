// This file contains the tests which validate that envelope validation is working.
// Envelope validation is the process of validating that the data provided in the IEA/GE/SE segments
// matches up with their opener segments and the amount of segments received.
// Note that SE segments count themselves as part of the count for whatever reason, effectively adding two
// to the count.
use edi::{loose_parse, parse};
#[test]
#[should_panic]
fn parse_empty_document() {
    parse("").unwrap();
}

#[test]
#[should_panic]
fn missing_interchange() {
    let input = "GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010~
ST*850*000000001~
BEG*****~
TEST*****~
ANY_AMOUNT_OF_CHARS_IS_OKAY_ALTHOUGH_ATYPICAL*****~
lowercase is chill***********************~
BEG*****~
BEG*****~
BEG*****~
BEG*****~
SE*10*000000001~
GE*1*1~";
    parse(input).unwrap();
}

#[test]
#[should_panic]
fn incorrect_number_of_segments() {
    let input = "ISA*  *          *  *          *ZZ*SENDERISA      *14*0073268795005  *020226*1534*U*00401*000000001*0*T*>~
GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010~
ST*850*000000001~
BEG*****~
TEST*****~
ANY_AMOUNT_OF_CHARS_IS_OKAY_ALTHOUGH_ATYPICAL*****~
lowercase is chill***********************~
BEG*****~
BEG*****~
BEG*****~
BEG*****~
SE*11*000000001~
GE*1*1~
IEA*1*000000001~"; // (SE is 11 when it should be 10)
    parse(input).unwrap();
}
#[test]
#[should_panic]
fn incorrect_number_of_functional_groups() {
    // this document has two functional groups but the interchange header says there should be only one
    // it also uses newlines as the separator
    let test_input = "ISA*01*0000000000*01*0000000000*ZZ*ABCDEFGHIJKLMNO*ZZ*123456789012345*101127*1719*U*00400*000001320*0*P*>
GS*IN*4405197800*999999999*20101205*1710*1320*X*004010VICS
ST*810*1004
BIG*20101204*217224*20101204*P792940
REF*DP*099
REF*IA*99999
N1*ST**92*123
ITD*01*3***0**60
IT1*1*4*EA*8.60**UP*999999330023
IT1*2*2*EA*15.00**UP*999999330115
IT1*3*2*EA*7.30**UP*999999330146
IT1*4*4*EA*17.20**UP*999999330184
IT1*5*8*EA*4.30**UP*999999330320
IT1*6*4*EA*4.30**UP*999999330337
IT1*7*6*EA*1.50**UP*999999330634
IT1*8*6*EA*1.50**UP*999999330641
TDS*21740
CAD*****GTCT**BM*99999
CTT*8
SE*18*1004
GE*1*1320
GS*IN*4405197800*999999999*20101205*1710*1320*X*004010VICS
ST*810*1004
BIG*20101204*217224*20101204*P792940
REF*DP*099
REF*IA*99999
N1*ST**92*123
ITD*01*3***0**60
IT1*1*4*EA*8.60**UP*999999330023
IT1*2*2*EA*15.00**UP*999999330115
IT1*3*2*EA*7.30**UP*999999330146
IT1*4*4*EA*17.20**UP*999999330184
IT1*5*8*EA*4.30**UP*999999330320
IT1*6*4*EA*4.30**UP*999999330337
IT1*7*6*EA*1.50**UP*999999330634
IT1*8*6*EA*1.50**UP*999999330641
TDS*21740
CAD*****GTCT**BM*99999
CTT*8
SE*18*1004
GE*1*1320
IEA*1*000001320";

    parse(test_input).unwrap();
}

#[test]
#[should_panic]
fn missing_functional_group() {
    let test_input = "ISA*01*0000000000*01*0000000000*ZZ*ABCDEFGHIJKLMNO*ZZ*123456789012345*101127*1719*U*00400*000003438*0*P*>
    ST*997*0001
    AK1*PO*1421
    AK9*A*1*1*1
    SE*4*0001
    IEA*1*000003438";

    parse(test_input).unwrap();
}

// The below tests should _not_ fail because they are loose parsing and thus allow these inconsistencies.
#[test]
fn incorrect_number_of_functional_groups_loose_parse() {
    // this document has two functional groups but the interchange header says there should be only one
    // it also uses newlines as the separator
    let test_input = "ISA*01*0000000000*01*0000000000*ZZ*ABCDEFGHIJKLMNO*ZZ*123456789012345*101127*1719*U*00400*000001320*0*P*>
GS*IN*4405197800*999999999*20101205*1710*1320*X*004010VICS
ST*810*1004
BIG*20101204*217224*20101204*P792940
REF*DP*099
REF*IA*99999
N1*ST**92*123
ITD*01*3***0**60
IT1*1*4*EA*8.60**UP*999999330023
IT1*2*2*EA*15.00**UP*999999330115
IT1*3*2*EA*7.30**UP*999999330146
IT1*4*4*EA*17.20**UP*999999330184
IT1*5*8*EA*4.30**UP*999999330320
IT1*6*4*EA*4.30**UP*999999330337
IT1*7*6*EA*1.50**UP*999999330634
IT1*8*6*EA*1.50**UP*999999330641
TDS*21740
CAD*****GTCT**BM*99999
CTT*8
SE*18*1004
GE*1*1320
GS*IN*4405197800*999999999*20101205*1710*1320*X*004010VICS
ST*810*1004
BIG*20101204*217224*20101204*P792940
REF*DP*099
REF*IA*99999
N1*ST**92*123
ITD*01*3***0**60
IT1*1*4*EA*8.60**UP*999999330023
IT1*2*2*EA*15.00**UP*999999330115
IT1*3*2*EA*7.30**UP*999999330146
IT1*4*4*EA*17.20**UP*999999330184
IT1*5*8*EA*4.30**UP*999999330320
IT1*6*4*EA*4.30**UP*999999330337
IT1*7*6*EA*1.50**UP*999999330634
IT1*8*6*EA*1.50**UP*999999330641
TDS*21740
CAD*****GTCT**BM*99999
CTT*8
SE*18*1004
GE*1*1320
IEA*1*000001320";

    loose_parse(test_input).unwrap();
}

#[test]
fn incorrect_number_of_segments_loose_parse() {
    let input = "ISA*  *          *  *          *ZZ*SENDERISA      *14*0073268795005  *020226*1534*U*00401*000000001*0*T*>~
GS*PO*SENDERGS*007326879*20020226*1534*1*X*004010~
ST*850*000000001~
BEG*****~
TEST*****~
ANY_AMOUNT_OF_CHARS_IS_OKAY_ALTHOUGH_ATYPICAL*****~
lowercase is chill***********************~
BEG*****~
BEG*****~
BEG*****~
BEG*****~
SE*11*000000001~
GE*1*1~
IEA*1*000000001~"; // (SE is 11 when it should be 10)
    loose_parse(input).unwrap();
}
