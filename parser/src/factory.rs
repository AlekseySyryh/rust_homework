use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
};

use crate::{
    BinReader, BinWriter, CsvReader, CsvWriter, TransactionReader, TransactionWriter, TxtReader,
    TxtWriter,
};

/// Format of transactions file
pub enum Format {
    /// Binary
    BIN,
    /// CSV
    CSV,
    /// Text
    TXT,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Format::BIN => "bin",
            Format::CSV => "csv",
            Format::TXT => "txt",
        })
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bin" => Ok(Format::BIN),
            "csv" => Ok(Format::CSV),
            "txt" => Ok(Format::TXT),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

pub struct TransactionReaderFactory {}

impl TransactionReaderFactory {
    /// Creates a new TransactionReader instance.
    /// 
    /// #Examples
    /// 
    /// ```
    /// use std::io::Cursor;
    /// use parser::{TransactionReaderFactory, TransactionReader, Format};
    /// 
    /// let cursor = Cursor::new(vec![0; 100]);
    /// let reader = TransactionReaderFactory::create_transaction_reader(Format::BIN, cursor);
    /// ```
    pub fn create_transaction_reader<R: Read + 'static>(
        format: Format,
        reader: R,
    ) -> Result<Box<dyn TransactionReader>, String> {
        Ok(match format {
            Format::BIN => Box::new(BinReader::new(reader)),
            Format::CSV => Box::new(
                CsvReader::try_new(reader).map_err(|e| format!("Can not create reader {:?}", e))?,
            ),
            Format::TXT => Box::new(TxtReader::new(reader)),
        })
    }
}

pub struct TransactionWriterFactory {}

impl TransactionWriterFactory {
    /// Creates a new TransactionWriter instance.
    /// 
    /// #Examples
    /// ```
    /// use std::io::Cursor;
    /// use parser::{TransactionWriterFactory, TransactionWriter, Format};
    /// 
    /// let cursor = Cursor::new(vec![0; 100]);
    /// let writer = TransactionWriterFactory::create_transaction_writer(Format::BIN, cursor);
    /// ```
    pub fn create_transaction_writer<W: Write + 'static>(
        format: Format,
        writer: W,
    ) -> Result<Box<dyn TransactionWriter>, String> {
        Ok(match format {
            Format::BIN => Box::new(BinWriter::new(writer)),
            Format::CSV => Box::new(
                CsvWriter::try_new(writer).map_err(|e| format!("Can not create writer {:?}", e))?,
            ),
            Format::TXT => Box::new(TxtWriter::new(writer)),
        })
    }
}
