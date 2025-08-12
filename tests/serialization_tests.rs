use edi::parse;

#[test]
fn test_json_serialization() {
    let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
REF*DP*123456~
SE*4*0001~
GE*1*1~
IEA*1*000000001~";
    
    let doc = parse(input).unwrap();
    
    // Serialize to JSON
    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("\"sender_id\":\"SENDER\""));
    assert!(json.contains("\"receiver_id\":\"RECEIVER\""));
    assert!(json.contains("\"element_delimiter\":\"*\""));
    
    // Deserialize from JSON
    let deserialized: edi::EdiDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.interchanges.len(), 1);
    assert_eq!(deserialized.element_delimiter, '*');
    assert_eq!(deserialized.segment_delimiter, '~');
}

#[test]
fn test_json_serialization_preserves_structure() {
    let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
ITD*01*3*1**15**16~
DTM*002*19971219~
N1*BT*BUYSNACKS INC.*9*1223334444~
N3*P.O. BOX 0000~
N4*TEMPLE*TX*76503~
SE*8*0001~
GE*1*1~
IEA*1*000000001~";
    
    let doc = parse(input).unwrap();
    
    // Serialize and deserialize
    let json = serde_json::to_string(&doc).unwrap();
    let deserialized: edi::EdiDocument = serde_json::from_str(&json).unwrap();
    
    // Check structure is preserved
    assert_eq!(
        doc.interchanges[0].functional_groups[0].transactions[0].segments.len(),
        deserialized.interchanges[0].functional_groups[0].transactions[0].segments.len()
    );
    
    // Check specific segment data
    let orig_n1 = &doc.interchanges[0].functional_groups[0].transactions[0].segments[3];
    let deser_n1 = &deserialized.interchanges[0].functional_groups[0].transactions[0].segments[3];
    assert_eq!(orig_n1.segment_abbreviation, deser_n1.segment_abbreviation);
    assert_eq!(orig_n1.elements.len(), deser_n1.elements.len());
}

#[test]
fn test_pretty_json_output() {
    let input = "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *200301*1253*U*00401*000000001*0*T*>~
GS*PO*SENDER*RECEIVER*20200301*1253*1*X*004010~
ST*850*0001~
BEG*00*SA*PO123**20200301~
SE*3*0001~
GE*1*1~
IEA*1*000000001~";
    
    let doc = parse(input).unwrap();
    let pretty_json = serde_json::to_string_pretty(&doc).unwrap();
    
    // Pretty JSON should have proper formatting
    assert!(pretty_json.contains("\n"));
    assert!(pretty_json.contains("  "));
}