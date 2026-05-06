mod bin;
mod csv;
mod error;
mod factory;
mod txt;

pub use bin::{BinReader, BinWriter};
pub use csv::{CsvReader, CsvWriter};
pub use error::{ReaderError, WriterError, ValidationError, ParserError};
pub use txt::{TxtReader, TxtWriter};
pub use factory::{Format, TransactionReaderFactory, TransactionWriterFactory};

use std::{fmt::Display, str::FromStr};

use crate::error::ParseError;

/// Fields of transaction record
#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
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

impl FromStr for FieldName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TX_ID" => Ok(FieldName::TxId),
            "TX_TYPE" => Ok(FieldName::TxType),
            "FROM_USER_ID" => Ok(FieldName::FromUserId),
            "TO_USER_ID" => Ok(FieldName::ToUserId),
            "AMOUNT" => Ok(FieldName::Amount),
            "TIMESTAMP" => Ok(FieldName::Timestamp),
            "STATUS" => Ok(FieldName::Status),
            "DESCRIPTION" => Ok(FieldName::Description),
            _ => Err(format!("Unknown field name: {}", s)),
        }
    }
}

impl Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FieldName::TxId => "TX_ID",
            FieldName::TxType => "TX_TYPE",
            FieldName::FromUserId => "FROM_USER_ID",
            FieldName::ToUserId => "TO_USER_ID",
            FieldName::Amount => "AMOUNT",
            FieldName::Timestamp => "TIMESTAMP",
            FieldName::Status => "STATUS",
            FieldName::Description => "DESCRIPTION",
        })
    }
}

#[derive(Debug, PartialEq, Default)]
/// Transaction
pub struct Transaction {
    /// Transaction ID
    pub tx_id: u64,
    /// Transaction type
    pub tx_type: TxType,
    /// Id of the user who sent the money
    pub from_user_id: u64,
    /// Id of the user who received the money
    pub to_user_id: u64,
    /// Amount of money
    pub amount: u64,
    /// Timestamp   
    pub timestamp: u64,
    /// Status
    pub status: Status,
    /// Description
    pub description: String,
}

impl Transaction {
    /// Transaction validation
    ///
    /// # Examples
    /// ```
    /// use parser::{Transaction, TxType};
    ///
    /// let tx = Transaction{
    ///     tx_type: TxType::TRANSFER,
    ///     from_user_id: 1,
    ///     to_user_id: 2,
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(tx.validate(), Ok(()));
    /// ```
    pub fn validate(&self) -> Result<(), error::ValidationError> {
        let is_from_user_valid = match self.tx_type {
            TxType::DEPOSIT => self.from_user_id == 0,
            _ => self.from_user_id != 0,
        };
        if !is_from_user_valid {
            return Err(ValidationError::BadFromUserId);
        }

        let is_to_user_valid = match self.tx_type {
            TxType::WITHDRAWAL => self.to_user_id == 0,
            _ => self.to_user_id != 0,
        };
        if !is_to_user_valid {
            Err(ValidationError::BadToUserId)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
/// Transaction type
pub enum TxType {
    /// Deposit
    DEPOSIT = 0,
    #[default]
    /// Transfer
    TRANSFER = 1,
    /// Withdrawal
    WITHDRAWAL = 2,
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TxType::DEPOSIT => "DEPOSIT",
            TxType::TRANSFER => "TRANSFER",
            TxType::WITHDRAWAL => "WITHDRAWAL",
        })
    }
}

impl FromStr for TxType {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TxType::DEPOSIT),
            "TRANSFER" => Ok(TxType::TRANSFER),
            "WITHDRAWAL" => Ok(TxType::WITHDRAWAL),
            _ => Err(ParseError {
                field_name: FieldName::TxType,
                value: s.to_string(),
            }),
        }
    }
}

impl TryFrom<u8> for TxType {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TxType::DEPOSIT),
            1 => Ok(TxType::TRANSFER),
            2 => Ok(TxType::WITHDRAWAL),
            _ => Err(ParseError {
                field_name: FieldName::TxType,
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
/// Status
pub enum Status {
    #[default]
    /// Success
    SUCCESS = 0,
    /// Failure
    FAILURE = 1,
    /// Pending
    PENDING = 2,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Status::SUCCESS => "SUCCESS",
            Status::FAILURE => "FAILURE",
            Status::PENDING => "PENDING",
        })
    }
}

