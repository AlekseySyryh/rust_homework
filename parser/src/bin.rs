use std::io::{BufReader, IoSlice, Read, Write};

use crate::{
    ReaderError, Status, Transaction, TransactionReader, TransactionWriter, TxType, WriterError,
};

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

/// Bin reader
pub struct BinReader<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> BinReader<R> {
    /// Creates a new Bin reader.
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use parser::{BinReader, TransactionReader, Transaction, TxType, Status};
    ///
    /// let bin_data: [u8; 213] = [
    ///     0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x3F,
    ///     0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///     0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///     0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94, 0xFA,
    ///     0x60, 0x01, 0x00, 0x00, 0x00, 0x11, 0x22, 0x52,
    ///     0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75,
    ///     0x6D, 0x62, 0x65, 0x72, 0x20, 0x31, 0x22, 0x59,
    ///     0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x3F, 0x00,
    ///     0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x01, 0x01,
    ///     0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///     0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8,
    ///     0x00, 0x00, 0x01, 0x7C, 0x38, 0x95, 0xE4, 0xC0,
    ///     0x02, 0x00, 0x00, 0x00, 0x11, 0x22, 0x52, 0x65,
    ///     0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D,
    ///     0x62, 0x65, 0x72, 0x20, 0x32, 0x22, 0x59, 0x50,
    ///     0x42, 0x4E, 0x00, 0x00, 0x00, 0x3F, 0x00, 0x03,
    ///     0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x02, 0x02, 0x08,
    ///     0x50, 0x68, 0x64, 0x76, 0x76, 0xC2, 0x68, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2C, 0x00,
    ///     0x00, 0x01, 0x7C, 0x38, 0x96, 0xCF, 0x20, 0x00,
    ///     0x00, 0x00, 0x00, 0x11, 0x22, 0x52, 0x65, 0x63,
    ///     0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62,
    ///     0x65, 0x72, 0x20, 0x33, 0x22];
    ///
    /// let cursor = Cursor::new(bin_data);
    /// let mut reader = BinReader::new(cursor);
    ///
    /// let txs = reader.read_vector().unwrap();
    ///
    /// assert_eq!(txs.len(), 3, "Vector length is 3");
    /// assert_eq!(txs[0],
    ///     Transaction {
    ///         tx_id: 0x00038D7EA4C68000,
    ///         tx_type: TxType::DEPOSIT,
    ///         from_user_id: 0,
    ///         to_user_id: 0x7FFFFFFFFFFFFFFF,
    ///         amount: 100,
    ///         timestamp: 0x0000017C3894FA60,
    ///         status: Status::FAILURE,
    ///         description: "Record number 1".to_string(),
    ///     }, "Tx1 is incorrect");
    /// assert_eq!(txs[1],
    ///     Transaction {
    ///         tx_id: 0x00038D7EA4C68001,
    ///         tx_type: TxType::TRANSFER,
    ///         from_user_id: 0x7FFFFFFFFFFFFFFF,
    ///         to_user_id: 0x7FFFFFFFFFFFFFFF,
    ///         amount: 200,
    ///         timestamp: 0x0000017C3895E4C0,
    ///         status: Status::PENDING,
    ///         description: "Record number 2".to_string(),
    ///    }, "Tx2 is incorrect");
    /// assert_eq!(txs[2],
    ///     Transaction {
    ///         tx_id: 0x00038D7EA4C68002,
    ///         tx_type: TxType::WITHDRAWAL,
    ///         from_user_id: 0x085068647676C268,
    ///         to_user_id: 0,
    ///         amount: 300,
    ///         timestamp: 0x0000017C3896CF20,
    ///         status: Status::SUCCESS,
    ///         description: "Record number 3".to_string(),
    ///     }, "Tx3 is incorrect");
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
        }
    }
}

