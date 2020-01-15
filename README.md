![docs.rs](https://docs.rs/edi/badge.svg)
![crates.io](https://img.shields.io/crates/v/edi.svg)
[![Build Status](https://travis-ci.org/sezna/edi.svg?branch=master)](https://travis-ci.org/sezna/edi)
# Overview
[Check the documentation for more details](https://docs.rs/edi).
### A quick summary of features
* Provides two top-level parsing functions: `parse` and `loose_parse`. `loose_parse` is less strict on the format of the incoming EDI document.
* Parses a valid X12 EDI document into a struct called `EdiDocument`.
* Provides verbose error messages if the document being parsed is invalid.
  * Error messages include the actual segment in which the error occurred.
* `EdiDocument` and all data it contains implement `Serialize` and `Deserialize` from `serde`, so zero-copy serialization and deserialization to any serde-able format is supported (this includes json).
* `EdiDocument`'s fields are all public and it can be navigated like any other struct for simplicity

See the [examples directory](https://github.com/sezna/edi/tree/master/examples) for an example.


### A quick summary of limitations
* Cannot accurately determine segment types, as that requires an implementation guide from the individual transactor
* Cannot detect loops for the same reason as above
* Only supports standard X12 EDI


# Roadmap
  * benches to identify regressions
  * output back into EDI with proper padding in the ISA segment
  * iterator over segments for the frequent cases in which there's only one transaction/functional group/interchange (EdiDocument.segments_iter() -> SegmentIter?)
    * to_string(delimiters) is needed to output the edi document
