use edi::parse;
use std::fs::read_to_string;

fn main() {
    let edi_file_path = format!("{}/examples/sample_edi.txt", env!("CARGO_MANIFEST_DIR"));
    let edi_string = read_to_string(edi_file_path).unwrap();
    let edi_document = parse(&edi_string).unwrap();
    // `edi_document` now contains an `EdiDocument` which we can interact with.
    println!(
        "The EDI document contains {} segments. It is from {} and being sent to {}",
        edi_document.interchanges[0].functional_groups[0].transactions[0]
            .segments
            .len(),
        edi_document.interchanges[0].sender_id,
        edi_document.interchanges[0].receiver_id
    );
}
