use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

use crate::{
    FieldName, ReaderError, Transaction, TransactionReader, TransactionWriter, WriterError,
    error::ParseError,
};

/// TXT reader
pub struct TxtReader<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> TxtReader<R> {
    /// Creates a new TXT reader.
    ///
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use parser::{TxtReader, TransactionReader, Transaction, TxType, Status};
    ///
    /// let txt_data = "TX_ID: 1234567890123456
    ///TX_TYPE: DEPOSIT
    ///FROM_USER_ID: 0
    ///TO_USER_ID: 9876543210987654
    ///AMOUNT: 10000
    ///TIMESTAMP: 1633036800000
    ///STATUS: SUCCESS
    ///DESCRIPTION: \"Terminal deposit\"
    ///
    ///TX_ID: 2312321321321321
    ///TIMESTAMP: 1633056800000
    ///STATUS: FAILURE
    ///TX_TYPE: TRANSFER
    ///FROM_USER_ID: 1231231231231231
    ///TO_USER_ID: 9876543210987654
    ///AMOUNT: 1000
    ///DESCRIPTION: \"User transfer\"
    ///
    ///TX_ID: 3213213213213213
    ///AMOUNT: 100
    ///TX_TYPE: WITHDRAWAL
    ///FROM_USER_ID: 9876543210987654
    ///TO_USER_ID: 0
    ///TIMESTAMP: 1633066800000
    ///STATUS: SUCCESS
    ///DESCRIPTION: \"User withdrawal\"".to_string();
    ///
    /// let cursor = Cursor::new(txt_data);
    /// let mut reader = TxtReader::new(cursor);
    ///
    /// let txs = reader.read_vector().unwrap();
    ///
    /// assert_eq!(txs.len(), 3);
    /// assert_eq!(txs[0], Transaction {
    ///     tx_id: 1234567890123456,
    ///     tx_type: TxType::DEPOSIT,
    ///     from_user_id: 0,
    ///     to_user_id: 9876543210987654,
    ///     amount: 10000,
    ///     timestamp: 1633036800000,
    ///     status: Status::SUCCESS,
    ///     description: "Terminal deposit".to_string(),
    /// });
    /// assert_eq!(txs[1], Transaction {
    ///     tx_id: 2312321321321321,
    ///     tx_type: TxType::TRANSFER,
    ///     from_user_id: 1231231231231231,
    ///     to_user_id: 9876543210987654,
    ///     amount: 1000,
    ///     timestamp: 1633056800000,
    ///     status: Status::FAILURE,
    ///     description: "User transfer".to_string(),
    /// });
    /// assert_eq!(txs[2], Transaction {
    ///     tx_id: 3213213213213213,
    ///     tx_type: TxType::WITHDRAWAL,
    ///     from_user_id: 9876543210987654,
    ///     to_user_id: 0,
    ///     amount: 100,
    ///     timestamp: 1633066800000,
    ///     status: Status::SUCCESS,
    ///     description: "User withdrawal".to_string(),
    /// });
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
        }
    }

    fn get_value<'a>(
        &self,
        fields: &'a HashMap<FieldName, String>,
        field_name: FieldName,
    ) -> Result<&'a String, ReaderError> {
        fields
            .get(&field_name)
            .ok_or_else(|| ReaderError::RecordFormatError(format!("{} is required", field_name)))
    }

    fn parse<T: FromStr>(
        &self,
        fields: &HashMap<FieldName, String>,
        field_name: FieldName,
    ) -> Result<T, ReaderError> {
        let value = self.get_value(fields, field_name)?;

        value.parse::<T>().map_err(|_| {
            ReaderError::FieldParseError(ParseError {
                field_name: field_name,
                value: value.clone(),
            })
        })
    }
}

