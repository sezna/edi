
pub type Tokens<'a> = Vec<&'a str>;

/// The input is the entire EDI document string, and the output is a 2d array of edi segments and their elements.
/// Optionally, an element can also have subelements.
//pub fn tokenizer(input: &'a str)