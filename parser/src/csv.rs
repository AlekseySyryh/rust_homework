use std::{
    io::{Read, Write},
    str::FromStr,
};

use csv::StringRecord;

use crate::{
    FieldName, Transaction, TransactionReader, TransactionWriter,
    error::{ParseError, ReaderError, WriterError},
};
/// CSV reader
pub struct CsvReader<R: Read> {
    reader: csv::Reader<R>,
}

impl<R: Read> CsvReader<R> {
    /// Creates a new CSV reader.
    ///
    /// The CSV file must have the following header:
    /// `TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION`.
    ///
    /// # Errors
    ///
    /// Returns `ReaderError::FileFormatError` if the header is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use parser::{CsvReader, TransactionReader, Transaction, TxType, Status};
    ///
    /// let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
    ///1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"
    ///1002,TRANSFER,501,502,15000,1672534800000,FAILURE,\"Payment for services, invoice #123\"
    ///1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,\"ATM withdrawal\"\n";
    ///
    ///
    /// let cursor = Cursor::new(csv_data);
    /// let mut reader = CsvReader::try_new(cursor).unwrap();
    /// let txs = reader.read_vector().unwrap();
    ///
    /// assert_eq!(txs.len(), 3);
    /// assert_eq!(txs[0],Transaction {
    ///        tx_id: 1001,
    ///        tx_type: TxType::DEPOSIT,
    ///        from_user_id: 0,
    ///        to_user_id: 501,
    ///        amount: 50000,
    ///        timestamp: 1672531200000,
    ///        status: Status::SUCCESS,
    ///        description: "Initial account funding".to_string(),
    /// });
    /// assert_eq!(txs[1], Transaction {
    ///        tx_id: 1002,
    ///        tx_type: TxType::TRANSFER,
    ///        from_user_id: 501,
    ///        to_user_id: 502,
    ///        amount: 15000,
    ///        timestamp: 1672534800000,
    ///        status: Status::FAILURE,
    ///        description: "Payment for services, invoice #123".to_string(),
    ///    });
    /// assert_eq!(txs[2], Transaction {
    ///        tx_id: 1003,
    ///        tx_type: TxType::WITHDRAWAL,
    ///        from_user_id: 502,
    ///        to_user_id: 0,
    ///        amount: 1000,
    ///        timestamp: 1672538400000,
    ///        status: Status::PENDING,
    ///        description: "ATM withdrawal".to_string(),
    ///    });
    /// ```
    pub fn try_new(reader: R) -> Result<Self, ReaderError> {
        let mut csv_reader = csv::Reader::from_reader(reader);

        let headers = csv_reader
            .headers()
            .map_err(|e| ReaderError::FileFormatError(e.to_string()))?;

        let expected_headers = vec![
            "TX_ID",
            "TX_TYPE",
            "FROM_USER_ID",
            "TO_USER_ID",
            "AMOUNT",
            "TIMESTAMP",
            "STATUS",
            "DESCRIPTION",
        ];

        if headers.iter().ne(expected_headers) {
            return Err(ReaderError::FileFormatError(format!("Invalid CSV header")));
        }

        Ok(CsvReader { reader: csv_reader })
    }

    fn parse<T: FromStr>(&self, s: &str, field_name: FieldName) -> Result<T, ReaderError> {
        s.parse::<T>().map_err(|_| {
            ReaderError::FieldParseError(ParseError {
                field_name,
                value: s.to_string(),
            })
        })
    }
}

impl<R: Read> TransactionReader for CsvReader<R> {
    fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError> {
        let mut rec = StringRecord::new();
        let result = self
            .reader
            .read_record(&mut rec)
            .map_err(|err| ReaderError::RecordFormatError(err.to_string()))?;

        if !result {
            Ok(None)
        } else if rec.len() != 8 {
            Err(ReaderError::RecordFormatError(
                "Wrong record length".to_string(),
            ))
        } else {
            let tx = Transaction {
                tx_id: self.parse(&rec[0], FieldName::TxId)?,
                tx_type: self.parse(&rec[1], FieldName::TxType)?,
                from_user_id: self.parse(&rec[2], FieldName::FromUserId)?,
                to_user_id: self.parse(&rec[3], FieldName::ToUserId)?,
                amount: self.parse(&rec[4], FieldName::Amount)?,
                timestamp: self.parse(&rec[5], FieldName::Timestamp)?,
                status: self.parse(&rec[6], FieldName::Status)?,
                description: rec[7].to_string(),
            };
            match tx.validate() {
                Ok(_) => Ok(Some(tx)),
                Err(e) => Err(ReaderError::RecordValidationError(e)),
            }
        }
    }
}