impl FromStr for Status {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SUCCESS" => Ok(Status::SUCCESS),
            "FAILURE" => Ok(Status::FAILURE),
            "PENDING" => Ok(Status::PENDING),
            _ => Err(ParseError {
                field_name: FieldName::Status,
                value: s.to_string(),
            }),
        }
    }
}

impl TryFrom<u8> for Status {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::SUCCESS),
            1 => Ok(Status::FAILURE),
            2 => Ok(Status::PENDING),
            _ => Err(ParseError {
                field_name: FieldName::Status,
                value: value.to_string(),
            }),
        }
    }
}
/// Transaction reader trait
pub trait TransactionReader {
    #[must_use = "Reading may failed, check the result"]
    /// Reads one transaction from the reader
    fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError>;

    #[must_use = "Reading may fail, check the result"]
    /// Reads all transactions from the reader to a vector
    fn read_vector(&mut self) -> Result<Vec<Transaction>, ReaderError> {
        let mut result: Vec<Transaction> = Vec::new();

        while let Some(tx) = self.read_tx()? {
            result.push(tx);
        }

        Ok(result)
    }
}

/// Transaction writer trait
pub trait TransactionWriter {
    #[must_use = "Writing may fail, check the result"]
    /// Writes one transaction to the writer
    fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError>;

