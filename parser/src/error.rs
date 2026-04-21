#[derive(Debug, PartialEq)]
pub enum ReaderError {
    ParseError(String),
}

#[derive(Debug, PartialEq)]
pub enum WriterError {
    WriterError(String),
}