impl<R: Read> TransactionReader for TxtReader<R> {
    fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError> {
        let mut fields: HashMap<FieldName, String> = HashMap::new();
        let mut line = String::new();
        let mut eof = false;
        loop {
            line.clear();
            let result = self.reader.read_line(&mut line);
            if let Err(err) = result {
                return Err(ReaderError::FileFormatError(err.to_string()));
            }
            if let Ok(0) = result {
                eof = true;
                break;
            }
            if line.starts_with('#') {
                continue;
            }
            match line.trim().split_once(':') {
                Some((key, value)) => {
                    if let Ok(field_name) = key.parse() {
                        let old_value = fields.insert(field_name, value.trim().to_string());
                        if old_value.is_some() {
                            return Err(ReaderError::RecordFormatError(format!(
                                "{} is duplicated",
                                key
                            )));
                        }
                    } else {
                        return Err(ReaderError::RecordFormatError(format!(
                            "Invalid field name: {}",
                            key
                        )));
                    }
                }
                None => {
                    break;
                }
            }
        }

        if eof && fields.len() == 0 {
            return Ok(None);
        }
        let tx = Transaction {
            tx_id: self.parse(&fields, FieldName::TxId)?,
            tx_type: self.parse(&fields, FieldName::TxType)?,
            from_user_id: self.parse(&fields, FieldName::FromUserId)?,
            to_user_id: self.parse(&fields, FieldName::ToUserId)?,
            amount: self.parse(&fields, FieldName::Amount)?,
            timestamp: self.parse(&fields, FieldName::Timestamp)?,
            status: self.parse(&fields, FieldName::Status)?,
            description: self
                .get_value(&fields, FieldName::Description)?
                .trim()
                .trim_matches('"')
                .to_string(),
        };
        match tx.validate() {
            Ok(_) => Ok(Some(tx)),
            Err(err) => Err(ReaderError::RecordValidationError(err)),
        }
    }
}

pub struct TxtWriter<W: Write> {
    writer: W,
    first: bool,
}

impl<W: Write> TxtWriter<W> {
    /// Creates a new TxtWriter instance.
    ///
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use parser::{TxtWriter, TransactionWriter, Transaction, TxType, Status};
    ///     
    /// let transactions = vec![
    ///     Transaction {
    ///        tx_id: 1234567890123456,
    ///        tx_type: TxType::DEPOSIT,
    ///        from_user_id: 0,
    ///        to_user_id: 9876543210987654,
    ///        amount: 10000,
    ///        timestamp: 1633036800000,
    ///        status: Status::SUCCESS,
    ///        description: "Terminal deposit".to_string(),
    ///     },
    ///     Transaction {
    ///        tx_id: 2312321321321321,
    ///        tx_type: TxType::TRANSFER,
    ///        from_user_id: 1231231231231231,
    ///        to_user_id: 9876543210987654,
    ///        amount: 1000,
    ///        timestamp: 1633056800000,
    ///        status: Status::FAILURE,
    ///        description: "User transfer".to_string(),
    ///     },
    ///     Transaction {
    ///        tx_id: 3213213213213213,
    ///        tx_type: TxType::WITHDRAWAL,
    ///        from_user_id: 9876543210987654,
    ///        to_user_id: 0,
    ///        amount: 100,
    ///        timestamp: 1633066800000,
    ///        status: Status::SUCCESS,
    ///       description: "User withdrawal".to_string(),
    ///   }
    /// ];
    ///
    /// let mut data: Vec<u8> = Vec::new();
    /// {
    ///     let mut writer = TxtWriter::new(&mut data);
    ///     writer.write_vector(&transactions).unwrap();
    /// }
    ///
    /// let expected_txt_data = "TX_ID: 1234567890123456
    ///TX_TYPE: DEPOSIT
    ///FROM_USER_ID: 0
    ///TO_USER_ID: 9876543210987654
    ///AMOUNT: 10000
    ///TIMESTAMP: 1633036800000
    ///STATUS: SUCCESS
    ///DESCRIPTION: \"Terminal deposit\"
    ///
    ///TX_ID: 2312321321321321
    ///TX_TYPE: TRANSFER
    ///FROM_USER_ID: 1231231231231231
    ///TO_USER_ID: 9876543210987654
    ///AMOUNT: 1000
    ///TIMESTAMP: 1633056800000
    ///STATUS: FAILURE
    ///DESCRIPTION: \"User transfer\"
    ///
    ///TX_ID: 3213213213213213
    ///TX_TYPE: WITHDRAWAL
    ///FROM_USER_ID: 9876543210987654
    ///TO_USER_ID: 0
    ///AMOUNT: 100
    ///TIMESTAMP: 1633066800000
    ///STATUS: SUCCESS
    ///DESCRIPTION: \"User withdrawal\"\n";
    ///
    ///    let txt_data = String::from_utf8(data).unwrap();
    ///
    ///    assert_eq!(txt_data, expected_txt_data);
    /// ```
    pub fn new(writer: W) -> Self {
        Self {
            writer: writer,
            first: true,
        }
    }
}

