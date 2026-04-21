pub mod error;

use crate::error::{ReaderError, WriterError};

#[derive(Debug, PartialEq, Default)]
pub struct Transaction {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: Status,
    pub description: String,
}

#[derive(Debug, PartialEq, Default)]
pub enum TxType {
    DEPOSIT,
    #[default]
    TRANSFER,
    WITHDRAWAL,
}

#[derive(Debug, PartialEq, Default)]
pub enum Status {
    #[default]
    SUCCESS,
    FAILURE,
    PENDING,
}

pub trait TransactionReader {
    fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError>;

    fn read_vector(&mut self) -> Result<Vec<Transaction>, ReaderError> {
        let mut result: Vec<Transaction> = Vec::new();

        while let Some(tx) = self.read_tx()? {
            result.push(tx);
        }

        Ok(result)
    }
}
pub trait TransactionWriter {
    fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError>;

    fn write_vector(&mut self, txs: &[Transaction]) -> Result<(), WriterError> {
        for tx in txs {
            self.write_tx(tx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
                Err(e) => Err(ReaderError::ParseError(format!(
                    "Failed to read transaction: {}",
                    e
                ))),
            }
        }
    }

    #[test]
    fn test_read_multiple_transactions() {
        let data: &[u8] = &[10, 20, 30];

        let mut reader: Box<dyn TransactionReader> = Box::new(FakeReader { data: data });

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
            Err(ReaderError::ParseError("Error".to_string()))
        }
    }

    #[test]
    fn test_read_error() {
        let mut reader = ErrorReader;
        let result = reader.read_vector();
        assert_eq!(result, Err(ReaderError::ParseError("Error".to_string())));
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
}