/// CSV writer
pub struct CsvWriter<W: Write> {
    writer: csv::Writer<W>,
}

impl<W: Write> CsvWriter<W> {
    /// Creates a new CSV writer.
    ///
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use parser::{CsvWriter, TransactionWriter, Transaction, TxType, Status};
    ///
    /// let transactions = vec![
    ///     Transaction {
    ///        tx_id: 1001,
    ///        tx_type: TxType::DEPOSIT,
    ///        from_user_id: 0,
    ///        to_user_id: 501,
    ///        amount: 50000,
    ///        timestamp: 1672531200000,
    ///        status: Status::SUCCESS,
    ///        description: "Initial account funding".to_string(),
    ///     },
    ///     Transaction {
    ///        tx_id: 1002,
    ///        tx_type: TxType::TRANSFER,
    ///        from_user_id: 501,
    ///        to_user_id: 502,
    ///        amount: 15000,
    ///        timestamp: 1672534800000,
    ///        status: Status::FAILURE,
    ///        description: "Payment for services, invoice #123".to_string(),
    ///     },
    ///     Transaction {
    ///        tx_id: 1003,
    ///        tx_type: TxType::WITHDRAWAL,
    ///        from_user_id: 502,
    ///        to_user_id: 0,
    ///        amount: 1000,
    ///        timestamp: 1672538400000,
    ///        status: Status::PENDING,
    ///        description: "ATM withdrawal".to_string(),
    ///    }];
    ///    let mut data: Vec<u8> = Vec::new();
    ///
    ///    {
    ///        let mut writer = CsvWriter::try_new(&mut data).unwrap();
    ///
    ///        writer.write_vector(&transactions).unwrap();
    ///    }
    ///
    ///    let expected_csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
    ///1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"
    ///1002,TRANSFER,501,502,15000,1672534800000,FAILURE,\"Payment for services, invoice #123\"
    ///1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,\"ATM withdrawal\"\n";
    ///
    ///    let csv_data = String::from_utf8(data).unwrap();
    ///
    ///    assert_eq!(csv_data, expected_csv_data);
    /// ```
    pub fn try_new(writer: W) -> Result<Self, WriterError> {
        let mut csv_writer = csv::WriterBuilder::new()
            .quote_style(csv::QuoteStyle::Never)
            .from_writer(writer);

        csv_writer
            .write_record([
                "TX_ID",
                "TX_TYPE",
                "FROM_USER_ID",
                "TO_USER_ID",
                "AMOUNT",
                "TIMESTAMP",
                "STATUS",
                "DESCRIPTION",
            ])
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;

        Ok(CsvWriter { writer: csv_writer })
    }
}

impl<W: Write> TransactionWriter for CsvWriter<W> {
    fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError> {
        match tx.validate() {
            Ok(_) => {
                self.writer
                    .write_record([
                        &tx.tx_id.to_string(),
                        &tx.tx_type.to_string(),
                        &tx.from_user_id.to_string(),
                        &tx.to_user_id.to_string(),
                        &tx.amount.to_string(),
                        &tx.timestamp.to_string(),
                        &tx.status.to_string(),
                        &format!("\"{}\"", tx.description.replace("\"", "\"\"")),
                    ])
                    .map_err(|e| WriterError::WriterError(format!("{}", e)))?;

                Ok(())
            }
            Err(error) => Err(WriterError::RecordValidationError(error)),
        }
    }
}

impl<W: Write> Drop for CsvWriter<W> {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::error::ValidationError;

    use super::*;

    const CSV_DATA: &str = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,"ATM withdrawal"
"#;

    fn tx1_data() -> Transaction {
        Transaction {
            tx_id: 1001,
            tx_type: crate::TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 501,
            amount: 50000,
            timestamp: 1672531200000,
            status: crate::Status::SUCCESS,
            description: "Initial account funding".to_string(),
        }
    }