impl<R: Read> TransactionReader for BinReader<R> {
    fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError> {
        let mut buf = [0; 4];

        if let Err(_) = self.reader.read_exact(&mut buf) {
            return Ok(None);
        }

        if buf != MAGIC {
            loop {
                buf.rotate_left(1);
                if let Err(_) = self.reader.read_exact(&mut buf[3..4]) {
                    return Ok(None);
                }
                if buf == MAGIC {
                    break;
                }
            }
        }

        let mut len_buf = [0u8; 4];
        let mut tx_id_buf = [0u8; 8];
        let mut tx_type_buf = [0u8; 1];
        let mut from_user_id_buf = [0u8; 8];
        let mut to_user_id_buf = [0u8; 8];
        let mut amount_buf = [0u8; 8];
        let mut timestamp_buf = [0u8; 8];
        let mut status_buf = [0u8; 1];
        let mut desc_len_buf = [0u8; 4];

        let mut bufs = [
            &mut len_buf[..],
            &mut tx_id_buf[..],
            &mut tx_type_buf[..],
            &mut from_user_id_buf[..],
            &mut to_user_id_buf[..],
            &mut amount_buf[..],
            &mut timestamp_buf[..],
            &mut status_buf[..],
            &mut desc_len_buf[..],
        ];

        for buf in bufs.iter_mut() {
            self.reader
                .read_exact(buf)
                .map_err(|e| ReaderError::FileFormatError(format!("{e:}")))?;
        }

        let len = u32::from_be_bytes(len_buf);
        let desc_len: u32 = u32::from_be_bytes(desc_len_buf);

        if len != 46 + desc_len {
            return Err(ReaderError::RecordFormatError(format!(
                "Invalid record length. Len = {len}, Desc_len = {desc_len}"
            )));
        }

        let mut desc_buf = vec![0u8; desc_len as usize];
        self.reader
            .read_exact(&mut desc_buf)
            .map_err(|e| ReaderError::FileFormatError(format!("Read error {:?}", e)))?;

        let tx = Transaction {
            tx_id: u64::from_be_bytes(tx_id_buf),
            tx_type: TxType::try_from(tx_type_buf[0])
                .map_err(|e| ReaderError::FieldParseError(e))?,
            from_user_id: u64::from_be_bytes(from_user_id_buf),
            to_user_id: u64::from_be_bytes(to_user_id_buf),
            amount: u64::from_be_bytes(amount_buf),
            timestamp: u64::from_be_bytes(timestamp_buf),
            status: Status::try_from(status_buf[0]).map_err(|e| ReaderError::FieldParseError(e))?,
            description: String::from_utf8(desc_buf)
                .map_err(|e| {
                    ReaderError::FileFormatError(format!("Invalid UTF-8 sequence: {:?}", e))
                })?
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

/// Bin writer
pub struct BinWriter<W: Write> {
    writer: W,
}

impl<W: Write> BinWriter<W> {
    /// Creates a new BinWriter instance.
    ///
    /// #Examples
    /// ```
    /// use std::io::Cursor;
    /// use parser::{BinWriter, TransactionWriter, Transaction, TxType, Status};
    ///
    /// let transactions = vec![
    ///     Transaction {
    ///         tx_id: 0x00038D7EA4C68000,
    ///         tx_type: TxType::DEPOSIT,
    ///         from_user_id: 0,
    ///         to_user_id: 0x7FFFFFFFFFFFFFFF,
    ///         amount: 100,
    ///         timestamp: 0x0000017C3894FA60,
    ///         status: Status::FAILURE,
    ///         description: "Record number 1".to_string(),
    ///     },
    ///     Transaction {
    ///         tx_id: 0x00038D7EA4C68001,
    ///         tx_type: TxType::TRANSFER,
    ///         from_user_id: 0x7FFFFFFFFFFFFFFF,
    ///         to_user_id: 0x7FFFFFFFFFFFFFFF,
    ///         amount: 200,
    ///         timestamp: 0x0000017C3895E4C0,
    ///         status: Status::PENDING,
    ///         description: "Record number 2".to_string(),
    ///     },
    ///     Transaction {
    ///         tx_id: 0x00038D7EA4C68002,
    ///         tx_type: TxType::WITHDRAWAL,
    ///         from_user_id: 0x085068647676C268,
    ///         to_user_id: 0,
    ///         amount: 300,
    ///         timestamp: 0x0000017C3896CF20,
    ///         status: Status::SUCCESS,
    ///         description: "Record number 3".to_string(),
    ///      }
    /// ];
    ///
    /// let mut data: Vec<u8> = Vec::new();
    /// {
    ///     let mut writer = BinWriter::new(&mut data);
    ///     writer.write_vector(&transactions).unwrap();
    /// }
    ///
    /// let expected_bin_data: [u8; 213] = [
    ///     0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x3F,
    ///     0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///     0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///     0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94, 0xFA,
    ///     0x60, 0x01, 0x00, 0x00, 0x00, 0x11, 0x22, 0x52,
    ///     0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75,
    ///     0x6D, 0x62, 0x65, 0x72, 0x20, 0x31, 0x22, 0x59,
    ///     0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x3F, 0x00,
    ///     0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x01, 0x01,
    ///     0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///     0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8,
    ///     0x00, 0x00, 0x01, 0x7C, 0x38, 0x95, 0xE4, 0xC0,
    ///     0x02, 0x00, 0x00, 0x00, 0x11, 0x22, 0x52, 0x65,
    ///     0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D,
    ///     0x62, 0x65, 0x72, 0x20, 0x32, 0x22, 0x59, 0x50,
    ///     0x42, 0x4E, 0x00, 0x00, 0x00, 0x3F, 0x00, 0x03,
    ///     0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x02, 0x02, 0x08,
    ///     0x50, 0x68, 0x64, 0x76, 0x76, 0xC2, 0x68, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2C, 0x00,
    ///     0x00, 0x01, 0x7C, 0x38, 0x96, 0xCF, 0x20, 0x00,
    ///     0x00, 0x00, 0x00, 0x11, 0x22, 0x52, 0x65, 0x63,
    ///     0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62,
    ///     0x65, 0x72, 0x20, 0x33, 0x22];
    ///
    /// assert_eq!(data, expected_bin_data, "Bin data mismatch");
    /// ```
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> TransactionWriter for BinWriter<W> {
    fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError> {
        tx.validate()
            .map_err(|e| WriterError::RecordValidationError(e))?;

        let desc_len: u32 = tx.description.len() as u32 + 2;
        let len: u32 = 46 + desc_len as u32;

        let len_buf = len.to_be_bytes();
        let tx_id_buf = tx.tx_id.to_be_bytes();
        let tx_type_buf = [tx.tx_type as u8];
        let from_user_id_buf = tx.from_user_id.to_be_bytes();
        let to_user_id_buf = tx.to_user_id.to_be_bytes();
        let amount_buf = tx.amount.to_be_bytes();
        let timestamp_buf = tx.timestamp.to_be_bytes();
        let status_buf = [tx.status as u8];
        let desc_len_buf = desc_len.to_be_bytes();
        let quote_buf: [u8; 1] = [b'"'];
        let desc_buf = tx.description.as_bytes();

        let bufs = [
            IoSlice::new(&MAGIC),
            IoSlice::new(&len_buf),
            IoSlice::new(&tx_id_buf),
            IoSlice::new(&tx_type_buf),
            IoSlice::new(&from_user_id_buf),
            IoSlice::new(&to_user_id_buf),
            IoSlice::new(&amount_buf),
            IoSlice::new(&timestamp_buf),
            IoSlice::new(&status_buf),
            IoSlice::new(&desc_len_buf),
            IoSlice::new(&quote_buf),
            IoSlice::new(desc_buf),
            IoSlice::new(&quote_buf),
        ];

        self.writer
            .write_vectored(&bufs)
            .map_err(|e| WriterError::WriterError(format!("{e}")))?;

        Ok(())
    }
}

impl<W: Write> Drop for BinWriter<W> {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{FieldName, ValidationError, error::ParseError};

    use {Status, Transaction, TxType};

    use super::*;

    fn bin_data() -> Vec<u8> {
        vec![
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x00, //TX_ID: 0x00038D7EA4C68000
            0x00, //TX_TYPE: Deposit
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //AMOUNT: 100
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x94, 0xFA, 0x60, //TIMESTAMP: 0x0000017C3894FA60
            0x01, //STATUS: FAILURE
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION: "Record number 1"
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x01, //TX_ID: 0x00038D7EA4C68001
            0x01, //TX_TYPE: Transfer
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //FROM_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8, //AMOUNT: 200
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x95, 0xE4, 0xC0, //TIMESTAMP: 0x0000017C3895E4C0
            0x02, //STATUS: PENDING
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x32, 0x22, // DESCRIPTION: "Record number 2"
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x02, //TX_ID: 0x00038D7EA4C68002
            0x02, //TX_TYPE: Withdrawal
            0x08, 0x50, 0x68, 0x64, 0x76, 0x76, 0xC2, 0x68, //FROM_USER_ID: 0x085068647676C268
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2C, //AMOUNT: 300
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x96, 0xCF, 0x20, //TIMESTAMP: 0x0000017C3896CF20
            0x00, //STATUS: SUCCESS
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x33, 0x22, // DESCRIPTION: "Record number 3"
        ]
    }

    fn tx1() -> Transaction {
        Transaction {
            tx_id: 0x00038D7EA4C68000,
            tx_type: TxType::DEPOSIT,
            from_user_id: 0,
            to_user_id: 0x7FFFFFFFFFFFFFFF,
            amount: 100,
            timestamp: 0x0000017C3894FA60,
            status: Status::FAILURE,
            description: "Record number 1".to_string(),
        }
    }
    fn tx2() -> Transaction {
        Transaction {
            tx_id: 0x00038D7EA4C68001,
            tx_type: TxType::TRANSFER,
            from_user_id: 0x7FFFFFFFFFFFFFFF,
            to_user_id: 0x7FFFFFFFFFFFFFFF,
            amount: 200,
            timestamp: 0x0000017C3895E4C0,
            status: Status::PENDING,
            description: "Record number 2".to_string(),
        }
    }

    fn tx3() -> Transaction {
        Transaction {
            tx_id: 0x00038D7EA4C68002,
            tx_type: TxType::WITHDRAWAL,
            from_user_id: 0x085068647676C268,
            to_user_id: 0,
            amount: 300,
            timestamp: 0x0000017C3896CF20,
            status: Status::SUCCESS,
            description: "Record number 3".to_string(),
        }
    }

    #[test]
    fn test_bin_reader() {
        let mut reader = BinReader::new(Cursor::new(bin_data()));

        assert_eq!(reader.read_tx(), Ok(Some(tx1())), "Read TX1 failed");
        assert_eq!(reader.read_tx(), Ok(Some(tx2())), "Read TX2 failed");
        assert_eq!(reader.read_tx(), Ok(Some(tx3())), "Read TX3 failed");
        assert_eq!(reader.read_tx(), Ok(None), "Read EOF failed");
    }

    #[test]
    fn test_read_filed_validation_test() {
        let data = vec![
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x00, //TX_ID: 0x00038D7EA4C68000
            0xFF, //TX_TYPE: WRONG
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //AMOUNT: 100
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x94, 0xFA, 0x60, //TIMESTAMP: 0x0000017C3894FA60
            0x01, //STATUS: FAILURE
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION: "Record number 1"
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x01, //TX_ID: 0x00038D7EA4C68001
            0x01, //TX_TYPE: Transfer
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //FROM_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8, //AMOUNT: 200
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x95, 0xE4, 0xC0, //TIMESTAMP: 0x0000017C3895E4C0
            0xEE, //STATUS: WRONG
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x32, 0x22, // DESCRIPTION: "Record number 2"
        ];

        let mut reader = BinReader::new(Cursor::new(data));

        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::TxType,
                    ..
                }))
            ),
            "Wrong TxType parsing test failed"
        );

        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::FieldParseError(ParseError {
                    field_name: FieldName::Status,
                    ..
                }))
            ),
            "Wrong Status parsing test failed"
        );
    }

    #[test]
    fn test_read_size_mismatch_test() {
        let data = vec![
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0xFF, 0xFF, //Length:  65535
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x00, //TX_ID: 0x00038D7EA4C68000
            0x00, //TX_TYPE: DEPOSIT
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //AMOUNT: 100
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x94, 0xFA, 0x60, //TIMESTAMP: 0x0000017C3894FA60
            0x01, //STATUS: FAILURE
            0x00, 0x00, 0x00, 0xFF, //DESC_LEN: 255
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION: "Record number 1"
        ];
        // Length should be 46 + Desc_len

        let mut reader = BinReader::new(Cursor::new(data));

        assert!(
            matches!(reader.read_tx(), Err(ReaderError::RecordFormatError(_))),
            "Size mismatch test failed"
        );
    }

    #[test]
    fn test_bin_reader_resync() {
        let data = vec![
            0x04, 0x23, 0x43, 0x59, //Random bytes
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x00, //TX_ID: 0x00038D7EA4C68000
            0x00, //TX_TYPE: Deposit
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //AMOUNT: 100
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x94, 0xFA, 0x60, //TIMESTAMP: 0x0000017C3894FA60
            0x01, //STATUS: FAILURE
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION: "Record number 1"
            0x00, //Random bytes
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x01, //TX_ID: 0x00038D7EA4C68001
            0x01, //TX_TYPE: Transfer
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //FROM_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //TO_USER_ID: 0x7FFFFFFFFFFFFFFF
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8, //AMOUNT: 200
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x95, 0xE4, 0xC0, //TIMESTAMP: 0x0000017C3895E4C0
            0x02, //STATUS: PENDING
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x32, 0x22, // DESCRIPTION: "Record number 2"
            0x00, 0xFF, 0x00, 0xFF, //Random bytes
            0x59, 0x50, 0x42, 0x4E, //Magic
            0x00, 0x00, 0x00, 0x3F, //Length:  63
            0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6, 0x80, 0x02, //TX_ID: 0x00038D7EA4C68002
            0x02, //TX_TYPE: Withdrawal
            0x08, 0x50, 0x68, 0x64, 0x76, 0x76, 0xC2, 0x68, //FROM_USER_ID: 0x085068647676C268
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2C, //AMOUNT: 300
            0x00, 0x00, 0x01, 0x7C, 0x38, 0x96, 0xCF, 0x20, //TIMESTAMP: 0x0000017C3896CF20
            0x00, //STATUS: SUCCESS
            0x00, 0x00, 0x00, 0x11, //DESC_LEN: 17
            0x22, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x20, 0x6E, 0x75, 0x6D, 0x62, 0x65, 0x72,
            0x20, 0x33, 0x22, // DESCRIPTION: "Record number 3"
        ];

        let mut reader = BinReader::new(Cursor::new(data));

        assert_eq!(reader.read_tx(), Ok(Some(tx1())));
        assert_eq!(reader.read_tx(), Ok(Some(tx2())));
        assert_eq!(reader.read_tx(), Ok(Some(tx3())));
        assert_eq!(reader.read_tx(), Ok(None));
    }

    #[test]
    fn test_bin_reader_validates() {
        let data = vec![
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x00, //TX_TYPE: Deposit
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Depostit 0 0
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x00, //TX_TYPE: Deposit
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //TO_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Depostit 0 100
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x00, //TX_TYPE: Deposit
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //FROM_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Depostit 100 0
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x00, //TX_TYPE: Deposit
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //FROM_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8, //TO_USER_ID: 200
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Depostit 100 200
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x01, //TX_TYPE: Transfer
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Transfer 0 0
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x01, //TX_TYPE: Transfer
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //TO_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Transfer 0 100
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x01, //TX_TYPE: Transfer
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //FROM_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Transfer 100 0
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x01, //TX_TYPE: Transfer
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //FROM_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8, //TO_USER_ID: 200
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Transfer 100 200
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x02, //TX_TYPE: Withdrawal
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Withdraw 0 0
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x02, //TX_TYPE: Withdrawal
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //FROM_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //TO_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Withdraw 0 100
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x02, //TX_TYPE: Withdrawal
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //FROM_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //TO_USER_ID: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Withdraw 100 0
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x03, 0x8D, 0x7E, 0xA4, 0xC6,
            0x80, 0x00, 0x02, //TX_TYPE: Withdrawal
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, //FROM_USER_ID: 100
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC8, //TO_USER_ID: 200
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x01, 0x7C, 0x38, 0x94,
            0xFA, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, //Withdraw 100 200
        ];

        let mut reader = BinReader::new(Cursor::new(data));

        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::RecordValidationError(
                    ValidationError::BadToUserId
                ))
            ),
            "Deposit 0 0 should have BadToUserId"
        );

        assert!(
            matches!(reader.read_tx(), Ok(_)),
            "Deposit 0 100 should be valid"
        );
        assert!(
            matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))),
            "Deposit 100 0 should have ValidationError"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::RecordValidationError(
                    ValidationError::BadFromUserId
                ))
            ),
            "Deposit 100 200 should have BadFromUserId"
        );

        assert!(
            matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))),
            "Transfer 0 0 should have ValidationError"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::RecordValidationError(
                    ValidationError::BadFromUserId
                ))
            ),
            "Transfer 0 100 should have BadFromUserId"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::RecordValidationError(
                    ValidationError::BadToUserId
                ))
            ),
            "Transfer 100 0 should have BadFromUserId"
        );
        assert!(
            matches!(reader.read_tx(), Ok(_)),
            "Transfer 100 200 should be valid"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::RecordValidationError(
                    ValidationError::BadFromUserId
                ))
            ),
            "Withdraw 0 0 should have BadFromUserId"
        );
        assert!(
            matches!(reader.read_tx(), Err(ReaderError::RecordValidationError(_))),
            "Withdraw 0 100 should have ValidationError"
        );
        assert!(
            matches!(reader.read_tx(), Ok(_)),
            "Withdraw 100 0 should be valid"
        );
        assert!(
            matches!(
                reader.read_tx(),
                Err(ReaderError::RecordValidationError(
                    ValidationError::BadToUserId
                ))
            ),
            "Withdraw 100 2000 should have BadToUserId"
        );
    }

    #[test]
    fn test_bin_writer() {
        let mut data: Vec<u8> = Vec::new();

        {
            let mut writer = BinWriter::new(&mut data);

            writer.write_tx(&tx1()).unwrap();
            writer.write_tx(&tx2()).unwrap();
            writer.write_tx(&tx3()).unwrap();
        }
        assert_eq!(data, bin_data(), "Bin data should match");
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
                    tx_type: TxType::DEPOSIT,
                    from_user_id: 0,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadToUserId),
                "Deposit 0 0 should return BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::DEPOSIT,
                    from_user_id: 0,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Valid,
                "Deposit 0 501 should be valid".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::DEPOSIT,
                    from_user_id: 500,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::AnyError,
                "Deposit 500 0 should return BadFromUserId or BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::DEPOSIT,
                    from_user_id: 500,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadFromUserId),
                "Deposit 500 501 should return BadFromUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::TRANSFER,
                    from_user_id: 0,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::AnyError,
                "Transfer 0 0 should return BadFromUserId or BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::TRANSFER,
                    from_user_id: 0,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadFromUserId),
                "Transfer 0 501 should return BadFromUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::TRANSFER,
                    from_user_id: 500,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadToUserId),
                "Transfer 500 0 should return BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::TRANSFER,
                    from_user_id: 500,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Valid,
                "Transfer 500 501 should be valid".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::WITHDRAWAL,
                    from_user_id: 0,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadFromUserId),
                "Withdrawal 0 0 should return BadFromUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::WITHDRAWAL,
                    from_user_id: 0,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::AnyError,
                "Withdrawal 0 501 should return BadFromUserId or BadToUserId".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::WITHDRAWAL,
                    from_user_id: 500,
                    to_user_id: 0,
                    ..Default::default()
                },
                ExpectedBehavior::Valid,
                "Withdrawal 500 0 should be valid".to_string(),
            ),
            (
                Transaction {
                    tx_type: TxType::WITHDRAWAL,
                    from_user_id: 500,
                    to_user_id: 501,
                    ..Default::default()
                },
                ExpectedBehavior::Error(ValidationError::BadToUserId),
                "Withdrawal 500 501 should return BadToUserId".to_string(),
            ),
        ];

        let mut data: Vec<u8> = Vec::new();

        let mut writer = BinWriter::new(&mut data);

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
