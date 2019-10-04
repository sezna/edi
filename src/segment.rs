use std::borrow::Cow;
/// A generic segment.
pub struct Segment<'a> {
    segment_abbreviation: Cow<'a, str>,
    segment_name: &'static str,
    elements: Vec<Cow<'a, str>>,
}