    fn tx2_data() -> Transaction {
        Transaction {
            tx_id: 1002,
            tx_type: crate::TxType::TRANSFER,
            from_user_id: 501,
            to_user_id: 502,
            amount: 15000,
            timestamp: 1672534800000,
            status: crate::Status::FAILURE,
            description: "Payment for services, invoice #123".to_string(),
        }
    }

    fn tx3_data() -> Transaction {
        Transaction {
            tx_id: 1003,
            tx_type: crate::TxType::WITHDRAWAL,
            from_user_id: 502,
            to_user_id: 0,
            amount: 1000,
            timestamp: 1672538400000,
            status: crate::Status::PENDING,
            description: "ATM withdrawal".to_string(),
        }
    }

    #[test]
    fn test_read_csv() {
        let cursor = Cursor::new(CSV_DATA.as_bytes());

        let mut reader = CsvReader::try_new(cursor).unwrap();

        let tx1 = reader.read_tx();

        assert_eq!(tx1, Ok(Some(tx1_data())));

        let tx2 = reader.read_tx();

        assert_eq!(tx2, Ok(Some(tx2_data())));

        let tx3 = reader.read_tx();
        assert_eq!(tx3, Ok(Some(tx3_data())));

        let tx4 = reader.read_tx();
        assert_eq!(tx4, Ok(None));
    }

    #[test]
    fn test_read_csv_wrong_header() {
        let csv_data = "Invalid CSV header";
        let cursor = Cursor::new(csv_data.to_string());
        let wrong_header = CsvReader::try_new(cursor);

        if let Err(ReaderError::FileFormatError(_)) = wrong_header {
        } else {
            panic!("Should return an error");
        }
    }

    #[test]
    fn test_read_csv_wrong_records() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
WRONG,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,WRONG,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,WRONG,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,0,WRONG,50000,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,0,501,WRONG,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,0,501,50000,WRONG,SUCCESS,"Initial account funding"
1001,DEPOSIT,0,501,50000,1672531200000,WRONG,"Initial account funding"
"#;

        let cursor = Cursor::new(csv_data.to_string());
        let mut reader = CsvReader::try_new(cursor).unwrap();

        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::TxId,
                    ..
                }))
            ),
            "Parse wrong TxId shoud return error"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::TxType,
                    ..
                }))
            ),
            "Parse wrong TxType shoud return error"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::FromUserId,
                    ..
                }))
            ),
            "Parse wrong FromUserId shoud return error"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::ToUserId,
                    ..
                }))
            ),
            "Parse wrong ToUserId shoud return error"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::Amount,
                    ..
                }))
            ),
            "Parse wrong Amount shoud return error"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::Timestamp,
                    ..
                }))
            ),
            "Parse wrong Timestamp shoud return error"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::Status,
                    ..
                }))
            ),
            "Parse wrong Status shoud return error"
        );
    }

    #[test]
    fn test_read_csv_error() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
WRONG,DEPOSIT,0,501,50000,1672531200000,SUCCESS"#;

        let cursor = Cursor::new(csv_data.to_string());
        let mut reader = CsvReader::try_new(cursor).unwrap();

        let result = reader.read_tx();

        if let Err(ReaderError::RecordFormatError(_)) = result {
        } else {
            panic!("Should return RecordFromat");
        }
    }

    #[test]
    fn test_read_csv_validates_transactions() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,0,50000,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,500,0,50000,1672531200000,SUCCESS,"Initial account funding"
