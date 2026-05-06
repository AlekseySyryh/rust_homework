use crate::FieldName;

#[derive(Debug, PartialEq)]
/// Transaction validation errors
pub enum ValidationError {
    /// From user id must be 0 if transaction type is DEPOSIT and must be non-zero otherwise
    BadFromUserId,
    /// To user id must be 0 if transaction type is WITHDRAWAL and must be non-zero otherwise
    BadToUserId,
}

#[derive(Debug, PartialEq)]
/// Errors that can occur during parsing field values
pub struct ParseError {
    /// Name of the field that failed to parse
    pub field_name: FieldName,
    /// Value failed to parse
    pub value: String,
}

#[derive(Debug, PartialEq)]
/// Errors that cat occur during reading file
pub enum ReaderError {
    /// Field error
    FieldParseError(ParseError),
    /// Record error
    RecordFormatError(String),
    /// File error
    FileFormatError(String),
    /// Record validation error
    RecordValidationError(ValidationError),
}

#[derive(Debug, PartialEq)]
/// Errors that cat occur during write file
pub enum WriterError {
    /// Writer error
    WriterError(String),
    /// Record validation error
    RecordValidationError(ValidationError),
}

#[derive(Debug)]
/// Parser error
pub enum ParserError {
    /// Reader error
    ReaderError(ReaderError),
    /// Writer error
    WriterError(WriterError),
}