use std::io::{Error, Read, Write};

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub tx_id: u64
}

pub trait TransactionCodec {
    fn read_tx<R: Read>(data: &mut R) -> Result<Option<Transaction>, Error>;
    fn write_tx<W: Write>(data: &mut W, tx: &Transaction) -> Result<(), Error>;
}

pub struct TransactionReader;

impl TransactionReader {
    pub fn read<R: Read, T: TransactionCodec>(data: &mut R) -> Result<Vec<Transaction>, Error> {
        let mut result: Vec<Transaction> = Vec::new();

        while let Some(tx) = T::read_tx(data)? {
            result.push(tx);
        }

        Ok(result)
    }
}

pub struct TransactionWriter;

impl TransactionWriter {
    pub fn write<W: Write, T: TransactionCodec>(data: &mut W, txs: &Vec<Transaction>) -> Result<(), Error> {
        for tx in txs {
            T::write_tx(data, tx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeCodec;

    impl TransactionCodec for FakeCodec {
        fn read_tx<R: Read>(data: &mut R) -> Result<Option<Transaction>, Error> {
            let mut buf = [0u8; 1];

            match data.read(&mut buf)? {
                0 => Ok(None),
                _ => Ok(Some(Transaction {
                    tx_id: buf[0] as u64
                })),
            }
        }

        fn write_tx<W: Write>(data: &mut W, tx: &Transaction) -> Result<(), Error> {
            data.write(&[tx.tx_id as u8])?;
            Ok(())
        }
    }

    #[test]
    fn test_read_multiple_transactions() {
        let mut data = &vec![10, 20, 30][..];

        let result = TransactionReader::read::<_, FakeCodec>(&mut data).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].tx_id, 10);
        assert_eq!(result[1].tx_id, 20);
        assert_eq!(result[2].tx_id, 30);
    }

    #[test]
    fn test_write_multiple_transactions() {
        let txs: &Vec<Transaction> = &vec![Transaction { tx_id: 10 }, Transaction { tx_id: 20 }, Transaction { tx_id: 30 }];

        let mut data = Vec::new();

        TransactionWriter::write::<_, FakeCodec>(&mut data, &txs).unwrap();

        assert_eq!(data, vec![10, 20, 30]);
    }
}
