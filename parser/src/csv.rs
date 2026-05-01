use std::io::{Read, Write};

use csv::StringRecord;

use crate::{
    Transaction, TransactionReader, TransactionWriter,
    error::{FieldName, ParseError, ReaderError, WriterError},
};

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
    /// let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
    /// 1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n
    /// 1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,\"Record number 2\"\n
    /// 1000000000000002,WITHDRAWAL,599094029349995112,0,300,1633036980000,SUCCESS,\"Record number 3\"\n";
    ///
    ///
    /// let cursor = Cursor::new(csv_data);
    /// let mut reader = CsvReader::try_new(cursor).unwrap();
    /// let txs = reader.read_vector().unwrap();
    ///
    /// assert_eq!(txs.len(), 3);
    /// assert_eq!(txs[0],Transaction {
    ///     tx_id: 1000000000000000,
    ///     tx_type: TxType::DEPOSIT,
    ///     from_user_id: 0,
    ///     to_user_id: 9223372036854775807,
    ///     amount: 100,
    ///     timestamp: 1633036860000,
    ///     status: Status::FAILURE,
    ///     description: "Record number 1".to_string(),
    /// });
    /// assert_eq!(txs[1], Transaction {
    ///    tx_id: 1000000000000001,
    ///        tx_type: TxType::TRANSFER,
    ///        from_user_id: 9223372036854775807,
    ///        to_user_id: 9223372036854775807,
    ///        amount: 200,
    ///        timestamp: 1633036920000,
    ///        status: Status::PENDING,
    ///        description: "Record number 2".to_string(),
    ///    });
    /// assert_eq!(txs[2], Transaction {
    ///        tx_id: 1000000000000002,
    ///        tx_type: TxType::WITHDRAWAL,
    ///        from_user_id: 599094029349995112,
    ///        to_user_id: 0,
    ///        amount: 300,
    ///        timestamp: 1633036980000,
    ///        status: Status::SUCCESS,
    ///        description: "Record number 3".to_string(),
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
            Ok(Some(Transaction {
                tx_id: rec[0].parse().map_err(|_| {
                    ReaderError::FieldParseError(ParseError {
                        field_name: FieldName::TxId,
                        value: rec[0].to_string(),
                    })
                })?,

                tx_type: rec[1]
                    .parse()
                    .map_err(|e| ReaderError::FieldParseError(e))?,

                from_user_id: rec[2].parse().map_err(|_| {
                    ReaderError::FieldParseError(ParseError {
                        field_name: FieldName::FromUserId,
                        value: rec[2].to_string(),
                    })
                })?,

                to_user_id: rec[3].parse().map_err(|_| {
                    ReaderError::FieldParseError(ParseError {
                        field_name: FieldName::ToUserId,
                        value: rec[3].to_string(),
                    })
                })?,

                amount: rec[4].parse().map_err(|_| {
                    ReaderError::FieldParseError(ParseError {
                        field_name: FieldName::Amount,
                        value: rec[4].to_string(),
                    })
                })?,

                timestamp: rec[5].parse().map_err(|_| {
                    ReaderError::FieldParseError(ParseError {
                        field_name: FieldName::Timestamp,
                        value: rec[5].to_string(),
                    })
                })?,

                status: rec[6]
                    .parse()
                    .map_err(|e| ReaderError::FieldParseError(e))?,

                description: rec[7].to_string(),
            }))
        }
    }
}

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
    /// Transaction {
    ///     tx_id: 1000000000000000,
    ///     tx_type: TxType::DEPOSIT,
    ///     from_user_id: 0,
    ///     to_user_id: 9223372036854775807,
    ///     amount: 100,
    ///     timestamp: 1633036860000,
    ///     status: Status::FAILURE,
    ///     description: "Record number 1".to_string(),
    ///   }, Transaction {
    ///    tx_id: 1000000000000001,
    ///        tx_type: TxType::TRANSFER,
    ///        from_user_id: 9223372036854775807,
    ///        to_user_id: 9223372036854775807,
    ///        amount: 200,
    ///        timestamp: 1633036920000,
    ///        status: Status::PENDING,
    ///        description: "Record number 2".to_string(),
    ///    }, Transaction {
    ///        tx_id: 1000000000000002,
    ///        tx_type: TxType::WITHDRAWAL,
    ///        from_user_id: 599094029349995112,
    ///        to_user_id: 0,
    ///        amount: 300,
    ///        timestamp: 1633036980000,
    ///        status: Status::SUCCESS,
    ///        description: "Record number 3".to_string(),
    ///    }];
    ///    let mut data: Vec<u8> = Vec::new();
    ///
    ///    {
    ///        let mut writer = CsvWriter::try_new(&mut data).unwrap();
    ///
    ///        writer.write_vector(&transactions).unwrap();
    ///    }
    /// let expected_csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
    /// 1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"
    /// 1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,\"Record number 2\"
    /// 1000000000000002,WITHDRAWAL,599094029349995112,0,300,1633036980000,SUCCESS,\"Record number 3\"\n";
    ///
    ///    let csv_data = String::from_utf8(data).unwrap();
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
}