1001,DEPOSIT,500,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,TRANSFER,0,0,50000,1672531200000,SUCCESS,"Initial account funding"
1001,TRANSFER,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,TRANSFER,500,0,50000,1672531200000,SUCCESS,"Initial account funding"
1001,TRANSFER,500,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,WITHDRAWAL,0,0,50000,1672531200000,SUCCESS,"Initial account funding"
1001,WITHDRAWAL,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1001,WITHDRAWAL,500,0,50000,1672531200000,SUCCESS,"Initial account funding"
1001,WITHDRAWAL,500,501,50000,1672531200000,SUCCESS,"Initial account funding"
"#;
        let cursor = Cursor::new(csv_data.to_string());
        let mut reader = CsvReader::try_new(cursor).unwrap();

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadToUserId
            ))
        ) {
            panic!("Deposit 0 0 should return BadToUserId");
        }

        if !matches!(reader.read_tx(), Ok(_)) {
            panic!("Deposit 0 501 should be valid");
        }

        if !matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))) {
            panic!("Deposit 500 0 should return BadFromUserId or BadToUserId");
        }

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadFromUserId
            ))
        ) {
            panic!("Deposit 500 501 should return BadFromUserId");
        }

        if !matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))) {
            panic!("Transfer 0 0 should return BadFromUserId or BadToUserId");
        }

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadFromUserId
            ))
        ) {
            panic!("Transfer 0 501 should return BadFromUserId");
        }

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadToUserId
            ))
        ) {
            panic!("Transfer 500 0 should return BadToUserId");
        }

        if !matches!(reader.read_tx(), Ok(_)) {
            panic!("Transfer 500 501 should be valid");
        }

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadFromUserId
            ))
        ) {
            panic!("Withdrawal 0 0 should return BadFromUserId");
        }

        if !matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))) {
            panic!("Withdrawal 0 501 should return BadFromUserId or BadToUserId");
        }

        if !matches!(reader.read_tx(), Ok(_)) {
            panic!("Withdrawal 500 0 should be valid");
        }

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadToUserId
            ))
        ) {
            panic!("Withdrawal 500 501 should return BadToUserId");
        }
    }

    #[test]
    fn test_write_csv() {
        let mut data: Vec<u8> = Vec::new();

        {
            let mut writer = CsvWriter::try_new(&mut data).unwrap();

            writer.write_tx(&tx1_data()).unwrap();
            writer.write_tx(&tx2_data()).unwrap();
            writer.write_tx(&tx3_data()).unwrap();
        }
        let csv_data = String::from_utf8(data).expect("Found invalid UTF-8");
        assert_eq!(csv_data, CSV_DATA);
    }

    enum ExpectedBehavior {
        Valid,
        AnyError,
        Error(ValidationError),
    }

    #[test]
    fn test_write_csv_validates_transactions() {
        let test_cases = vec![
            (
                Transaction {
                    tx_type: crate::TxType::DEPOSIT,
                    from_user_id: 0,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadToUserId),
                "Deposit 0 0 should return BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::DEPOSIT,
                    from_user_id: 0,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Valid,
                "Deposit 0 501 should be valid".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::DEPOSIT,
                    from_user_id: 500,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::AnyError,
                "Deposit 500 0 should return BadFromUserId or BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::DEPOSIT,
                    from_user_id: 500,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadFromUserId),
                "Deposit 500 501 should return BadFromUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::TRANSFER,
                    from_user_id: 0,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::AnyError,
                "Transfer 0 0 should return BadFromUserId or BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::TRANSFER,
                    from_user_id: 0,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadFromUserId),
                "Transfer 0 501 should return BadFromUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::TRANSFER,
                    from_user_id: 500,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadToUserId),
                "Transfer 500 0 should return BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::TRANSFER,
                    from_user_id: 500,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Valid,
                "Transfer 500 501 should be valid".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::WITHDRAWAL,
                    from_user_id: 0,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadFromUserId),
                "Withdrawal 0 0 should return BadFromUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::WITHDRAWAL,
                    from_user_id: 0,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::AnyError,
                "Withdrawal 0 501 should return BadFromUserId or BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::WITHDRAWAL,
                    from_user_id: 500,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Valid,
                "Withdrawal 500 0 should be valid".to_string(),
            ),
            (
                Transaction {
                    tx_type: crate::TxType::WITHDRAWAL,
                    from_user_id: 500,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadToUserId),
                "Withdrawal 500 501 should return BadToUserId".to_string(),
            ),
        ];

        let mut data: Vec<u8> = Vec::new();

        let mut writer = CsvWriter::try_new(&mut data).unwrap();

        for (tx, expected_behavior, error) in test_cases {
            let result = writer.write_tx(&tx);
            if !match expected_behavior {
                ExpectedBehavior::Valid => matches!(result, Ok(_)),
                ExpectedBehavior::AnyError => {
                    matches!(result, Err(WriterError::RecordValidationError(_)))
                }
                ExpectedBehavior::Error(expected_error) => {
                    if let Err(WriterError::RecordValidationError(actual_error)) = &result {
                        actual_error == &expected_error
                    } else {
                        false
                    }
                }
            } {
                panic!("{error}");
            }
        }
    }
}
