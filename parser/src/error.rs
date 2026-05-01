#[derive(Debug, PartialEq)]
pub enum FieldName {
    TxId,
    TxType,
    FromUserId,
    ToUserId,
    Amount,
    Timestamp,
    Status,
    Description,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub field_name: FieldName,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub enum ReaderError {
    FieldParseError(ParseError),
    RecordFormatError(String),
    FileFormatError(String),
}

#[derive(Debug, PartialEq)]
pub enum WriterError {
    WriterError(String),
}