impl<W: Write> Drop for CsvWriter<W> {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const CSV_DATA: &str = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,"Record number 2"
1000000000000002,WITHDRAWAL,599094029349995112,0,300,1633036980000,SUCCESS,"Record number 3"
"#;

    fn tx1_data() -> Transaction {
        Transaction {
            tx_id: 1000000000000000,
            tx_type: crate::TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 9223372036854775807,
            amount: 100,
            timestamp: 1633036860000,
            status: crate::Status::FAILURE,
            description: "Record number 1".to_string(),
        }
    }

    fn tx2_data() -> Transaction {
        Transaction {
            tx_id: 1000000000000001,
            tx_type: crate::TxType::TRANSFER,
            from_user_id: 9223372036854775807,
            to_user_id: 9223372036854775807,
            amount: 200,
            timestamp: 1633036920000,
            status: crate::Status::PENDING,
            description: "Record number 2".to_string(),
        }
    }

    fn tx3_data() -> Transaction {
        Transaction {
            tx_id: 1000000000000002,
            tx_type: crate::TxType::WITHDRAWAL,
            from_user_id: 599094029349995112,
            to_user_id: 0,
            amount: 300,
            timestamp: 1633036980000,
            status: crate::Status::SUCCESS,
            description: "Record number 3".to_string(),
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
    fn test_read_csv_wrong_header() -> Result<(), String> {
        let csv_data = "Invalid CSV header";
        let cursor = Cursor::new(csv_data.to_string());
        let wrong_header = CsvReader::try_new(cursor);

        match wrong_header {
            Err(_) => Ok(()),
            Ok(_) => Err("Should return an error".to_string()),
        }
    }

    #[test]
    fn test_read_csv_wrong_records() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
WRONG,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000000,WRONG,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000000,DEPOSIT,WRONG,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000000,DEPOSIT,0,WRONG,100,1633036860000,FAILURE,"Record number 1"
1000000000000000,DEPOSIT,0,9223372036854775807,WRONG,1633036860000,FAILURE,"Record number 1"
1000000000000000,DEPOSIT,0,9223372036854775807,100,WRONG,FAILURE,"Record number 1"
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,WRONG,"Record number 1""#;

        let cursor = Cursor::new(csv_data.to_string());
        let mut reader = CsvReader::try_new(cursor).unwrap();

        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::TxId,
                ..
            }))
        ));
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::TxType,
                ..
            }))
        ));
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::FromUserId,
                ..
            }))
        ));
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::ToUserId,
                ..
            }))
        ));
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::Amount,
                ..
            }))
        ));
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::Timestamp,
                ..
            }))
        ));
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::Status,
                ..
            }))
        ));
    }

    #[test]
    fn test_read_csv_error() -> Result<(), String> {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE"#;

        let cursor = Cursor::new(csv_data.to_string());
        let mut reader = CsvReader::try_new(cursor).unwrap();

        let result = reader.read_tx();

        if let Err(ReaderError::RecordFormatError(_)) = result {
            Ok(())
        } else {
            Err("Should return RecordFromat".to_string())
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
}