    #[must_use = "Writing may fail, check the result"]
    /// Writes all transactions from the vector to the writer
    fn write_vector(&mut self, txs: &[Transaction]) -> Result<(), WriterError> {
        for tx in txs {
            self.write_tx(tx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ValidationError;

    use super::*;
    use std::io::{Read, Write};

    impl Transaction {
        fn new(tx_id: u8) -> Self {
            Transaction {
                tx_id: tx_id as u64,
                ..Default::default()
            }
        }
    }

    struct FakeReader<R: Read> {
        data: R,
    }

    impl<R: Read> TransactionReader for FakeReader<R> {
        fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError> {
            let mut buf = [0u8; 1];

            match self.data.read(&mut buf) {
                Ok(len) => match len {
                    0 => Ok(None),
                    _ => Ok(Some(Transaction::new(buf[0]))),
                },
                Err(e) => Err(ReaderError::FileFormatError(format!(
                    "Failed to read transaction: {}",
                    e
                ))),
            }
        }
    }

    #[test]
    fn test_read_multiple_transactions() {
        let data: &[u8] = &[10, 20, 30];

        let mut reader: Box<dyn TransactionReader> = Box::new(FakeReader { data });

        let result = reader.read_vector().unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].tx_id, 10);
        assert_eq!(result[1].tx_id, 20);
        assert_eq!(result[2].tx_id, 30);
    }

    struct FakeWriter<W: Write> {
        data: W,
    }

    impl<W: Write> TransactionWriter for FakeWriter<W> {
        fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError> {
            self.data.write_all(&[tx.tx_id as u8]).map_err(|e| {
                WriterError::WriterError(format!("Failed to write transaction: {}", e))
            })
        }
    }

    #[test]
    fn test_write_multiple_transactions() {
        let txs: &Vec<Transaction> = &vec![
            Transaction::new(10),
            Transaction::new(20),
            Transaction::new(30),
        ];

        let mut output = Vec::new();
        {
            let mut writer = FakeWriter { data: &mut output };
            writer.write_vector(txs).unwrap();
        }
        assert_eq!(output, vec![10, 20, 30]);
    }

    struct ErrorReader;
    impl TransactionReader for ErrorReader {
        fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError> {
            Err(ReaderError::FileFormatError("Error".to_string()))
        }
    }

    #[test]
    fn test_read_error() {
        let mut reader = ErrorReader;
        let result = reader.read_vector();
        assert_eq!(
            result,
            Err(ReaderError::FileFormatError("Error".to_string()))
        );
    }

    struct ErrorWriter;
    impl TransactionWriter for ErrorWriter {
        fn write_tx(&mut self, _tx: &Transaction) -> Result<(), WriterError> {
            Err(WriterError::WriterError("Error".to_string()))
        }
    }
    #[test]
    fn test_write_error() {
        let mut writer = ErrorWriter;

        let txs: &Vec<Transaction> = &vec![
            Transaction::new(10),
            Transaction::new(20),
            Transaction::new(30),
        ];

        let result = writer.write_vector(txs);
        assert_eq!(result, Err(WriterError::WriterError("Error".to_string())));
    }

    #[test]
    fn test_transaction_validation() {
        let tx_depostit_from_0 = Transaction {
            tx_type: TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(tx_depostit_from_0.validate(), Ok(())) {
            panic!("Deposit From 0 shoud be valid");
        }

        let tx_depostit_from_1 = Transaction {
            tx_type: TxType::DEPOSIT,
            from_user_id: 1,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(
            tx_depostit_from_1.validate(),
            Err(ValidationError::BadFromUserId)
        ) {
            panic!("Deposit From 1 should return BadFromUserId");
        }

        let tx_transfer_from_0 = Transaction {
            tx_type: TxType::TRANSFER,
            from_user_id: 0,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(
            tx_transfer_from_0.validate(),
            Err(ValidationError::BadFromUserId)
        ) {
            panic!("Transfer From 0 shoud return BadFromUserId");
        }

        let tx_transfer_from_1 = Transaction {
            tx_type: TxType::TRANSFER,
            from_user_id: 1,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(tx_transfer_from_1.validate(), Ok(())) {
            panic!("Transfer From 1 shoud be valid");
        }

        let tx_withdrawal_from_0 = Transaction {
            tx_type: TxType::WITHDRAWAL,
            from_user_id: 0,
            to_user_id: 0,
            ..Default::default()
        };
        if !matches!(
            tx_withdrawal_from_0.validate(),
            Err(ValidationError::BadFromUserId)
        ) {
            panic!("Withdrawal From 0 shoud return BadFromUserId");
        }

        let tx_withdrawal_from_1 = Transaction {
            tx_type: TxType::WITHDRAWAL,
            from_user_id: 1,
            to_user_id: 0,
            ..Default::default()
        };
        if !matches!(tx_withdrawal_from_1.validate(), Ok(())) {
            panic!("Withdrawal From 1 shoud be valid");
        }

        let tx_deposit_to_0 = Transaction {
            tx_type: TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 0,
            ..Default::default()
        };
        if !matches!(
            tx_deposit_to_0.validate(),
            Err(ValidationError::BadToUserId)
        ) {
            panic!("Deposit To 0 shoud return BadToUserId");
        }

        let tx_deposit_to_1 = Transaction {
            tx_type: TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(tx_deposit_to_1.validate(), Ok(())) {
            panic!("Deposit To 1 shoud be valid");
        }

        let tx_transfer_to_0 = Transaction {
            tx_type: TxType::TRANSFER,
            from_user_id: 1,
            to_user_id: 0,
            ..Default::default()
        };
        if !matches!(
            tx_transfer_to_0.validate(),
            Err(ValidationError::BadToUserId)
        ) {
            panic!("Transfer To 0 shoud return BadToUserId");
        }

        let tx_transfer_to_1 = Transaction {
            tx_type: TxType::TRANSFER,
            from_user_id: 1,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(tx_transfer_to_1.validate(), Ok(())) {
            panic!("Transfer To 1 shoud be valid");
        }

        let tx_withdrawal_to_0 = Transaction {
            tx_type: TxType::WITHDRAWAL,
            from_user_id: 1,
            to_user_id: 0,
            ..Default::default()
        };
        if !matches!(tx_withdrawal_to_0.validate(), Ok(())) {
            panic!("Withdrawal To 0 shoud be valid");
        }

        let tx_withdrawal_to_1 = Transaction {
            tx_type: TxType::WITHDRAWAL,
            from_user_id: 1,
            to_user_id: 1,
            ..Default::default()
        };
        if !matches!(
            tx_withdrawal_to_1.validate(),
            Err(ValidationError::BadToUserId)
        ) {
            panic!("Withdrawal To 1 shoud return BadToUserId");
        }
    }
}
