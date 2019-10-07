//! # EDI
//! This crate is for parsing and acting on EDI X12 documents. It exposes two main entry points for parsing:
//! `loose_parse()` and `parse()`. The strictest parsing mode is `parse()`, which parses a string and constructs
//! an [EdiDocument] in a zero-copy (almost -- more on that later) way. `loose_parse()` does the same thing, but
//! does not check and validate that segment closers (IEA, GE, SE) match their openers' ID or have the correct amount
//! of records.
//!
//! ## Zero Copy
//! Under the hood, this crate uses `std::borrow::Cow<&str>` for processing EDI documents. This means that if you are
//! not writing or mutating the document, it is zero copy. As soon as you write to or mutate any part of the `EdiDocument`,
//! that one part is copied. See the documentation for `std::borrow::Cow` for more details.
//!
//! ## Serialization
//! This crate also supports zero-copy serialization and deserialization via [`serde`](https://serde.rs). That means that this crate,
//! with the help of `serde`, also supports serialization to:
//! * YAML
//! * JSON
//! * Bincode
//! * Pickle
//! * TOML
//! * [and more...](https://serde.rs/#data-formats)
//!
//! # Getting Started
//! There are examples in the [examples directory](https://github.com/sezna/edi/tree/master/examples).

#![deny(missing_docs)]
pub use edi_document::EdiDocument;
pub use edi_document::{loose_parse, parse};
pub use functional_group::FunctionalGroup;
pub use generic_segment::GenericSegment;
pub use interchange_control::InterchangeControl;
pub use transaction::Transaction;

#[macro_use]
mod edi_parse_error;
mod edi_document;
mod functional_group;
mod generic_segment;
mod interchange_control;
mod tokenizer;
mod transaction;