impl<W: Write> TransactionWriter for TxtWriter<W> {
    fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError> {
        if let Err(err) = tx.validate() {
            return Err(WriterError::RecordValidationError(err));
        }

        if self.first {
            self.first = false;
        } else {
            writeln!(self.writer).map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        }
        writeln!(self.writer, "TX_ID: {}", tx.tx_id)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "TX_TYPE: {}", tx.tx_type)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "FROM_USER_ID: {}", tx.from_user_id)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "TO_USER_ID: {}", tx.to_user_id)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "AMOUNT: {}", tx.amount)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "TIMESTAMP: {}", tx.timestamp)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "STATUS: {}", tx.status)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        writeln!(self.writer, "DESCRIPTION: \"{}\"", tx.description)
            .map_err(|e| WriterError::WriterError(format!("{}", e)))?;
        Ok(())
    }
}

impl<W: Write> Drop for TxtWriter<W> {
    fn drop(&mut self) {
        self.writer.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{Status, Transaction, TxType, error::ValidationError};

    use super::*;

    const TXT_DATA: &str = r#"TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 2312321321321321
TX_TYPE: TRANSFER
FROM_USER_ID: 1231231231231231
TO_USER_ID: 9876543210987654
AMOUNT: 1000
TIMESTAMP: 1633056800000
STATUS: FAILURE
DESCRIPTION: "User transfer"

TX_ID: 3213213213213213
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 9876543210987654
TO_USER_ID: 0
AMOUNT: 100
TIMESTAMP: 1633066800000
STATUS: PENDING
DESCRIPTION: "User withdrawal"
"#;

    fn tx1() -> Transaction {
        Transaction {
            tx_id: 1234567890123456,
            tx_type: TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 9876543210987654,
            amount: 10000,
            timestamp: 1633036800000,
            status: Status::SUCCESS,
            description: "Terminal deposit".to_string(),
        }
    }

    fn tx2() -> Transaction {
        Transaction {
            tx_id: 2312321321321321,
            tx_type: TxType::TRANSFER,
            from_user_id: 1231231231231231,
            to_user_id: 9876543210987654,
            amount: 1000,
            timestamp: 1633056800000,
            status: Status::FAILURE,
            description: "User transfer".to_string(),
        }
    }

    fn tx3() -> Transaction {
        Transaction {
            tx_id: 3213213213213213,
            tx_type: TxType::WITHDRAWAL,
            from_user_id: 9876543210987654,
            to_user_id: 0,
            amount: 100,
            timestamp: 1633066800000,
            status: Status::PENDING,
            description: "User withdrawal".to_string(),
        }
    }

    #[test]
    fn test_read_text() {
        let cursor = Cursor::new(TXT_DATA.as_bytes());

        let mut reader = TxtReader::new(cursor);

        assert_eq!(reader.read_tx(), Ok(Some(tx1())), "TX1 Reading error");
        assert_eq!(reader.read_tx(), Ok(Some(tx2())), "TX2 Reading error");
        assert_eq!(reader.read_tx(), Ok(Some(tx3())), "TX3 Reading error");
        assert_eq!(reader.read_tx(), Ok(None), "EOF Reading error");
    }

    #[test]
    fn test_read_text_missing_key_are_error() {
        let cursor = Cursor::new(
            r#"TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
"#
            .as_bytes(),
        );

        let mut reader = TxtReader::new(cursor);

        loop {
            match reader.read_tx() {
                Ok(Some(_)) => panic!("Should be error"),
                Ok(None) => break,
                Err(e) => match e {
                    ReaderError::RecordFormatError(_) => {
                        continue;
                    }
                    _ => {
                        panic!("Should be RecordFormatError");
                    }
                },
            }
        }
    }

    #[test]
    fn test_read_text_same_key_are_error() {
        let cursor = Cursor::new(
            r#"TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit""#
                .as_bytes(),
        );

        let mut reader = TxtReader::new(cursor);

        assert!(
            matches!(reader.read_tx(), Err(ReaderError::RecordFormatError(_))),
            "Should return RecordFormatError"
        );
    }

    #[test]
    fn test_read_text_wrong_key_are_error() {
        let cursor = Cursor::new(
            r#"TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
WRONG_KEY: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit""#
                .as_bytes(),
        );

        let mut reader = TxtReader::new(cursor);

        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::RecordFormatError(_))
        ), "Should return RecordFormatError");
    }

    #[test]
    fn test_read_text_wrong_value_are_error() {
        let cursor = Cursor::new(
            r#"TX_ID: WRONG
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: WRONG
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: WRONG
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: WRONG
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: WRONG
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: WRONG
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: WRONG
DESCRIPTION: "Terminal deposit"
"#
            .as_bytes(),
        );

        let mut reader = TxtReader::new(cursor);

        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::TxId,
                value: _
            }))
        ), "Parse wrong TxId shoud return error");
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::TxType,
                value: _
            }))
        ), "Parse wrong TxType shoud return error");
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::FromUserId,
                value: _
            }))
        ), "Parse wrong FromUserId shoud return error");
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::ToUserId,
                value: _
            }))
        ), "Parse wrong ToUserId shoud return error");
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::Amount,
                value: _
            }))
        ), "Parse wrong Amount shoud return error");
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::Timestamp,
                value: _
            }))
        ), "Parse wrong Timestamp shoud return error");
        assert!(matches!(
            reader.read_tx(),
            Err(ReaderError::FieldParseError(ParseError {
                field_name: FieldName::Status,
                value: _
            }))
        ), "Parse wrong Status shoud return error");
    }

    #[test]
    fn test_read_text_comments_are_not_error() {
        let cursor = Cursor::new(
            r#"TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
#Comment 1
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"
#Comment 2

#Comment 3
TX_ID: 2312321321321321
TIMESTAMP: 1633056800000
STATUS: FAILURE
TX_TYPE: TRANSFER
FROM_USER_ID: 1231231231231231
TO_USER_ID: 9876543210987654
#Comment 4
AMOUNT: 1000
DESCRIPTION: "User transfer"

#Comment 5
TX_ID: 3213213213213213
AMOUNT: 100
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 9876543210987654
TO_USER_ID: 0
#Comment 6
TIMESTAMP: 1633066800000
STATUS: PENDING
DESCRIPTION: "User withdrawal"
"#,
        );

        let mut reader = TxtReader::new(cursor);

        assert_eq!(reader.read_tx(), Ok(Some(tx1())));
        assert_eq!(reader.read_tx(), Ok(Some(tx2())));
        assert_eq!(reader.read_tx(), Ok(Some(tx3())));
        assert_eq!(reader.read_tx(), Ok(None));
    }

    #[test]
    fn test_read_text_validates_transactions() {
        let cursor = Cursor::new(
            r#"TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 100
TO_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 100
TO_USER_ID: 200
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: TRANSFER
FROM_USER_ID: 0
TO_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: TRANSFER
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: TRANSFER
FROM_USER_ID: 100
TO_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: TRANSFER
FROM_USER_ID: 100
TO_USER_ID: 200
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 0
TO_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 100
TO_USER_ID: 0
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"

TX_ID: 1234567890123456
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 100
TO_USER_ID: 200
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Desc"
"#,
        );

        let mut reader = TxtReader::new(cursor);

        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadToUserId
            ))
        ) {
            panic!("Deposit 0 0 shoud have BadToUserId");
        }
        if !matches!(reader.read_tx(), Ok(_)) {
            panic!("Deposit 0 100 should be valid");
        }
        if !matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))) {
            panic!("Deposit 100 0 should have BadToUserId or BadFromUserId");
        }
        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadFromUserId
            ))
        ) {
            panic!("Deposit 100 200 should have BadFromUserId");
        }
        if !matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))) {
            panic!("Transfer 0 0 should have BadToUserId or BadFromUserId");
        }
        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadFromUserId
            ))
        ) {
            panic!("Transfer 0 100 should have BadFromUserId");
        }
        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadToUserId
            ))
        ) {
            panic!("Transfer 100 0 should have BadToUserId");
        }
        if !matches!(reader.read_tx(), Ok(_)) {
            panic!("Transfer 100 200 should be valid");
        }
        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadFromUserId
            ))
        ) {
            panic!("Withdrawal 0 0 should have BadFromUserId");
        }
        if !matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))) {
            panic!("Withdrawal 0 100 should have BadFromUserId or BadToUserId");
        }
        if !matches!(reader.read_tx(), Ok(_)) {
            panic!("Withdrawal 100 0 should be valid");
        }
        if !matches!(
            reader.read_tx(),
            Err(ReaderError::RecordValidationError(
                ValidationError::BadToUserId
            ))
        ) {
            panic!("Withdrawal 100 200 should have BadToUserId");
        }
    }

    #[test]
    fn test_write_txt() {
        let mut data: Vec<u8> = Vec::new();

        {
            let mut writer = TxtWriter::new(&mut data);

            writer.write_tx(&tx1()).unwrap();
            writer.write_tx(&tx2()).unwrap();
            writer.write_tx(&tx3()).unwrap();
        }
        let txt_data = String::from_utf8(data).expect("Found invalid UTF-8");
        assert_eq!(txt_data, TXT_DATA);
    }
    enum ExpectedBehavior {
        Valid,
        AnyError,
        Error(ValidationError),
    }

    #[test]
    fn test_write_txt_validates_transactions() {
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

        let mut writer = TxtWriter::new(&mut data);

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
